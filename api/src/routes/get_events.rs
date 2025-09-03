use crate::types::{ApiState, AppError};
use api::{Event, GetEventsHistoryResponse, events::GetEventsResponse};
use axum::{
    Json,
    extract::{Query, State},
};
use entity::{events, sea_orm_active_enums::EventStatus};
use itertools::Itertools;
use sea_orm::{EntityTrait, QueryOrder, QuerySelect};
use strum::VariantNames;

#[derive(serde::Deserialize)]
pub struct Filter {
    limit: Option<u64>,
}

pub async fn get_events_history(
    State(ApiState { event_manager, .. }): State<ApiState>,
    Query(query): Query<Filter>,
) -> Result<Json<GetEventsHistoryResponse>, AppError> {
    let events = events::Entity::find()
        .order_by_desc(events::Column::CreatedAt)
        .limit(Some(query.limit.unwrap_or(50)))
        .all(event_manager.db())
        .await?;

    let mut active_events = vec![];
    let mut historical_events = vec![];

    for event in events {
        match event.status {
            EventStatus::Completed | EventStatus::Failed => historical_events.push(event),
            EventStatus::Pending | EventStatus::Running => active_events.push(event),
            EventStatus::Duplicate => historical_events.push(event),
        }
    }

    Ok(Json(GetEventsHistoryResponse {
        active_events,
        historical_events,
    }))
}

pub async fn get_events(
    State(ApiState { .. }): State<ApiState>,
) -> Result<Json<GetEventsResponse>, AppError> {
    Ok(Json(GetEventsResponse {
        events: Event::VARIANTS
            .iter()
            .cloned()
            .map(|s| s.to_owned())
            .collect_vec(),
    }))
}
