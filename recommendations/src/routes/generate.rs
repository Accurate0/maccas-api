use crate::types::{ApiState, AppError};
use axum::{
    Json,
    extract::{Path, State},
};
use entity::offer_details;
use recommendations::GenerateClusterScores;
use reqwest::StatusCode;
use sea_orm::{EntityTrait, TransactionTrait};

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
        .refresh_embedding_for(model.unwrap().short_name, true)
        .await?;

    Ok(StatusCode::CREATED)
}
pub async fn generate_clusters(State(state): State<ApiState>) -> Result<StatusCode, AppError> {
    state.engine.generate_clusters().await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn generate_cluster_scores(
    State(state): State<ApiState>,
    Json(body): Json<GenerateClusterScores>,
) -> Result<StatusCode, AppError> {
    let txn = state.engine.db().begin().await?;
    for user_id in body.user_ids {
        state
            .engine
            .generate_recommendations_for_user(user_id, &txn)
            .await?;
    }
    txn.commit().await?;

    Ok(StatusCode::NO_CONTENT)
}
