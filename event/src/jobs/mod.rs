use crate::event_manager::EventManager;

use self::error::JobError;
use entity::job_history;
use sea_orm::DatabaseTransaction;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use std::fmt::Debug;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

pub mod account_unlock;
pub mod activate_account;
pub mod activate_existing_account;
pub mod categorise_offers;
pub mod create_account;
pub mod error;
pub mod generate_recommendations;
pub mod job_scheduler;
pub mod recategorise_offers;
pub mod refresh;
pub mod save_images;

#[async_trait::async_trait]
pub trait Job: Send + Sync + Debug {
    fn name(&self) -> String;
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
pub struct JobDetails {
    pub job: Arc<dyn Job>,
}

impl JobDetails {
    pub fn new(job: Arc<dyn Job>) -> Self {
        Self { job }
    }
}

pub struct JobContext<'a> {
    database: &'a DatabaseTransaction,
    event_manager: EventManager,
    execution_id: i32,
}

#[allow(unused)]
impl<'a> JobContext<'a> {
    pub fn new(
        database: &'a DatabaseTransaction,
        execution_id: i32,
        event_manager: EventManager,
    ) -> Self {
        Self {
            database,
            execution_id,
            event_manager,
        }
    }

    pub async fn get<T>(&self) -> Option<T>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        serde_json::from_value::<T>(
            job_history::Entity::find_by_id(self.execution_id)
                .one(self.database)
                .await
                .map(|e| e.map(|m| m.context))
                .ok()
                .flatten()
                .flatten()
                .into(),
        )
        .ok()
    }

    pub async fn set<T>(&self, context: T) -> Result<(), JobError>
    where
        T: serde::Serialize,
    {
        job_history::ActiveModel {
            id: Set(self.execution_id),
            context: Set(Some(serde_json::to_value(context)?)),
            ..Default::default()
        }
        .update(self.database)
        .await?;

        Ok(())
    }

    pub async fn reset(&self) -> Result<(), JobError> {
        job_history::ActiveModel {
            id: Set(self.execution_id),
            context: Set(None),
            ..Default::default()
        }
        .update(self.database)
        .await?;

        Ok(())
    }
}
