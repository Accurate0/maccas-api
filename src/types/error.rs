use crate::constants::mc_donalds;
use aws_sdk_dynamodb::types::SdkError;
use http::StatusCode;
use rocket::response::Responder;
use rocket::{http::Status, response, Request};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ApiError {
    TryAgain,
    InvalidConfig,
    McDonaldsError,
    AccountNotAvailable,
    Unauthorized,
    NotFound,
    InternalServerError,
    UnhandledError,
    InvalidJwt,
}

impl ApiError {
    pub fn get_status(&self) -> Status {
        match self {
            ApiError::TryAgain => Status::new(599),
            ApiError::InvalidConfig => Status::BadRequest,
            ApiError::McDonaldsError => Status::BadRequest,
            ApiError::AccountNotAvailable => Status::Conflict,
            ApiError::Unauthorized => Status::Unauthorized,
            ApiError::InvalidJwt => Status::Unauthorized,
            ApiError::NotFound => Status::NotFound,
            ApiError::InternalServerError => Status::InternalServerError,
            ApiError::UnhandledError => Status::InternalServerError,
        }
    }
}

impl<'r> Responder<'r, 'static> for ApiError {
    fn respond_to(self, _req: &'r Request<'_>) -> response::Result<'static> {
        Err(self.get_status())
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(e: anyhow::Error) -> Self {
        log::error!("UNHANDLED ERROR: {:#?}", e);
        Self::UnhandledError
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(e: serde_json::Error) -> Self {
        log::error!("serde_json: UNHANDLED ERROR: {:#?}", e);
        Self::UnhandledError
    }
}

// getting a lot of 403 from McDonalds recently
// often this resolves itself by simply trying again
// so we'll instruct the client (APIM) to retry this error
fn handle_mcdonalds_403(e: reqwest::Error) -> ApiError {
    if let Some(status_code) = e.status() {
        match status_code {
            StatusCode::FORBIDDEN => match e.url() {
                Some(url) if url.as_str().starts_with(mc_donalds::default::BASE_URL) => {
                    ApiError::TryAgain
                }
                _ => ApiError::UnhandledError,
            },
            _ => ApiError::UnhandledError,
        }
    } else {
        ApiError::UnhandledError
    }
}

impl From<reqwest::Error> for ApiError {
    fn from(e: reqwest::Error) -> Self {
        log::error!("reqwest: UNHANDLED ERROR: {:#?}", e);
        handle_mcdonalds_403(e)
    }
}

impl From<reqwest_middleware::Error> for ApiError {
    fn from(e: reqwest_middleware::Error) -> Self {
        match e {
            reqwest_middleware::Error::Middleware(e) => {
                log::error!("reqwest_middleware: UNHANDLED ERROR: {:#?}", e);
                Self::UnhandledError
            }
            reqwest_middleware::Error::Reqwest(e) => handle_mcdonalds_403(e),
        }
    }
}

impl From<jwt::Error> for ApiError {
    fn from(e: jwt::Error) -> Self {
        log::error!("jwt: UNHANDLED ERROR: {:#?}", e);
        Self::UnhandledError
    }
}

impl<T> From<SdkError<T>> for ApiError {
    fn from(e: SdkError<T>) -> Self {
        log::error!("AWS SDK: UNHANDLED ERROR: {}", e);
        Self::UnhandledError
    }
}
