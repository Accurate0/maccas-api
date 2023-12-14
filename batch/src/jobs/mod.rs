use crate::error::JobError;
use entity::jobs;
use sea_orm::{
    prelude::Uuid, sea_query::OnConflict, ActiveModelTrait, ColumnTrait, DatabaseConnection,
    EntityTrait, QueryFilter, Set, Unchanged,
};
use std::fmt::Debug;
use std::sync::Arc;
use tokio::{sync::RwLock, task::JoinHandle};
use tokio_util::sync::CancellationToken;
use tracing::{Instrument, Level};

pub mod refresh;

#[async_trait::async_trait]
pub trait Job: Send + Sync + Debug {
    fn name(&self) -> String;
    async fn prepare(&self) {}
    async fn execute(&self, _cancellation_token: CancellationToken) {}
    async fn cleanup(&self) {}
}

#[derive(Debug)]
pub enum JobType {
    Continuous,
    // Scheduled,
}

#[derive(Debug, Default)]
pub enum JobState {
    #[default]
    NotStarted,
    Running {
        cancellation_token: CancellationToken,
        handle: JoinHandle<()>,
    },
}

#[derive(Debug)]
struct JobDetails {
    job: &'static dyn Job,
    state: JobState,
    r#type: JobType,
}

impl JobDetails {
    pub fn new(job: &'static dyn Job, r#type: JobType) -> Self {
        Self {
            job,
            r#type,
            state: Default::default(),
        }
    }
}

#[derive(Default)]
pub struct JobScheduler {
    jobs: Vec<Arc<RwLock<JobDetails>>>,
    db: DatabaseConnection,
}

impl JobScheduler {
    pub fn new(db: DatabaseConnection) -> Self {
        Self {
            db,
            ..Default::default()
        }
    }

    pub fn add(&mut self, job: &'static dyn Job, r#type: JobType) -> &mut Self {
        self.jobs
            .push(Arc::new(RwLock::new(JobDetails::new(job, r#type))));

        self
    }

    pub async fn init(&self) -> Result<(), JobError> {
        for job_details in &self.jobs {
            let name = job_details.read().await.job.name();
            let id = Uuid::new_v4();

            let job_model = jobs::ActiveModel {
                id: Set(id),
                name: Set(name),
                last_run: Unchanged(None),
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

    pub async fn start(&mut self) -> Result<(), JobError> {
        for job_details in &self.jobs {
            let mut job_details_lock = job_details.write().await;
            match job_details_lock.r#type {
                JobType::Continuous => {
                    let name = job_details_lock.job.name();
                    let span = tracing::span!(Level::INFO, "job", name);

                    let cancellation_token = CancellationToken::new();
                    let cancellation_token_cloned = cancellation_token.clone();

                    let job_details_cloned = job_details.clone();

                    let handle = tokio::spawn(
                        async move {
                            job_details_cloned.read().await.job.prepare().await;
                            job_details_cloned
                                .read()
                                .await
                                .job
                                .execute(cancellation_token_cloned)
                                .await;
                            job_details_cloned.read().await.job.cleanup().await;
                        }
                        .instrument(span),
                    );

                    job_details_lock.state = JobState::Running {
                        cancellation_token,
                        handle,
                    };

                    let job = jobs::Entity::find()
                        .filter(jobs::Column::Name.eq(name.clone()))
                        .one(&self.db)
                        .await?
                        .unwrap();

                    jobs::ActiveModel {
                        id: Unchanged(job.id),
                        name: Set(name),
                        last_run: Set(Some(chrono::offset::Utc::now().naive_utc())),
                    }
                    .update(&self.db)
                    .await?;

                    // FIXME: can't ever regain the write lock after the task has started, it must be stopped
                    // it holds a read lock forever while executing...
                }
            }
        }

        Ok(())
    }

    pub async fn tick(&self) {
        for job_details in &self.jobs {
            let job_details = job_details.read().await;
            match job_details.r#type {
                JobType::Continuous => match &job_details.state {
                    JobState::NotStarted => todo!(),
                    JobState::Running {
                        cancellation_token,
                        handle,
                    } => tracing::info!(
                        "job: {} is_finished: {} is_cancelled: {}",
                        job_details.job.name(),
                        handle.is_finished(),
                        cancellation_token.is_cancelled()
                    ),
                },
            }

            // find and execute cron jobs
            // check health on continuous jobs
        }
    }
}

// Tracing span with job uuid
// Place into sql table with uuid, start, finish, error
