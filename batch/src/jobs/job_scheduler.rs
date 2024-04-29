use super::{
    error::JobError, IntrospectedJobDetails, Job, JobContext, JobDetails, JobState, JobType,
    RunningState,
};
use anyhow::Context;
use base::delay_queue::{DelayQueue, IntrospectionResult};
use entity::jobs;
use sea_orm::{
    prelude::Uuid, sea_query::OnConflict, ActiveModelTrait, ColumnTrait, DatabaseConnection,
    EntityTrait, QueryFilter, Set, Unchanged,
};
use serde::Serialize;
use serde_json::Value;
use std::{collections::HashMap, fmt::Debug, ops::ControlFlow, sync::Arc, time::Duration};
use tokio::{sync::RwLock, task::JoinHandle};
use tokio_util::sync::CancellationToken;
use tracing::{Instrument, Level};

// FIXME: confusing to have 2 params that mean opposite
#[derive(Clone, Serialize)]
pub(crate) enum Message {
    JobFinished { name: String, queue_next: bool },
    Init,
    RunJob { name: String, one_shot: bool },
}

impl Debug for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::JobFinished { name, .. } => {
                write!(f, "JobFinished: {}", name)
            }
            Self::Init => write!(f, "Init"),
            Self::RunJob { name, .. } => {
                write!(f, "RunPending: {}", name)
            }
        }
    }
}

pub struct JobSchedulerInner {
    cancellation_token: CancellationToken,
    jobs: Arc<RwLock<HashMap<String, JobDetails>>>,
    db: DatabaseConnection,
    task_queue: DelayQueue<Message>,
}

#[derive(Clone)]
pub struct JobScheduler(Arc<JobSchedulerInner>);

impl JobScheduler {
    pub fn new(db: DatabaseConnection) -> Self {
        JobScheduler(
            JobSchedulerInner {
                db,
                cancellation_token: Default::default(),
                jobs: Default::default(),
                task_queue: Default::default(),
            }
            .into(),
        )
    }

    pub async fn introspect(
        &self,
    ) -> (
        Vec<IntrospectedJobDetails>,
        Vec<IntrospectionResult<Message>>,
    ) {
        (
            self.0
                .jobs
                .read()
                .await
                .values()
                .map(|j| IntrospectedJobDetails {
                    name: j.job.name(),
                    state: match j.state {
                        JobState::Stopped => super::IntrospectedJobState::Stopped,
                        JobState::Running(_) => super::IntrospectedJobState::Running,
                    },
                })
                .collect(),
            self.0.task_queue.introspect().await,
        )
    }

    pub fn db(&self) -> &DatabaseConnection {
        &self.0.db
    }

    pub async fn add_scheduled<T>(&self, job: T, schedule: cron::Schedule) -> &Self
    where
        T: Job + 'static,
    {
        let mut jobs = self.0.jobs.write().await;
        jobs.insert(
            job.name(),
            JobDetails::new(Arc::new(job), JobType::Schedule(schedule)),
        );

        self
    }

    pub async fn add_manual<T>(&self, job: T) -> &Self
    where
        T: Job + 'static,
    {
        let mut jobs = self.0.jobs.write().await;
        jobs.insert(job.name(), JobDetails::new(Arc::new(job), JobType::Manual));

        self
    }

    pub async fn init(&self) -> Result<(), JobError> {
        let jobs = self.0.jobs.read().await;
        for (name, _) in jobs.iter() {
            let id = Uuid::new_v4();

            let job_model = jobs::ActiveModel {
                id: Set(id),
                name: Set(name.clone()),
                ..Default::default()
            };

            jobs::Entity::insert(job_model)
                .on_conflict(
                    OnConflict::columns([jobs::Column::Name])
                        .do_nothing()
                        .to_owned(),
                )
                .exec_without_returning(&self.0.db)
                .await?;
        }

        self.0.task_queue.push(Message::Init, Duration::ZERO).await;

        Ok(())
    }

    pub async fn run_job_one_shot(&self, name: &str) -> Result<(), JobError> {
        let jobs = self.0.jobs.read().await;
        let job_to_run = jobs.get(name).context("can't find job with name")?;

        self.0
            .task_queue
            .push(
                Message::RunJob {
                    name: job_to_run.job.name(),
                    one_shot: true,
                },
                Duration::ZERO,
            )
            .await;

        Ok(())
    }

    pub async fn shutdown(&self) {
        let mut jobs = self.0.jobs.write().await;
        let mut cancellation_futures = Vec::new();

        for (_, job_details) in jobs.iter_mut() {
            match job_details.state {
                JobState::Stopped => {}
                JobState::Running(ref mut state) => {
                    state.cancellation_token.cancel();
                    cancellation_futures.push(&mut state.handle)
                }
            }
        }

        futures::future::join_all(cancellation_futures).await;
        tracing::info!("shutting down job scheduler");
        self.0.cancellation_token.cancel()
    }

    pub async fn run(&self) -> JoinHandle<std::result::Result<(), JobError>> {
        let cloned_self = self.clone();
        tokio::spawn(async move {
            while let ControlFlow::Continue(_) = cloned_self.tick().await? {}
            Ok(())
        })
    }

    async fn handle_job_finish(&self, name: &str, queue_next: bool) -> Result<(), JobError> {
        let mut jobs = self.0.jobs.write().await;
        match jobs.get_mut(name) {
            Some(job_details) => {
                job_details.state = JobState::Stopped;
                let job_model = jobs::Entity::find()
                    .filter(jobs::Column::Name.eq(name))
                    .one(&self.0.db)
                    .await?
                    .unwrap();

                let last_execution = job_model.last_execution;
                let job_type = &job_details.job_type;
                let time_now = chrono::offset::Utc::now();

                if queue_next {
                    // if no last execution, execute immediately
                    let next = match job_type {
                        JobType::Schedule(schedule) => match last_execution {
                            Some(t) => schedule.after(&t.and_utc()).next(),
                            None => Some(time_now),
                        }
                        .unwrap(),
                        JobType::Manual => {
                            return Ok(());
                        }
                    };

                    tracing::info!("time now: {}", time_now);
                    tracing::info!("next scheduled run: {}", next);

                    let time_until = if time_now >= next {
                        Duration::ZERO
                    } else {
                        (next - time_now).to_std()?
                    };

                    // setup next wake up for this job
                    self.0
                        .task_queue
                        .push(
                            Message::RunJob {
                                name: job_model.name,
                                one_shot: false,
                            },
                            time_until,
                        )
                        .await;
                }
            }
            None => tracing::error!(
                "could not find job by name for state to set state to finished: {}",
                name
            ),
        }

        Ok(())
    }

    async fn handle_run_job(&self, name: &str, queue_next: bool) -> Result<(), JobError> {
        let mut jobs = self.0.jobs.write().await;

        let job_model = jobs::Entity::find()
            .filter(jobs::Column::Name.eq(name))
            .one(&self.0.db)
            .await?
            .unwrap();

        let job_details = jobs.get_mut(name).unwrap();

        let cancellation_token = CancellationToken::new();
        let cancellation_token_cloned = cancellation_token.clone();
        let job = job_details.job.clone();

        let running = match job_details.state {
            JobState::Stopped => false,
            JobState::Running(_) => true,
        };

        if !running {
            let db = self.0.db.clone();
            let context = JobContext::new(self.0.db.clone(), job_model.id);
            let span = tracing::span!(parent: None, Level::INFO, "job", job_name = name, "otel.name" = name);
            let queue = self.0.task_queue.clone();
            let task_name = name.to_string();

            let execution_id = entity::job_history::ActiveModel {
                context: Set(context.get::<Value>().await),
                job_name: Set(job_model.name),
                ..Default::default()
            }
            .insert(&db)
            .await?
            .id;

            let handle = tokio::spawn(
                async move {
                    tracing::info!("triggered job {}", job.name());
                    let result = job.execute(&context, cancellation_token_cloned).await;
                    let error = result.map_err(|e| e.to_string()).err();

                    let time_now = chrono::offset::Utc::now();

                    if error.is_some() {
                        tracing::error!("error with job completion: {:?}", &error)
                    }

                    if let Err(e) = {
                        let current_context = context.get::<serde_json::Value>().await;

                        entity::job_history::ActiveModel {
                            id: Unchanged(execution_id),
                            error: Set(error.is_some()),
                            error_message: Set(error.clone()),
                            completed_at: Set(Some(time_now.naive_utc())),
                            context: Set(current_context),

                            ..Default::default()
                        }
                        .update(&db)
                        .await
                    } {
                        tracing::error!("error setting job error: {}, {:?}", e, error)
                    }

                    let update_finish_time = jobs::ActiveModel {
                        id: Set(job_model.id),
                        name: Set(task_name.to_string()),
                        last_execution: Set(Some(time_now.naive_utc())),
                        ..Default::default()
                    }
                    .update(&db)
                    .await;

                    if let Err(e) = update_finish_time {
                        tracing::error!("error setting last execution: {}", e)
                    }

                    // must send after updating last execution or it can trigger twice
                    // race condition
                    queue
                        .push(
                            Message::JobFinished {
                                name: task_name,
                                queue_next,
                            },
                            Duration::ZERO,
                        )
                        .await;
                }
                .instrument(span),
            );

            job_details.state = JobState::Running(RunningState {
                cancellation_token,
                handle,
            });
        };
        Ok(())
    }

    async fn handle_message(&self, message: Message) -> Result<(), JobError> {
        match message {
            Message::JobFinished { name, queue_next } => {
                tracing::info!("received message for job finished: {}", name);
                self.handle_job_finish(&name, queue_next).await?;
            }

            Message::RunJob { name, one_shot } => {
                tracing::info!("run pending task {}", name);
                self.handle_run_job(&name, !one_shot).await?;
            }

            Message::Init => {
                let jobs = self.0.jobs.read().await;
                tracing::info!("initializing scheduler with required tasks");
                for (name, job_details) in jobs.iter() {
                    tracing::info!("task: {}", name);
                    let job_model = jobs::Entity::find()
                        .filter(jobs::Column::Name.eq(name.clone()))
                        .one(&self.0.db)
                        .await?
                        .unwrap();

                    let last_execution = job_model.last_execution;
                    let job_type = &job_details.job_type;
                    let time_now = chrono::offset::Utc::now();

                    // if no last execution, execute immediately
                    let next = match job_type {
                        JobType::Schedule(schedule) => match last_execution {
                            Some(t) => schedule.after(&t.and_utc()).next(),
                            None => Some(time_now),
                        }
                        .unwrap(),
                        JobType::Manual => {
                            tracing::info!("job is manual, skipping");
                            continue;
                        }
                    };

                    tracing::info!("time now: {}", time_now);
                    tracing::info!("next scheduled run: {}", next);

                    let time_until = if time_now >= next {
                        Duration::ZERO
                    } else {
                        (next - time_now).to_std()?
                    };

                    self.0
                        .task_queue
                        .push(
                            Message::RunJob {
                                name: job_model.name,
                                one_shot: false,
                            },
                            time_until,
                        )
                        .await;
                }
            }
        }

        Ok::<(), JobError>(())
    }

    async fn tick(&self) -> Result<ControlFlow<()>, JobError> {
        let cancellation_token = &self.0.cancellation_token;
        let task_queue = &self.0.task_queue;

        tokio::select! {
            _ = cancellation_token.cancelled() => {
                tracing::info!("ticking cancelled");
                Ok(ControlFlow::Break(()))
            },
            Some(message) = task_queue.pop() => {
                let msg = format!("{:?}", message);
                let span = tracing::span!(Level::INFO, "job", msg);
                self.handle_message(message).instrument(span).await?;
                Ok(ControlFlow::Continue(()))
            }
        }
    }
}
