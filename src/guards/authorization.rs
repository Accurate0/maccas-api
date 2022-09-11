use crate::{routes, types::error::ApiError};
use http::header;
use rocket::{
    http::Status,
    outcome::Outcome,
    request::{self, FromRequest},
    Request, State,
};
use std::convert::Infallible;

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
        let ctx = request.guard::<&State<routes::Context>>().await;
        if ctx.is_failure() {
            return Outcome::Failure((Status::InternalServerError, ApiError::InternalServerError));
        };

        let ctx = ctx.unwrap();
        let auth_outcome = match auth {
            Some(auth) => Outcome::Success(Self(auth.to_string())),
            None => Outcome::Failure((Status::Unauthorized, ApiError::Unauthorized)),
        };

        if ctx.config.api.jwt.validate && auth_outcome.is_success() {
            let _auth_header = &auth_outcome.as_ref().unwrap().0.replace("Bearer ", "");
            // TODO: jwks validation
        }

        auth_outcome
    }
}
