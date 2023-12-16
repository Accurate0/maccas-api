use crate::error::JobError;
use entity::jobs;
use sea_orm::{
    prelude::Uuid, sea_query::OnConflict, ActiveModelTrait, ColumnTrait, DatabaseConnection,
    EntityTrait, QueryFilter, Set, Unchanged,
};

use std::sync::Arc;

use std::fmt::Debug;
use tokio::{sync::RwLock, task::JoinHandle};
use tokio_util::sync::CancellationToken;
use tracing::{Instrument, Level};

pub mod refresh;

#[async_trait::async_trait]
pub trait Job: Send + Sync + Debug {
    fn name(&self) -> String;
    async fn prepare(&self) {}
    async fn execute(&self, _context: &JobContext, _cancellation_token: CancellationToken) {}
    async fn cleanup(&self, _context: &JobContext) {}
}

#[derive(Debug)]
pub enum JobType {
    Continuous,
    // Scheduled,
}

#[derive(Debug)]
pub struct RunningState {
    cancellation_token: CancellationToken,
    handle: JoinHandle<()>,
}

#[derive(Debug, Default)]
pub enum JobState {
    #[default]
    NotStarted,
    Running(RunningState),
}

#[derive(Debug)]
struct JobDetails {
    job: Arc<dyn Job>,
    state: JobState,
    r#type: JobType,
}

impl JobDetails {
    pub fn new(job: Arc<dyn Job>, r#type: JobType) -> Self {
        Self {
            job,
            r#type,
            state: Default::default(),
        }
    }
}

pub struct JobContext {
    database: DatabaseConnection,
    id: Uuid,
}

impl JobContext {
    pub fn new(database: DatabaseConnection, id: Uuid) -> Self {
        Self { database, id }
    }

    pub async fn get<T>(&self) -> Option<T>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        serde_json::from_value::<T>(
            jobs::Entity::find_by_id(self.id)
                .one(&self.database)
                .await
                .map(|e| e.map(|m| m.resume_context))
                .ok()
                .unwrap_or(None)
                .unwrap_or(None)
                .into(),
        )
        .ok()
    }

    pub async fn set<T>(&self, context: T) -> Result<(), JobError>
    where
        T: serde::Serialize,
    {
        jobs::ActiveModel {
            id: Set(self.id),
            resume_context: Set(Some(serde_json::to_value(context)?)),
            ..Default::default()
        }
        .update(&self.database)
        .await?;

        Ok(())
    }

    pub async fn reset(&self) -> Result<(), JobError> {
        jobs::ActiveModel {
            id: Set(self.id),
            resume_context: Set(None),
            ..Default::default()
        }
        .update(&self.database)
        .await?;

        Ok(())
    }
}

#[derive(Default, Clone)]
pub struct JobScheduler {
    cancellation_token: CancellationToken,
    jobs: Arc<RwLock<Vec<JobDetails>>>,
    db: DatabaseConnection,
}

impl JobScheduler {
    pub fn new(db: DatabaseConnection) -> Self {
        Self {
            db,
            ..Default::default()
        }
    }

    pub async fn add<T>(&self, job: T, r#type: JobType) -> &Self
    where
        T: Job + 'static,
    {
        let mut jobs = self.jobs.write().await;
        jobs.push(JobDetails::new(Arc::new(job), r#type));

        self
    }

    pub async fn init(&self) -> Result<(), JobError> {
        let jobs = self.jobs.read().await;
        for job_details in jobs.iter() {
            let name = job_details.job.name();
            let id = Uuid::new_v4();

            let job_model = jobs::ActiveModel {
                id: Set(id),
                name: Set(name),
                ..Default::default()
            };

            jobs::Entity::insert(job_model)
                .on_conflict(
                    OnConflict::columns([jobs::Column::Name])
                        .do_nothing()
                        .to_owned(),
                )
                .exec_without_returning(&self.db)
                .await?;
        }

        Ok(())
    }

    pub async fn start(&self) -> Result<(), JobError> {
        let mut jobs = self.jobs.write().await;
        for job_details in jobs.iter_mut() {
            match job_details.r#type {
                JobType::Continuous => {
                    let name = job_details.job.name();
                    let span = tracing::span!(Level::INFO, "job", name);

                    let cancellation_token = CancellationToken::new();
                    let cancellation_token_cloned = cancellation_token.clone();
                    let job = job_details.job.clone();

                    let job_model = jobs::Entity::find()
                        .filter(jobs::Column::Name.eq(name.clone()))
                        .one(&self.db)
                        .await?
                        .unwrap();

                    let context = JobContext::new(self.db.clone(), job_model.id);

                    let handle = tokio::spawn(
                        async move {
                            job.prepare().await;
                            job.execute(&context, cancellation_token_cloned).await;
                            job.cleanup(&context).await;
                        }
                        .instrument(span),
                    );

                    job_details.state = JobState::Running(RunningState {
                        cancellation_token,
                        handle,
                    });

                    jobs::ActiveModel {
                        id: Unchanged(job_model.id),
                        name: Set(name),
                        last_execution: Set(Some(chrono::offset::Utc::now().naive_utc())),
                        ..Default::default()
                    }
                    .update(&self.db)
                    .await?;
                }
            }
        }

        Ok(())
    }

    pub async fn shutdown(&self) {
        let mut jobs = self.jobs.write().await;
        let mut cancellation_futures = Vec::new();

        for job_details in jobs.iter_mut() {
            match job_details.state {
                JobState::NotStarted => todo!(),
                JobState::Running(ref mut state) => {
                    state.cancellation_token.cancel();
                    cancellation_futures.push(&mut state.handle)
                }
            }
        }

        futures::future::join_all(cancellation_futures).await;
        self.cancellation_token.cancel()
    }

    pub async fn tick(&self) -> Result<(), JobError> {
        let future = async move {
            let jobs = self.jobs.read().await;

            for job_details in jobs.iter() {
                match job_details.r#type {
                    JobType::Continuous => match &job_details.state {
                        JobState::NotStarted => todo!(),
                        JobState::Running(state) => tracing::info!(
                            "job: {} is_finished: {} is_cancelled: {}",
                            job_details.job.name(),
                            state.handle.is_finished(),
                            state.cancellation_token.is_cancelled()
                        ),
                    },
                }

                // find and execute cron jobs
                // check health on continuous jobs
            }
        };

        tokio::select! {
            _ = self.cancellation_token.cancelled() => {
                tracing::warn!("cancellation requested for scheduler");
                Err(JobError::SchedulerCancelled)
            }
            _ = future => {
                Ok(())
            }
        }
    }
}

// Tracing span with job uuid
// Place into sql table with uuid, start, finish, error
