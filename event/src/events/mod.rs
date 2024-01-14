use actix_web::{body::EitherBody, error::JsonPayloadError, HttpRequest, HttpResponse, Responder};
use core::fmt;
use sea_orm::prelude::Uuid;
use std::time::Duration;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub enum Event {
    Cleanup { offer_id: Uuid },
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Event::Cleanup { .. } => write!(f, "Cleanup"),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct CreateEvent {
    pub event: Event,
    pub delay: Duration,
}

impl CreateEvent {
    pub fn path() -> &'static str {
        "event"
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct CreateEventResponse {
    pub id: Uuid,
}

impl Responder for CreateEventResponse {
    type Body = EitherBody<String>;

    fn respond_to(self, _: &HttpRequest) -> HttpResponse<Self::Body> {
        match serde_json::to_string(&self) {
            Ok(body) => match HttpResponse::Created()
                .content_type("application/json")
                .message_body(body)
            {
                Ok(res) => res.map_into_left_body(),
                Err(err) => HttpResponse::from_error(err).map_into_right_body(),
            },

            Err(err) => {
                HttpResponse::from_error(JsonPayloadError::Serialize(err)).map_into_right_body()
            }
        }
    }
}
