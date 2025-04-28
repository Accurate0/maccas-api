use super::{error::JobError, IntrospectedJobDetails, Job, JobContext, JobDetails, JobType};
use anyhow::Context;
use base::delay_queue::IntrospectionResult;
use entity::jobs;
use futures::FutureExt;
use sea_orm::{
    prelude::Uuid, sea_query::OnConflict, ActiveModelTrait, ColumnTrait, DatabaseConnection,
    EntityTrait, QueryFilter, Set, TransactionTrait, Unchanged,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap, fmt::Debug, ops::ControlFlow, panic::AssertUnwindSafe, sync::Arc,
    time::Duration,
};
use tokio::{sync::RwLock, task::JoinHandle};
use tokio_util::sync::CancellationToken;
use tracing::{Instrument, Level};

const JOB_QUEUE_NAME: &str = "batch_job_queue";

// FIXME: confusing to have 2 params that mean opposite
#[derive(Clone, Serialize, Deserialize)]
pub(crate) enum JobMessage {
    JobFinished { name: String, queue_next: bool },
    RunJob { name: String, one_shot: bool },
}

impl Debug for JobMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::JobFinished { name, .. } => {
                write!(f, "JobFinished: {}", name)
            }
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
    task_queue: delayqueue::DelayQueue<JobMessage>,
}

#[derive(Clone)]
pub struct JobScheduler(Arc<JobSchedulerInner>);

impl JobScheduler {
    pub async fn new(db: DatabaseConnection) -> Result<Self, JobError> {
        let connection_pool = db.get_postgres_connection_pool().clone();
        let task_queue =
            delayqueue::DelayQueue::new(connection_pool, JOB_QUEUE_NAME.to_owned()).await?;

        Ok(JobScheduler(
            JobSchedulerInner {
                db,
                cancellation_token: Default::default(),
                jobs: Default::default(),
                task_queue,
            }
            .into(),
        ))
    }

    pub async fn introspect(
        &self,
    ) -> (
        Vec<IntrospectedJobDetails>,
        Vec<IntrospectionResult<JobMessage>>,
    ) {
        (
            self.0
                .jobs
                .read()
                .await
                .values()
                .map(|j| IntrospectedJobDetails {
                    name: j.job.name(),
                    state: super::IntrospectedJobState::Stopped,
                })
                .collect(),
            // FIXME: FIXME
            vec![],
        )
    }

    pub fn db(&self) -> &DatabaseConnection {
        &self.0.db
    }

    pub async fn add<T>(&self, job: T, enabled: bool) -> &Self
    where
        T: Job + 'static,
    {
        let mut jobs = self.0.jobs.write().await;
        let execution_type = job.job_type();
        jobs.insert(
            job.name(),
            JobDetails::new(Arc::new(job), execution_type, enabled),
        );

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

        self.init_jobs().await?;

        Ok(())
    }

    pub async fn run_job_one_shot(&self, name: &str) -> Result<(), JobError> {
        let jobs = self.0.jobs.read().await;
        let job_to_run = jobs.get(name).context("can't find job with name")?;

        self.0
            .task_queue
            .push(
                JobMessage::RunJob {
                    name: job_to_run.job.name(),
                    one_shot: true,
                },
                Duration::ZERO,
            )
            .await?;

        Ok(())
    }

    pub async fn shutdown(&self) {
        let mut _jobs = self.0.jobs.write().await;
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
                    // FIXME: if this is not sent, the job is not marked done
                    // we should deal with this
                    self.0
                        .task_queue
                        .push(
                            JobMessage::RunJob {
                                name: job_model.name,
                                one_shot: false,
                            },
                            time_until,
                        )
                        .await?;
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

        let cancellation_token_cloned = self.0.cancellation_token.child_token();

        let _handle = self
            .handle_start_new_job(
                &job_model,
                job_details,
                queue_next,
                cancellation_token_cloned,
            )
            .await?;

        Ok(())
    }

    async fn handle_start_new_job(
        &self,
        job_model: &entity::jobs::Model,
        job_details: &JobDetails,
        queue_next: bool,
        cancellation_token: CancellationToken,
    ) -> Result<JoinHandle<()>, JobError> {
        let job = job_details.job.clone();
        let db = self.0.db.clone();
        let job_id = job_model.id;
        let task_name = job_model.name.to_string();
        let span = tracing::span!(parent: None, Level::INFO, "job", job_name = task_name, "otel.name" = format!("job::{}", task_name));
        let queue = self.0.task_queue.clone();

        let execution_id = entity::job_history::ActiveModel {
            job_name: Set(task_name.clone()),
            ..Default::default()
        }
        .insert(&db)
        .await?
        .id;

        let fut = async move {
            let result = async {
                let txn = db.begin().await?;
                let context = JobContext::new(&txn, execution_id);

                let cancellation_token = cancellation_token.child_token();
                tracing::info!("triggered job {}", job.name());
                let result = AssertUnwindSafe(job.execute(&context, cancellation_token))
                    .catch_unwind()
                    .await;

                txn.commit().await?;

                Ok::<_, JobError>(result)
            }
            .instrument(tracing::span!(
                Level::INFO,
                "job::execute",
                "otel.name" = format!("job::{}::execute", task_name),
                "execution_id" = execution_id
            ))
            .await?;

            let job_error = match result {
                Ok(r) => match r {
                    Ok(_) => None,
                    Err(e) => Some(e.to_string()),
                },
                Err(e) => Some(format!("panic: {e:?}")),
            };

            if job_error.is_none() {
                let post_result = async {
                    let txn = db.begin().await?;
                    let post_context = JobContext::new(&txn, execution_id);

                    let cancellation_token = cancellation_token.child_token();
                    let result =
                        AssertUnwindSafe(job.post_execute(&post_context, cancellation_token))
                            .catch_unwind()
                            .await;

                    txn.commit().await?;

                    Ok::<_, JobError>(result)
                }
                .instrument(tracing::span!(
                    Level::INFO,
                    "job::post::execute",
                    "otel.name" = format!("job::{}::post::execute", task_name),
                    "execution_id" = execution_id
                ))
                .await?;

                let post_error = match post_result {
                    Ok(r) => match r {
                        Ok(_) => None,
                        Err(e) => Some(e.to_string()),
                    },
                    Err(e) => Some(format!("panic: {e:?}")),
                };

                if let Some(e) = post_error {
                    tracing::error!("error in post execute task: {e}");
                }
            } else {
                tracing::warn!("skipping post_execute as execute failed");
            }

            async {
                let time_now = chrono::offset::Utc::now();

                entity::job_history::ActiveModel {
                    id: Unchanged(execution_id),
                    error: Set(job_error.is_some()),
                    error_message: Set(job_error.clone()),
                    completed_at: Set(Some(time_now.naive_utc())),
                    ..Default::default()
                }
                .update(&db)
                .await?;

                jobs::ActiveModel {
                    id: Set(job_id),
                    name: Set(task_name.to_string()),
                    last_execution: Set(Some(time_now.naive_utc())),
                    ..Default::default()
                }
                .update(&db)
                .await?;

                Ok::<_, JobError>(())
            }
            .instrument(tracing::span!(
                Level::INFO,
                "job::complete",
                "otel.name" = format!("job::{}::complete", task_name),
                "execution_id" = execution_id
            ))
            .await?;

            Ok::<(), JobError>(())
        }
        .instrument(span);

        let task_name = job_model.name.to_string();

        let handle = tokio::spawn(
            fut.then(|r| async move {
                if let Err(e) = r {
                    tracing::error!("error in job completion: {e}")
                }
            })
            .then(move |_| async move {
                queue
                    .push(
                        JobMessage::JobFinished {
                            name: task_name,
                            queue_next,
                        },
                        Duration::ZERO,
                    )
                    .await?;
                Ok::<(), JobError>(())
            })
            .then(|r| async {
                if let Err(e) = r {
                    tracing::error!("error in finishing job: {e}")
                }
            }),
        );

        Ok(handle)
    }

    async fn init_jobs(&self) -> Result<(), JobError> {
        let jobs = self.0.jobs.read().await;
        tracing::info!("initializing scheduler with required tasks");
        for (name, job_details) in jobs.iter() {
            tracing::info!("task: {}", name);

            if !job_details.enabled {
                tracing::info!("skipping task");
                continue;
            }

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
                    JobMessage::RunJob {
                        name: job_model.name,
                        one_shot: false,
                    },
                    time_until,
                )
                .await?;
        }

        Ok::<(), JobError>(())
    }

    async fn handle_message(&self, message: JobMessage) -> Result<(), JobError> {
        match message {
            JobMessage::JobFinished { name, queue_next } => {
                tracing::info!("received message for job finished: {}", name);
                self.handle_job_finish(&name, queue_next).await?;
            }

            JobMessage::RunJob { name, one_shot } => {
                tracing::info!("run pending task {}", name);
                self.handle_run_job(&name, !one_shot).await?;
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
            message = task_queue.read(Duration::from_secs(300)) => {
                match message {
                    Ok(Some(message)) => {
                        let message_string = format!("{:?}", message.message);
                        let span = tracing::span!(Level::INFO, "job", message = message_string, message_id = message.msg_id);
                        self.handle_message(message.message).instrument(span).await?;
                        self.0.task_queue.archive(message.msg_id).await?;
                        Ok(ControlFlow::Continue(()))
                    },
                    Err(e) => {
                        tracing::error!("error reading message: {e}");
                        Ok(ControlFlow::Continue(()))
                    },
                    Ok(None) => {
                        tracing::trace!("no message found");
                        Ok(ControlFlow::Continue(()))
                    },
                }
            }
        }
    }
}
