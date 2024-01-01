use crate::event_manager::EventManagerError;
use actix_web::error;
use sea_orm::DbErr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EventError {
    #[error("Database error has ocurred: `{0}`")]
    Database(#[from] DbErr),
    #[error("Event Manager error has ocurred: `{0}`")]
    EventManager(#[from] EventManagerError),
}

// Use default implementation for `error_response()` method
impl error::ResponseError for EventError {}
