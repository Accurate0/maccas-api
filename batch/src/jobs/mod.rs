use self::error::JobError;
use entity::jobs;
use sea_orm::DatabaseTransaction;
use sea_orm::{prelude::Uuid, ActiveModelTrait, EntityTrait, Set};
use std::fmt::Debug;
use std::sync::Arc;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

pub mod account_unlock;
pub mod activate_account;
pub mod categorise_offers;
pub mod create_account;
pub mod error;
pub mod job_scheduler;
pub mod recategorise_offers;
pub mod refresh;
pub mod save_images;

#[async_trait::async_trait]
pub trait Job: Send + Sync + Debug {
    fn name(&self) -> String;
    fn job_type(&self) -> JobType;
    async fn execute(
        &self,
        _context: &JobContext,
        _cancellation_token: CancellationToken,
    ) -> Result<(), JobError> {
        Ok(())
    }
    async fn post_execute(
        &self,
        _context: &JobContext,
        _cancellation_token: CancellationToken,
    ) -> Result<(), JobError> {
        Ok(())
    }
}

#[derive(Debug)]
pub struct RunningState {
    pub cancellation_token: CancellationToken,
    pub handle: JoinHandle<()>,
}

#[derive(Debug, Default)]
pub enum JobState {
    #[default]
    Stopped,
    Running(RunningState),
}

#[derive(Debug)]
pub enum JobType {
    Schedule(cron::Schedule),
    Manual,
}

#[derive(Debug)]
pub struct JobDetails {
    pub job: Arc<dyn Job>,
    pub state: JobState,
    pub enabled: bool,
    pub job_type: JobType,
}

#[derive(Debug, Clone, serde::Serialize)]
pub enum IntrospectedJobState {
    Stopped,
    Running,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct IntrospectedJobDetails {
    pub name: String,
    pub state: IntrospectedJobState,
}

impl JobDetails {
    pub fn new(job: Arc<dyn Job>, job_type: JobType, enabled: bool) -> Self {
        Self {
            job,
            job_type,
            state: Default::default(),
            enabled,
        }
    }
}

pub struct JobContext<'a> {
    database: &'a DatabaseTransaction,
    id: Uuid,
}

#[allow(unused)]
impl<'a> JobContext<'a> {
    pub fn new(database: &'a DatabaseTransaction, id: Uuid) -> Self {
        Self { database, id }
    }

    pub async fn get<T>(&self) -> Option<T>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        serde_json::from_value::<T>(
            jobs::Entity::find_by_id(self.id)
                .one(self.database)
                .await
                .map(|e| e.map(|m| m.context))
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
            context: Set(Some(serde_json::to_value(context)?)),
            ..Default::default()
        }
        .update(self.database)
        .await?;

        Ok(())
    }

    pub async fn reset(&self) -> Result<(), JobError> {
        jobs::ActiveModel {
            id: Set(self.id),
            context: Set(None),
            ..Default::default()
        }
        .update(self.database)
        .await?;

        Ok(())
    }
}
