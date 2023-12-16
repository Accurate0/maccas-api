use crate::error::JobError;
use chrono::Utc;
use entity::jobs;
use sea_orm::{
    prelude::Uuid, sea_query::OnConflict, ActiveModelTrait, ColumnTrait, DatabaseConnection,
    EntityTrait, QueryFilter, Set,
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
    async fn execute(&self, _context: &JobContext, _cancellation_token: CancellationToken) {}
    async fn cleanup(&self, _context: &JobContext) {}
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
    schedule: cron::Schedule,
}

impl JobDetails {
    pub fn new(job: Arc<dyn Job>, schedule: cron::Schedule) -> Self {
        Self {
            job,
            schedule,
            state: Default::default(),
        }
    }
}

pub struct JobContext {
    database: DatabaseConnection,
    id: Uuid,
}

#[allow(unused)]
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

    pub async fn add<T>(&self, job: T, schedule: cron::Schedule) -> &Self
    where
        T: Job + 'static,
    {
        let mut jobs = self.jobs.write().await;
        jobs.push(JobDetails::new(Arc::new(job), schedule));

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

    #[allow(unused)]
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
        let mut jobs = self.jobs.write().await;

        for job_details in jobs.iter_mut() {
            let name = job_details.job.name();

            if let JobState::Running(state) = &job_details.state {
                if state.handle.is_finished() {
                    job_details.state = JobState::NotStarted;
                    tracing::info!("[{name}] job finished, transitioning state");
                }
            };

            let cancellation_token = CancellationToken::new();
            let cancellation_token_cloned = cancellation_token.clone();
            let job = job_details.job.clone();

            let job_model = jobs::Entity::find()
                .filter(jobs::Column::Name.eq(name.clone()))
                .one(&self.db)
                .await?
                .unwrap();

            let last_execution = job_model.last_execution;
            let schedule = &job_details.schedule;

            let next = match last_execution {
                Some(t) => schedule.after(&t.and_utc()).next(),
                None => schedule.upcoming(Utc).next(),
            }
            .unwrap();

            let time_now = chrono::offset::Utc::now();

            tracing::trace!("[{name}] time now: {}", time_now);
            tracing::trace!("[{name}] next scheduled run: {}", next);

            let running = match job_details.state {
                JobState::NotStarted => false,
                JobState::Running(_) => true,
            };

            if next <= time_now && !running {
                tracing::info!("[{name}] triggered job");
                let context = JobContext::new(self.db.clone(), job_model.id);
                let span = tracing::span!(Level::INFO, "job", name);

                let handle = tokio::spawn(
                    async move {
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
                    id: Set(job_model.id),
                    name: Set(name),
                    last_execution: Set(Some(time_now.naive_utc())),
                    ..Default::default()
                }
                .update(&self.db)
                .await?;
            };
        }

        Ok(())
    }
}
