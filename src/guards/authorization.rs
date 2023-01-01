use http::header;

use rocket::{
    outcome::Outcome,
    request::{self, FromRequest},
    Request,
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
