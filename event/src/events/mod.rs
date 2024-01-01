use core::fmt;
use std::time::Duration;

use sea_orm::prelude::Uuid;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub enum Event {
    Cleanup { offer_id: String },
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Event::Cleanup { .. } => write!(f, "Cleanup"),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct CreateEvent {
    pub event: Event,
    pub delay: Duration,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct CreateEventResponse {
    pub id: Uuid,
}
