use foundation::{constants::CORRELATION_ID_HEADER, util::get_uuid};
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
        let correlation_id = request
            .headers()
            .get_one(CORRELATION_ID_HEADER)
            .map_or_else(get_uuid, |s| s.to_owned());

        Outcome::Success(Self(correlation_id))
    }
}
