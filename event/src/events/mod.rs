use actix_web::{HttpRequest, HttpResponse, Responder, body::EitherBody, error::JsonPayloadError};
use core::fmt;
use sea_orm::prelude::Uuid;
use std::time::Duration;
use strum::VariantNames;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, VariantNames)]
pub enum Event {
    UnlockAllAccounts {},
    ActivateAccount {},
    ActivateExistingAccount {},
    CategoriseOffers {},
    CreateAccount {},
    GenerateRecommendations {},
    RecategoriseOffers {},
    Refresh {},
    RefreshAccount {
        account_id: Uuid,
    },
    SaveImages {},
    Cleanup {
        offer_id: Uuid,
        transaction_id: Uuid,
        audit_id: i32,
        store_id: String,
        account_id: Uuid,
        user_id: Option<Uuid>,
    },
    SaveImage {
        basename: String,
        #[serde(default)]
        force: bool,
    },
    RefreshPoints {
        account_id: Uuid,
    },
    NewOfferFound {
        offer_proposition_id: i64,
    },
    PopulateOfferDetailsCache,
}

impl MaybeJobScheduler for Event {
    fn name(&self) -> Option<&'static str> {
        match self {
            Event::UnlockAllAccounts {} => Some("account_unlock"),
            Event::ActivateAccount {} => Some("activate_account"),
            Event::ActivateExistingAccount {} => Some("activate_existing_account"),
            Event::CategoriseOffers {} => Some("categorise_offers"),
            Event::CreateAccount {} => Some("create_account"),
            Event::GenerateRecommendations {} => Some("generate_recommendations"),
            Event::RecategoriseOffers {} => Some("recategorise_offers"),
            Event::Refresh {} => Some("refresh"),
            Event::SaveImages {} => Some("save_images"),
            _ => None,
        }
    }
}

pub trait MaybeJobScheduler {
    fn name(&self) -> Option<&'static str>;
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Event::Cleanup { .. } => write!(f, "Cleanup"),
            Event::SaveImage { .. } => write!(f, "SaveImage"),
            Event::RefreshPoints { .. } => write!(f, "RefreshPoints"),
            Event::NewOfferFound { .. } => write!(f, "NewOfferFound"),
            Event::UnlockAllAccounts {} => write!(f, "UnlockAllAccounts "),
            Event::ActivateAccount {} => write!(f, "ActivateAccount"),
            Event::ActivateExistingAccount {} => write!(f, "ActivateExistingAccount"),
            Event::CategoriseOffers {} => write!(f, "CategoriseOffers"),
            Event::CreateAccount {} => write!(f, "CreateAccount"),
            Event::GenerateRecommendations {} => write!(f, "GenerateRecommendations"),
            Event::RecategoriseOffers {} => write!(f, "RecategoriseOffers"),
            Event::Refresh {} => write!(f, "Refresh"),
            Event::SaveImages {} => write!(f, "SaveImages"),
            Event::RefreshAccount { .. } => write!(f, "RefreshAccount"),
            Event::PopulateOfferDetailsCache => write!(f, "PopulateOfferDetailsCache"),
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
pub struct CreateBulkEvents {
    pub events: Vec<CreateEvent>,
}

impl CreateBulkEvents {
    pub fn path() -> &'static str {
        "event/bulk"
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct ExistingEvent {
    pub event: Event,
    pub remaining: Duration,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct GetEventsHistoryResponse {
    pub active_events: Vec<entity::events::Model>,
    pub historical_events: Vec<entity::events::Model>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct GetEventsResponse {
    pub events: Vec<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Health;

impl Health {
    pub fn path() -> &'static str {
        "health"
    }
}
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct CreateEventResponse {
    pub id: Uuid,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct CreateBulkEventsResponse {
    pub ids: Vec<Uuid>,
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

impl Responder for CreateBulkEventsResponse {
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
