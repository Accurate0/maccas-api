use aws_sdk_dynamodb::types::SdkError;
use http::StatusCode;
use rocket::response::Responder;
use rocket::{http::Status, response, Request, Response};
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
        let mut response = Response::new();
        response.set_status(self.get_status());
        Ok(response)
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
        log::error!("UNHANDLED ERROR: {:#?}", e);
        Self::UnhandledError
    }
}

impl From<reqwest::Error> for ApiError {
    fn from(e: reqwest::Error) -> Self {
        log::error!("UNHANDLED ERROR: {:#?}", e);
        if let Some(status_code) = e.status() {
            match status_code {
                // getting a lot of 403 from McDonalds recently
                // often this resolves itself by simply trying again
                // so we'll instruct the client (APIM) to retry this error
                StatusCode::FORBIDDEN => Self::TryAgain,
                _ => Self::UnhandledError,
            }
        } else {
            Self::UnhandledError
        }
    }
}

impl From<reqwest_middleware::Error> for ApiError {
    fn from(e: reqwest_middleware::Error) -> Self {
        log::error!("UNHANDLED ERROR: {:#?}", e);
        Self::UnhandledError
    }
}

impl From<jwt::Error> for ApiError {
    fn from(e: jwt::Error) -> Self {
        log::error!("UNHANDLED ERROR: {:#?}", e);
        Self::UnhandledError
    }
}

impl<T> From<SdkError<T>> for ApiError {
    fn from(_: SdkError<T>) -> Self {
        log::error!("UNHANDLED ERROR: SDK ERROR, NO MESSAGE");
        Self::UnhandledError
    }
}
