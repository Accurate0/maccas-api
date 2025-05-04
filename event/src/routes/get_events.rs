use crate::{error::EventError, state::AppState};
use actix_web::web::{self, Json};
use entity::{events, sea_orm_active_enums::EventStatus};
use event::{events::GetEventsResponse, Event, GetEventsHistoryResponse};
use itertools::Itertools;
use sea_orm::{EntityTrait, QueryOrder, QuerySelect};
use strum::VariantNames;

#[derive(serde::Deserialize)]
pub struct Filter {
    limit: Option<u64>,
}

pub async fn get_events_history(
    state: web::Data<AppState>,
    query: web::Query<Filter>,
) -> Result<Json<GetEventsHistoryResponse>, EventError> {
    let events = events::Entity::find()
        .order_by_desc(events::Column::CreatedAt)
        .limit(Some(query.limit.unwrap_or(50)))
        .all(state.event_manager.db())
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
    _state: web::Data<AppState>,
) -> Result<Json<GetEventsResponse>, EventError> {
    Ok(Json(GetEventsResponse {
        events: Event::VARIANTS
            .iter()
            .cloned()
            .map(|s| s.to_owned())
            .collect_vec(),
    }))
}
