use http::header;
use rocket::{
    http::Status,
    outcome::Outcome,
    request::{self, FromRequest},
    Request,
};
use std::convert::Infallible;

use crate::types::error::ApiError;

pub struct AuthorizationHeader(pub Option<String>);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthorizationHeader {
    type Error = Infallible;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let auth = request.headers().get_one(header::AUTHORIZATION.as_str());
        Outcome::Success(Self(auth.map(|s| s.to_string())))
    }
}

pub struct RequiredAuthorizationHeader(pub String);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for RequiredAuthorizationHeader {
    type Error = ApiError;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let auth = request.headers().get_one(header::AUTHORIZATION.as_str());
        match auth {
            Some(auth) => Outcome::Success(Self(auth.to_string())),
            None => Outcome::Failure((Status::Unauthorized, ApiError::Unauthorized)),
        }
    }
}
