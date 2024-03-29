use crate::{error::EventError, state::AppState};
use actix_web::web::{self, Json};
use entity::events;
use event::GetEventsResponse;
use sea_orm::{EntityTrait, QueryOrder, QuerySelect};

#[derive(serde::Deserialize)]
pub struct Filter {
    limit: Option<u64>,
}

pub async fn get_events(
    state: web::Data<AppState>,
    query: web::Query<Filter>,
) -> Result<Json<GetEventsResponse>, EventError> {
    let events = events::Entity::find()
        .order_by_desc(events::Column::CreatedAt)
        .limit(Some(query.limit.unwrap_or(50)))
        .all(state.event_manager.db())
        .await?;

    let mut active_events = vec![];
    let mut historical_events = vec![];

    for event in events {
        match event.is_completed {
            true => historical_events.push(event),
            false => active_events.push(event),
        }
    }

    Ok(Json(GetEventsResponse {
        active_events,
        historical_events,
    }))
}
