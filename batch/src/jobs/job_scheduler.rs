use super::{error::JobError, Job, JobContext, JobDetails, JobState, JobType, RunningState};
use entity::jobs;
use sea_orm::{
    prelude::Uuid, sea_query::OnConflict, ActiveModelTrait, ColumnTrait, DatabaseConnection,
    EntityTrait, QueryFilter, Set,
};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::{
    sync::{
        mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
        Mutex, RwLock,
    },
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;
use tracing::{Instrument, Level};

enum Message {
    JobFinished { name: String },
    Tick,
}

struct Queue {
    pub tx: UnboundedSender<Message>,
    pub rx: Mutex<UnboundedReceiver<Message>>,
}

pub struct JobSchedulerInner {
    cancellation_token: CancellationToken,
    jobs: Arc<RwLock<HashMap<String, JobDetails>>>,
    db: DatabaseConnection,
    task_queue: Queue,
}

#[derive(Clone)]
pub struct JobScheduler(Arc<JobSchedulerInner>);

impl JobScheduler {
    pub fn new(db: DatabaseConnection) -> Self {
        let channel = unbounded_channel::<Message>();

        JobScheduler(
            JobSchedulerInner {
                db,
                cancellation_token: Default::default(),
                jobs: Default::default(),
                task_queue: Queue {
                    tx: channel.0,
                    rx: Mutex::new(channel.1),
                },
            }
            .into(),
        )
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

    pub async fn tick(&self) -> Result<(), JobError> {
        while let Some(message) = self.0.task_queue.rx.lock().await.recv().await {
            match message {
                Message::JobFinished { name } => {
                    tracing::info!("received message for job finished: {}", name);
                    let mut jobs = self.0.jobs.write().await;
                    match jobs.get_mut(&name) {
                        Some(job_details) => job_details.state = JobState::Stopped,
                        None => tracing::error!(
                            "could not find job by name for state to set state to finished: {}",
                            name
                        ),
                    }
                }

                Message::Tick => {
                    let mut jobs = self.0.jobs.write().await;

                    for (name, job_details) in jobs.iter_mut() {
                        let span = tracing::span!(Level::INFO, "tick", name);

                        match async move {
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
                            };

                            let cancellation_token = CancellationToken::new();
                            let cancellation_token_cloned = cancellation_token.clone();
                            let job = job_details.job.clone();

                            tracing::trace!("time now: {}", time_now);
                            tracing::trace!("next scheduled run: {}", next);

                            let running = match job_details.state {
                                JobState::Stopped => false,
                                JobState::Running(_) => true,
                            };

                            if next <= time_now && !running {
                                tracing::info!("triggered job");
                                let context = JobContext::new(self.0.db.clone(), job_model.id);
                                let span = tracing::span!(parent: None, Level::INFO, "job", name);
                                let end_channel = self.0.task_queue.tx.clone();
                                let task_name = name.clone();

                                let handle = tokio::spawn(
                                    async move {
                                        job.execute(&context, cancellation_token_cloned).await;
                                        job.cleanup(&context).await;
                                        if let Err(e) = end_channel
                                            .send(Message::JobFinished { name: task_name })
                                        {
                                            tracing::error!("error sending finished message: {}", e)
                                        };
                                    }
                                    .instrument(span),
                                );

                                job_details.state = JobState::Running(RunningState {
                                    cancellation_token,
                                    handle,
                                });

                                jobs::ActiveModel {
                                    id: Set(job_model.id),
                                    name: Set(name.clone()),
                                    last_execution: Set(Some(time_now.naive_utc())),
                                    ..Default::default()
                                }
                                .update(&self.0.db)
                                .await?;
                            };

                            Ok::<(), JobError>(())
                        }
                        .instrument(span)
                        .await
                        {
                            Ok(_) => {}
                            Err(e) => tracing::error!("error with job: {}", e),
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub async fn run(&self) -> JoinHandle<()> {
        let cancellation_token = self.0.cancellation_token.clone();
        let sender = self.0.task_queue.tx.clone();

        tokio::spawn(async move {
            loop {
                if let Err(e) = sender.send(Message::Tick) {
                    tracing::error!("error during tick: {}", e);
                };

                if cancellation_token.is_cancelled() {
                    tracing::warn!("scheduler cancellation received");
                    break;
                }

                tokio::time::sleep(Duration::from_millis(500)).await
            }
        })
    }
}
