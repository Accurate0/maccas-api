use crate::{error::EventError, state::AppState};
use actix_web::web;
use event::{CreateEvent, CreateEventResponse};
use opentelemetry::trace::TraceContextExt;

pub(crate) async fn create_event(
    state: web::Data<AppState>,
    request: web::Json<CreateEvent>,
) -> Result<CreateEventResponse, EventError> {
    let trace_id = opentelemetry::Context::current()
        .span()
        .span_context()
        .trace_id();

    let id = state
        .event_manager
        .create_event(request.event.clone(), request.delay, trace_id.to_string())
        .await?;

    Ok(CreateEventResponse { id })
}
