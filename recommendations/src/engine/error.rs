use sea_orm::DbErr;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RecommendationError {
    #[error("Database error has occurred: `{0}`")]
    Database(#[from] DbErr),
    #[error("An shape error has occurred: `{0}`")]
    ShapeError(#[from] ndarray::ShapeError),
    #[error("An join error has occurred: `{0}`")]
    JoinError(#[from] tokio::task::JoinError),
    #[error("An unexpected error has occurred: `{0}`")]
    Other(#[from] anyhow::Error),
}
