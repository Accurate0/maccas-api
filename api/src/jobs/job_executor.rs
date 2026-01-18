use super::{Job, JobContext, JobDetails, error::JobError};
use crate::event_manager::EventManager;
use anyhow::Context;
use entity::jobs;
use futures::FutureExt;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set,
    TransactionTrait, Unchanged, prelude::Uuid, sea_query::OnConflict,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap, fmt::Debug, ops::ControlFlow, panic::AssertUnwindSafe, sync::Arc,
    time::Duration,
};
use tokio::{sync::RwLock, task::JoinHandle};
use tokio_util::sync::CancellationToken;
use tracing::{Instrument, Level, instrument};

const JOB_QUEUE_NAME: &str = "batch_job_queue";

// FIXME: confusing to have 2 params that mean opposite
#[derive(Clone, Serialize, Deserialize)]
pub(crate) enum JobMessage {
    RunJob { name: String },
}

impl Debug for JobMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RunJob { name, .. } => {
                write!(f, "RunPending: {}", name)
            }
        }
    }
}

pub struct JobSchedulerInner {
    cancellation_token: CancellationToken,
    event_manager: EventManager,
    jobs: Arc<RwLock<HashMap<String, JobDetails>>>,
    db: DatabaseConnection,
    task_queue: crate::queue::DelayQueue<JobMessage>,
}

#[derive(Clone)]
pub struct JobExecutor(Arc<JobSchedulerInner>);

impl JobExecutor {
    pub async fn new(
        db: DatabaseConnection,
        event_manager: EventManager,
        cancellation_token: CancellationToken,
    ) -> Result<Self, JobError> {
        let connection_pool = db.get_postgres_connection_pool().clone();
        let task_queue =
            crate::queue::DelayQueue::new(connection_pool, JOB_QUEUE_NAME.to_owned()).await?;

        Ok(JobExecutor(
            JobSchedulerInner {
                db,
                event_manager,
                cancellation_token,
                jobs: Default::default(),
                task_queue,
            }
            .into(),
        ))
    }

    pub async fn add<T>(&self, job: T) -> &Self
    where
        T: Job + 'static,
    {
        let mut jobs = self.0.jobs.write().await;
        jobs.insert(job.name(), JobDetails::new(Arc::new(job)));

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

    #[instrument(skip(self))]
    pub async fn run_job(&self, name: &str) -> Result<(), JobError> {
        let jobs = self.0.jobs.read().await;
        let job_to_run = jobs.get(name).context("can't find job with name")?;

        self.0
            .task_queue
            .push(
                JobMessage::RunJob {
                    name: job_to_run.job.name(),
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

    async fn handle_run_job(&self, name: &str) -> Result<(), JobError> {
        let mut jobs = self.0.jobs.write().await;

        let job_model = jobs::Entity::find()
            .filter(jobs::Column::Name.eq(name))
            .one(&self.0.db)
            .await?
            .unwrap();

        let job_details = jobs.get_mut(name).unwrap();

        let cancellation_token_cloned = self.0.cancellation_token.child_token();

        let _handle = self
            .handle_start_new_job(&job_model, job_details, cancellation_token_cloned)
            .await?;

        Ok(())
    }

    async fn handle_start_new_job(
        &self,
        job_model: &entity::jobs::Model,
        job_details: &JobDetails,
        cancellation_token: CancellationToken,
    ) -> Result<JoinHandle<()>, JobError> {
        let job = job_details.job.clone();
        let db = self.0.db.clone();
        let job_id = job_model.id;
        let task_name = job_model.name.to_string();
        let span = tracing::span!(
            parent: None,
            Level::INFO,
            "job",
            job_name = task_name,
            "otel.name" = format!("job::{}", task_name)
        );

        let execution_id = entity::job_history::ActiveModel {
            job_name: Set(task_name.clone()),
            ..Default::default()
        }
        .insert(&db)
        .await?
        .id;

        let event_manager_cloned = self.0.event_manager.clone();
        let fut = async move {
            let result = async {
                let txn = db.begin().await?;
                let context =
                    JobContext::new(&txn, &db, execution_id, event_manager_cloned.clone());

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
                    let post_context =
                        JobContext::new(&txn, &db, execution_id, event_manager_cloned.clone());

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

        let handle = tokio::spawn(fut.then(|r| async move {
            if let Err(e) = r {
                tracing::error!("error in job completion: {e}")
            }
        }));

        Ok(handle)
    }

    async fn handle_message(&self, message: JobMessage) -> Result<(), JobError> {
        match message {
            JobMessage::RunJob { name } => {
                tracing::info!("run pending task {}", name);
                self.handle_run_job(&name).await?;
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
