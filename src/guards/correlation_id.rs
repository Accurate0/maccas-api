use crate::{constants::CORRELATION_ID_HEADER, utils::get_uuid};
use rocket::{
    outcome::Outcome,
    request::{self, FromRequest},
    Request,
};
use std::convert::Infallible;

pub struct CorrelationId(pub String);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for CorrelationId {
    type Error = Infallible;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let auth = request.headers().get_one(CORRELATION_ID_HEADER);
        Outcome::Success(Self(match auth {
            Some(s) => s.to_string(),
            None => get_uuid(),
        }))
    }
}
