use base::delay_queue::DelayQueue;
use entity::events;
use event::Event;
use sea_orm::prelude::Uuid;
use sea_orm::sea_query::Expr;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, DbErr, EntityTrait, QueryFilter,
    Set,
};
use std::{sync::Arc, time::Duration};
use thiserror::Error;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

mod handlers;

#[derive(Error, Debug)]
pub enum EventManagerError {
    #[error("Serializer error has ocurred: `{0}`")]
    Serializer(#[from] serde_json::Error),
    #[error("Database error has ocurred: `{0}`")]
    Database(#[from] DbErr),
    #[error("Chrono out of range error has ocurred: `{0}`")]
    OutOfRangeError(#[from] chrono::OutOfRangeError),
}

#[derive(Debug)]
struct QueuedEvent {
    evt: Event,
    id: i32,
}

struct EventManagerInner {
    db: DatabaseConnection,
    event_queue: DelayQueue<QueuedEvent>,
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
            }
            .into(),
        }
    }

    pub async fn create_event(
        &self,
        evt: Event,
        delay: Duration,
    ) -> Result<Uuid, EventManagerError> {
        let event_id = Uuid::new_v4();
        let should_be_completed_at = chrono::offset::Utc::now().naive_utc() + delay;

        let event = events::ActiveModel {
            name: Set(evt.to_string()),
            event_id: Set(event_id),
            data: Set(serde_json::to_value(&evt)?),
            is_completed: Set(false),
            should_be_completed_at: Set(should_be_completed_at),
            ..Default::default()
        }
        .insert(&self.inner.db)
        .await?;

        self.inner
            .event_queue
            .push(QueuedEvent { evt, id: event.id }, delay)
            .await;

        Ok(event_id)
    }

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

                self.inner
                    .event_queue
                    .push(
                        QueuedEvent {
                            evt: serde_json::from_value(event.data.clone())?,
                            id: event.id,
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
                        _ =  handlers::handle(em.clone()) => {}
                    }
                }
            }),
            cancellation_token_cloned,
        )
    }

    pub async fn set_retry_attempts(
        &self,
        id: i32,
        attempts: i32,
    ) -> Result<(), EventManagerError> {
        events::Entity::update_many()
            .col_expr(events::Column::Attempts, Expr::value(attempts))
            .filter(events::Column::Id.eq(id))
            .exec(&self.inner.db)
            .await?;

        Ok(())
    }

    pub async fn complete_event(&self, id: i32) -> Result<(), EventManagerError> {
        let completed_at = chrono::offset::Utc::now().naive_utc();
        events::ActiveModel {
            id: Set(id),
            is_completed: Set(true),
            completed_at: Set(Some(completed_at)),
            ..Default::default()
        }
        .update(&self.inner.db)
        .await?;

        Ok(())
    }

    pub async fn set_event_error(&self, id: i32, msg: &str) -> Result<(), EventManagerError> {
        events::ActiveModel {
            id: Set(id),
            error: Set(true),
            error_message: Set(Some(msg.to_owned())),
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
