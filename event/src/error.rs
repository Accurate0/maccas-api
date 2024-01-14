use crate::event_manager::EventManagerError;
use actix_web::{error, http::StatusCode, HttpResponse, ResponseError};
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

#[derive(thiserror::Error, Debug)]
pub enum MiddlewareError {
    #[error("Unauthenticated request")]
    Unauthenticated,
}

impl ResponseError for MiddlewareError {
    fn status_code(&self) -> StatusCode {
        match &self {
            Self::Unauthenticated => StatusCode::UNAUTHORIZED,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}
