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
}
