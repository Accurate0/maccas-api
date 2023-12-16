use sea_orm::DbErr;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum JobError {
    #[error("Database error has ocurred: `{0}`")]
    DatabaseError(#[from] DbErr),
    #[error("Scheduler execution was cancelled")]
    SchedulerCancelled,
    #[error("Serialization error has occurred: `{0}`")]
    SerializationError(#[from] serde_json::Error),
    // #[error("unknown error")]
    // Unknown,
}
