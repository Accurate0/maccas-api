use sea_orm::DbErr;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RecommendationError {
    #[error("Database error has occurred: `{0}`")]
    Database(#[from] DbErr),
    #[error("An unexpected error has occurred: `{0}`")]
    Other(#[from] anyhow::Error),
}
