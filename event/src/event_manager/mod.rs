use base::delay_queue::DelayQueue;
use entity::events;
use event::Event;
use sea_orm::prelude::Uuid;
use sea_orm::{ActiveModelTrait, DatabaseConnection, DbErr, Set};
use std::{sync::Arc, time::Duration};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EventManagerError {
    #[error("Serializer error has ocurred: `{0}`")]
    Serializer(#[from] serde_json::Error),
    #[error("Database error has ocurred: `{0}`")]
    Database(#[from] DbErr),
}

struct EventManagerInner {
    db: DatabaseConnection,
    event_queue: DelayQueue<Event>,
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

    pub async fn create(&self, evt: Event, delay: Duration) -> Result<Uuid, EventManagerError> {
        let event_id = Uuid::new_v4();
        let should_be_completed_at = chrono::offset::Utc::now().naive_utc() + delay;

        events::ActiveModel {
            name: Set(evt.to_string()),
            event_id: Set(event_id),
            data: Set(serde_json::to_value(&evt)?),
            is_completed: Set(false),
            should_be_completed_at: Set(should_be_completed_at),
            ..Default::default()
        }
        .insert(&self.inner.db)
        .await?;

        self.inner.event_queue.push(evt, delay).await;

        Ok(event_id)
    }
}

impl Clone for EventManager {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
