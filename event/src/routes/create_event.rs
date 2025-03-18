use crate::{error::EventError, state::AppState};
use actix_web::web;
use event::{CreateBulkEvents, CreateBulkEventsResponse, CreateEvent, CreateEventResponse};
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

pub(crate) async fn create_bulk_events(
    state: web::Data<AppState>,
    request: web::Json<CreateBulkEvents>,
) -> Result<CreateBulkEventsResponse, EventError> {
    let trace_id = opentelemetry::Context::current()
        .span()
        .span_context()
        .trace_id();

    let event_tasks = request.events.iter().map(async |e| {
        let result = state
            .event_manager
            .create_event(e.event.clone(), e.delay, trace_id.to_string())
            .await;

        if let Err(e) = result.as_ref() {
            tracing::error!("error creating event: {e}");
        }

        result
    });

    let event_ids = futures::future::join_all(event_tasks)
        .await
        .into_iter()
        .flatten()
        .collect();

    Ok(CreateBulkEventsResponse { ids: event_ids })
}
