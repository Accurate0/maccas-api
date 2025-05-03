use base::{http::HttpCreationError, jwt::JwtValidationError};
use sea_orm::DbErr;
use thiserror::Error;
use tokio::sync::mpsc::error::SendError;

use crate::event_manager::EventManagerError;

use super::job_scheduler;

#[derive(Error, Debug)]
pub enum JobError {
    #[error("Database error has occurred: `{0}`")]
    Database(#[from] DbErr),
    #[error("EventManager error has occurred: `{0}`")]
    EventManager(#[from] EventManagerError),
    #[error("DelayQueue error has occurred: `{0}`")]
    DelayQueue(#[from] crate::queue::DelayQueueError),
    #[error("Serialization error has occurred: `{0}`")]
    Serialization(#[from] serde_json::Error),
    #[error("Chrono error has occurred: `{0}`")]
    Chrono(#[from] chrono::OutOfRangeError),
    #[error("Send error has occurred: `{0}`")]
    Send(#[from] SendError<job_scheduler::JobMessage>),
    #[error("An unknown error occurred: `{0}`")]
    UnknownError(#[from] anyhow::Error),
    #[error("McDonald's client error occurred: `{0}`")]
    McDonaldsClientError(#[from] libmaccas::ClientError),
    #[error("OpenAI client error occurred: `{0}`")]
    OpenAIClientError(#[from] openai::types::ClientError),
    #[error("A conversion error occurred: `{0}`")]
    ConversionError(#[from] converters::ConversionError),
    #[error("A imap error occurred: `{0}`")]
    ImapError(#[from] imap::Error),
    #[error("A mail parse error occurred: `{0}`")]
    MailParseError(#[from] mailparse::MailParseError),
    #[error("A reqwest error occurred: `{0}`")]
    ReqwestError(#[from] reqwest::Error),
    #[error("A http creation error occurred: `{0}`")]
    HttpCreationError(#[from] HttpCreationError),
    #[error("A reqwest middleware error occurred: `{0}`")]
    ReqwestMiddlewareError(#[from] reqwest_middleware::Error),
    #[error("A jwt validation error occurred: `{0}`")]
    JwtValidation(#[from] JwtValidationError),
}
