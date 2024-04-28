use crate::{error::EventError, state::AppState};
use actix_web::web;
use event::{CreateEvent, CreateEventResponse};

pub(crate) async fn create_event(
    state: web::Data<AppState>,
    request: web::Json<CreateEvent>,
    req: actix_web::HttpRequest,
) -> Result<CreateEventResponse, EventError> {
    tracing::info!("{:#?}", req.headers().get("traceparent"));
    let id = state
        .event_manager
        .create_event(request.event.clone(), request.delay)
        .await?;

    Ok(CreateEventResponse { id })
}
