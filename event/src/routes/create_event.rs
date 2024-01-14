use crate::{error::EventError, state::AppState};
use actix_web::{
    post,
    web::{self},
};
use event::{CreateEvent, CreateEventResponse};

#[post("/event")]
pub(crate) async fn create_event(
    state: web::Data<AppState>,
    request: web::Json<CreateEvent>,
) -> Result<CreateEventResponse, EventError> {
    let id = state
        .event_manager
        .create_event(request.event.clone(), request.delay)
        .await?;

    Ok(CreateEventResponse { id })
}
