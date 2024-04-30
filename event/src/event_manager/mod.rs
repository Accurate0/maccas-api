use base::delay_queue::DelayQueue;
use entity::events;
use entity::sea_orm_active_enums::EventStatus;
use event::Event;
use sea_orm::prelude::Uuid;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, DbErr, EntityTrait, QueryFilter,
    Set, Unchanged,
};
use serde::Serialize;
use state::TypeMap;
use std::{sync::Arc, time::Duration};
use thiserror::Error;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tracing::instrument;

mod handlers;

#[derive(Error, Debug)]
pub enum EventManagerError {
    #[error("Serializer error has ocurred: `{0}`")]
    Serializer(#[from] serde_json::Error),
    #[error("Database error has ocurred: `{0}`")]
    Database(#[from] DbErr),
    #[error("Chrono out of range error has ocurred: `{0}`")]
    OutOfRangeError(#[from] chrono::OutOfRangeError),
    #[error("An unknown error ocurred: `{0}`")]
    UnknownError(#[from] anyhow::Error),
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct QueuedEvent {
    pub(crate) evt: Event,
    pub(crate) id: i32,
    pub(crate) trace_id: String,
}

struct EventManagerInner {
    db: DatabaseConnection,
    event_queue: DelayQueue<QueuedEvent>,
    state: TypeMap![Sync + Send],
}

pub struct EventManager {
    inner: Arc<EventManagerInner>,
}

impl EventManager {
    pub fn new(db: DatabaseConnection) -> Self {
        Self {
            inner: EventManagerInner {
                db,
                event_queue: Default::default(),
                state: Default::default(),
            }
            .into(),
        }
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

    pub fn get_state<T>(&self) -> &T
    where
        T: Send + Sync + 'static,
    {
        self.inner.state.get::<T>()
    }

    pub async fn create_event(
        &self,
        evt: Event,
        delay: Duration,
        trace_id: String,
    ) -> Result<Uuid, EventManagerError> {
        let event_id = Uuid::new_v4();
        let should_be_completed_at = chrono::offset::Utc::now().naive_utc() + delay;

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
        .insert(&self.inner.db)
        .await?;

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
            .await;

        Ok(event_id)
    }

    #[instrument(skip(self))]
    pub async fn reload_incomplete_events(&self) -> Result<(), EventManagerError> {
        let incomplete_events = events::Entity::find()
            .filter(Condition::all().add(events::Column::IsCompleted.eq(false)))
            .all(&self.inner.db)
            .await?;
        let now = chrono::offset::Utc::now().naive_utc();

        for event in &incomplete_events {
            tracing::info!("reloading incomplete event: {}", event.event_id);
            let reload_event = async move {
                let delay = event.should_be_completed_at - now;
                tracing::info!("delay for this event is: {}", delay);

                // FIXME: throttle events if too many to avoid overload / ratelimit

                self.inner
                    .event_queue
                    .push(
                        QueuedEvent {
                            evt: serde_json::from_value(event.data.clone())?,
                            id: event.id,
                            trace_id: event.trace_id.to_owned().unwrap_or_default(),
                        },
                        // run immediately if its past the should be completed at
                        delay.to_std().unwrap_or(Duration::ZERO),
                    )
                    .await;

                Ok::<(), EventManagerError>(())
            };

            match reload_event.await {
                Ok(_) => {}
                Err(e) => {
                    tracing::error!("error while reloading event: {}", e);
                }
            }
        }

        Ok(())
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
            inner: self.inner.clone(),
        }
    }
}
