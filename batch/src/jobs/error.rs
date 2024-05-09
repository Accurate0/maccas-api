use base::http::HttpCreationError;
use sea_orm::DbErr;
use thiserror::Error;
use tokio::sync::mpsc::error::SendError;

use super::job_scheduler;

#[derive(Error, Debug)]
pub enum JobError {
    #[error("Database error has ocurred: `{0}`")]
    Database(#[from] DbErr),
    #[error("Serialization error has occurred: `{0}`")]
    Serialization(#[from] serde_json::Error),
    #[error("Chrono error has occurred: `{0}`")]
    Chrono(#[from] chrono::OutOfRangeError),
    #[error("Send error has occurred: `{0}`")]
    Send(#[from] SendError<job_scheduler::Message>),
    #[error("An unknown error ocurred: `{0}`")]
    UnknownError(#[from] anyhow::Error),
    #[error("McDonald's client error occurred: `{0}`")]
    McDonaldsClientError(#[from] libmaccas::ClientError),
    #[error("OpenAI client error occurred: `{0}`")]
    OpenAIClientError(#[from] openai::types::ClientError),
    #[error("A conversion error ocurred: `{0}`")]
    ConversionError(#[from] converters::ConversionError),
    #[error("A imap error ocurred: `{0}`")]
    ImapError(#[from] imap::Error),
    #[error("A mail parse error ocurred: `{0}`")]
    MailParseError(#[from] mailparse::MailParseError),
    #[error("A reqwest error ocurred: `{0}`")]
    ReqwestError(#[from] reqwest::Error),
    #[error("A http creation error ocurred: `{0}`")]
    HttpCreationError(#[from] HttpCreationError),
    #[error("A reqwest middleware error ocurred: `{0}`")]
    ReqwestMiddlewareError(#[from] reqwest_middleware::Error),
}
