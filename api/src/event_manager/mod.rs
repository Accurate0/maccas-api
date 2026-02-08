use api::Event;
use base::feature_flag::FeatureFlagClient;
use entity::events;
use entity::sea_orm_active_enums::{EventStatus, EventStatusEnum};
use futures::TryFutureExt;
use open_feature::EvaluationContext;
use sea_orm::prelude::Uuid;
use sea_orm::sea_query::Expr;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, Set,
    TransactionTrait, Unchanged,
};
use serde::{Deserialize, Serialize};
use state::TypeMap;
use std::{sync::Arc, time::Duration};
use thiserror::Error;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tracing::instrument;

mod handlers;
pub use handlers::S3BucketType;

#[derive(Error, Debug)]
pub enum EventManagerError {
    #[error("Serializer error has occurred: `{0}`")]
    Serializer(#[from] serde_json::Error),
    #[error("DelayQueue error has occurred: `{0}`")]
    DelayQueue(#[from] crate::queue::DelayQueueError),
    #[error("Database error has occurred: `{0}`")]
    Database(#[from] DbErr),
    #[error("Chrono out of range error has occurred: `{0}`")]
    OutOfRangeError(#[from] chrono::OutOfRangeError),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct QueuedEvent {
    pub(crate) evt: Event,
    pub(crate) id: i32,
    pub(crate) trace_id: String,
}

#[derive(Debug)]
struct EventManagerInner {
    db: DatabaseConnection,
    event_queue: crate::queue::DelayQueue<QueuedEvent>,
    state: TypeMap![Sync + Send],
}

#[derive(Debug)]
pub struct EventManager {
    semaphore: Arc<Semaphore>,
    inner: Arc<EventManagerInner>,
}

const EVENT_QUEUE_NAME: &str = "event_processing_queue";

impl EventManager {
    pub async fn new(
        db: DatabaseConnection,
        max_concurrency: usize,
    ) -> Result<Self, EventManagerError> {
        let connection_pool = db.get_postgres_connection_pool().clone();
        let event_queue =
            crate::queue::DelayQueue::new(connection_pool, EVENT_QUEUE_NAME.to_owned()).await?;

        Ok(Self {
            semaphore: Arc::new(Semaphore::new(max_concurrency)),
            inner: EventManagerInner {
                db,
                event_queue,
                state: Default::default(),
            }
            .into(),
        })
    }

    #[instrument(skip(self))]
    pub async fn archive(&self, message_id: i64) -> Result<bool, EventManagerError> {
        let completed_at = chrono::offset::Utc::now().naive_utc();
        events::ActiveModel {
            id: Unchanged(message_id as i32),
            is_completed: Set(true),
            completed_at: Set(Some(completed_at)),
            status: Set(EventStatus::Cancelled),
            error_message: Set(Some("cancelled by should_run".to_owned())),
            ..Default::default()
        }
        .update(&self.inner.db)
        .await?;

        self.inner
            .event_queue
            .archive(message_id)
            .map_err(|e| e.into())
            .await
    }

    pub fn db(&self) -> &DatabaseConnection {
        &self.inner.db
    }

    pub fn set_state<T>(&self, state: T)
    where
        T: Send + Sync + 'static,
    {
        self.inner.state.set(state);
    }

    pub fn try_get_state<T>(&self) -> Option<&T>
    where
        T: Send + Sync + 'static,
    {
        self.inner.state.try_get::<T>()
    }

    pub fn get_state<T>(&self) -> &T
    where
        T: Send + Sync + 'static,
    {
        self.inner.state.get::<T>()
    }

    #[instrument(skip(self))]
    pub async fn create_event(
        &self,
        evt: Event,
        delay: Duration,
        trace_id: String,
    ) -> Result<Uuid, EventManagerError> {
        let event_id = Uuid::new_v4();
        let should_be_completed_at = chrono::offset::Utc::now().naive_utc() + delay;

        let txn = self.inner.db.begin().await?;

        let event = events::ActiveModel {
            name: Set(evt.to_string()),
            event_id: Set(event_id),
            data: Set(serde_json::to_value(&evt)?),
            is_completed: Set(false),
            should_be_completed_at: Set(should_be_completed_at),
            trace_id: Set(Some(trace_id.to_owned())),
            status: Set(EventStatus::Pending),
            ..Default::default()
        }
        .insert(&txn)
        .await?;

        // mark other matching pending events as duplicate
        let duplicate_events = events::Entity::update_many()
            .filter(events::Column::Hash.eq(&event.hash))
            // ignore newly added event :)
            .filter(events::Column::EventId.ne(event_id))
            .filter(events::Column::Status.eq(EventStatus::Pending))
            .col_expr(
                events::Column::Status,
                Expr::val(EventStatus::Duplicate).as_enum(EventStatusEnum),
            )
            .exec(&txn)
            .await?;

        txn.commit().await?;

        tracing::info!("created event: {event:?}");
        tracing::info!(
            "marked {} events as duplicates",
            duplicate_events.rows_affected
        );

        self.inner
            .event_queue
            .push(
                QueuedEvent {
                    evt,
                    id: event.id,
                    trace_id,
                },
                delay,
            )
            .await?;

        Ok(event_id)
    }

    pub async fn should_run(&self, event_id: i32) -> bool {
        let Some(event) = events::Entity::find_by_id(event_id)
            .one(&self.inner.db)
            .await
            .ok()
            .flatten()
        else {
            return false;
        };

        let is_pending = event.status == EventStatus::Pending;

        let feature_flag_client = self.try_get_state::<FeatureFlagClient>();
        let is_allowed_by_ff = if let Some(feature_flag_client) = feature_flag_client {
            let evaluation_context =
                EvaluationContext::default().with_custom_field("event_name", event.name.clone());
            feature_flag_client
                .is_feature_enabled_with_context(
                    "maccas-api-task-control",
                    true,
                    evaluation_context,
                )
                .await
        } else {
            false
        };

        if !is_allowed_by_ff {
            tracing::warn!("event {} disabled by feature flag", event.name);
        }

        is_pending && is_allowed_by_ff
    }

    pub async fn acquire_permit(&self) -> OwnedSemaphorePermit {
        self.semaphore.clone().acquire_owned().await.unwrap()
    }

    pub fn process_events(&self) -> (JoinHandle<()>, CancellationToken) {
        let em = self.clone();
        let cancellation_token = CancellationToken::new();
        let cancellation_token_cloned = cancellation_token.clone();

        (
            tokio::spawn(async move {
                loop {
                    tokio::select! {
                        _ = cancellation_token.cancelled() => {
                            tracing::info!("handle cancelled");
                            break;
                        },

                        // FIXME: replace with dynamic events or something?
                        _ =  handlers::handle(em.clone()) => {}
                    }
                }
            }),
            cancellation_token_cloned,
        )
    }

    pub async fn set_event_completed(
        &self,
        id: i32,
        attempts: i32,
    ) -> Result<(), EventManagerError> {
        let completed_at = chrono::offset::Utc::now().naive_utc();
        events::ActiveModel {
            id: Unchanged(id),
            is_completed: Set(true),
            completed_at: Set(Some(completed_at)),
            attempts: Set(attempts),
            status: Set(EventStatus::Completed),
            ..Default::default()
        }
        .update(&self.inner.db)
        .await?;

        Ok(())
    }

    pub async fn set_event_running(&self, id: i32) -> Result<(), EventManagerError> {
        events::ActiveModel {
            id: Unchanged(id),
            status: Set(EventStatus::Running),
            ..Default::default()
        }
        .update(&self.inner.db)
        .await?;

        Ok(())
    }

    pub async fn set_event_completed_in_error(
        &self,
        id: i32,
        msg: &str,
        attempts: i32,
    ) -> Result<(), EventManagerError> {
        let completed_at = chrono::offset::Utc::now().naive_utc();

        events::ActiveModel {
            id: Unchanged(id),
            is_completed: Set(true),
            completed_at: Set(Some(completed_at)),
            error: Set(true),
            attempts: Set(attempts),
            error_message: Set(Some(msg.to_owned())),
            status: Set(EventStatus::Failed),
            ..Default::default()
        }
        .update(&self.inner.db)
        .await?;

        Ok(())
    }
}

impl Clone for EventManager {
    fn clone(&self) -> Self {
        Self {
            semaphore: self.semaphore.clone(),
            inner: self.inner.clone(),
        }
    }
}
