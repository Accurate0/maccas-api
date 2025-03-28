use crate::types::{ApiState, AppError};
use axum::extract::{Path, State};
use entity::offer_details;
use reqwest::StatusCode;
use sea_orm::EntityTrait;

pub async fn generate(State(state): State<ApiState>) -> Result<StatusCode, AppError> {
    state.engine.refresh_all_embeddings().await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn generate_for(
    State(state): State<ApiState>,
    Path(proposition_id): Path<i64>,
) -> Result<StatusCode, AppError> {
    let model = offer_details::Entity::find_by_id(proposition_id)
        .one(state.engine.db())
        .await?;

    if model.is_none() {
        return Ok(StatusCode::BAD_REQUEST);
    }

    state
        .engine
        .refresh_embedding_for(proposition_id, model.unwrap().short_name, true)
        .await?;

    Ok(StatusCode::CREATED)
}
pub async fn generate_clusters(State(state): State<ApiState>) -> Result<StatusCode, AppError> {
    state.engine.generate_clusters().await?;
    Ok(StatusCode::NO_CONTENT)
}
