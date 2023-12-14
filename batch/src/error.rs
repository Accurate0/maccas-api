use sea_orm::DbErr;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum JobError {
    #[error("Database error has ocurred: `{0}`")]
    DateTimeParseError(#[from] DbErr),
    // #[error("unknown error")]
    // Unknown,
}
