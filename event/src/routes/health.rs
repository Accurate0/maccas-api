use crate::{error::EventError, state::AppState};
use actix_web::{
    http::StatusCode,
    web::{self},
    HttpResponse, HttpResponseBuilder,
};

pub(crate) async fn health(state: web::Data<AppState>) -> Result<HttpResponse, EventError> {
    let database_healthy = state.event_manager.db().ping().await.is_ok();

    Ok(HttpResponseBuilder::new(if database_healthy {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    })
    .finish())
}
