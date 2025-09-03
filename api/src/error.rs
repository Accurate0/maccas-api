use crate::event_manager::EventManagerError;
use sea_orm::DbErr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EventError {
    #[error("Database error has occurred: `{0}`")]
    Database(#[from] DbErr),
    #[error("Event Manager error has occurred: `{0}`")]
    EventManager(#[from] EventManagerError),
}
