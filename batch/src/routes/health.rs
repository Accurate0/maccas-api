use crate::types::ApiState;
use axum::extract::State;
use reqwest::StatusCode;

pub async fn health(State(state): State<ApiState>) -> StatusCode {
    if state.job_scheduler.db().ping().await.is_ok() {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    }
}
