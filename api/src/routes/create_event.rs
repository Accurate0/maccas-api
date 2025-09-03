use crate::types::{ApiState, AppError};
use api::{CreateBulkEvents, CreateBulkEventsResponse, CreateEvent, CreateEventResponse};
use axum::{Json, extract::State};
use opentelemetry::trace::TraceContextExt;

pub(crate) async fn create_event(
    State(ApiState { event_manager, .. }): State<ApiState>,
    Json(request): Json<CreateEvent>,
) -> Result<Json<CreateEventResponse>, AppError> {
    let trace_id = opentelemetry::Context::current()
        .span()
        .span_context()
        .trace_id();

    let id = event_manager
        .create_event(request.event.clone(), request.delay, trace_id.to_string())
        .await?;

    Ok(Json(CreateEventResponse { id }))
}

pub(crate) async fn create_bulk_events(
    State(ApiState { event_manager, .. }): State<ApiState>,
    Json(request): Json<CreateBulkEvents>,
) -> Result<Json<CreateBulkEventsResponse>, AppError> {
    let trace_id = opentelemetry::Context::current()
        .span()
        .span_context()
        .trace_id();

    let event_tasks = request.events.iter().map(async |e| {
        let result = event_manager
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

    Ok(Json(CreateBulkEventsResponse { ids: event_ids }))
}
