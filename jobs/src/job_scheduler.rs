use crate::{error::JobError, Job, JobContext, JobDetails, JobState, RunningState};
use entity::jobs;
use sea_orm::{
    prelude::Uuid, sea_query::OnConflict, ActiveModelTrait, ColumnTrait, DatabaseConnection,
    EntityTrait, QueryFilter, Set,
};
use std::{sync::Arc, time::Duration};
use tokio::{sync::RwLock, task::JoinHandle};
use tokio_util::sync::CancellationToken;
use tracing::{Instrument, Level};

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

    pub async fn shutdown(&self) {
        let mut jobs = self.jobs.write().await;
        let mut cancellation_futures = Vec::new();

        for job_details in jobs.iter_mut() {
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
        self.cancellation_token.cancel()
    }

    // FIMXE: replace with messages and wake up based
    async fn tick(&self) -> Result<(), JobError> {
        let mut jobs = self.jobs.write().await;

        for job_details in jobs.iter_mut() {
            let name = job_details.job.name();
            let span = tracing::span!(Level::INFO, "tick", name);

            match async move {
                let job_model = jobs::Entity::find()
                    .filter(jobs::Column::Name.eq(name.clone()))
                    .one(&self.db)
                    .await?
                    .unwrap();

                let last_execution = job_model.last_execution;
                let schedule = &job_details.schedule;
                let time_now = chrono::offset::Utc::now();

                // if no last execution, execute immediately
                let next = match last_execution {
                    Some(t) => schedule.after(&t.and_utc()).next(),
                    None => Some(time_now),
                }
                .unwrap();

                if let JobState::Running(state) = &job_details.state {
                    if state.handle.is_finished() {
                        job_details.state = JobState::Stopped;
                        tracing::info!("job finished, transitioning state");
                        tracing::info!("next scheduled at {}", next);
                    }
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
                    let context = JobContext::new(self.db.clone(), job_model.id);
                    let span = tracing::span!(parent: None, Level::INFO, "job", name);

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

                Ok::<(), JobError>(())
            }
            .instrument(span)
            .await
            {
                Ok(_) => {}
                Err(e) => tracing::error!("error with job: {}", e),
            }
        }

        Ok(())
    }

    pub async fn run(&self) -> JoinHandle<()> {
        let cancellation_token = self.cancellation_token.clone();
        let scheduler = self.clone();

        tokio::spawn(async move {
            loop {
                if let Err(e) = scheduler.tick().await {
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
