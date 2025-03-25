use crate::types::ApiState;
use axum::extract::State;
use reqwest::StatusCode;

pub async fn health(State(state): State<ApiState>) -> StatusCode {
    if state.engine.is_healthy().await {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    }
}
