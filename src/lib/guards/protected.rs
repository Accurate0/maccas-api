use crate::{
    routes,
    types::{error::ApiError, jwt::JwtClaim},
};
use jwt::{Header, Token};
use rocket::{
    http::Status,
    outcome::Outcome,
    request::{self, FromRequest},
    Request, State,
};

use super::authorization::RequiredAuthorizationHeader;

pub struct ProtectedRoute;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ProtectedRoute {
    type Error = ApiError;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let auth_header = request.guard::<RequiredAuthorizationHeader>().await;
        let ctx = request.guard::<&State<routes::Context>>().await;

        if ctx.is_failure() {
            return Outcome::Failure((Status::InternalServerError, ApiError::InternalServerError));
        };

        if auth_header.is_failure() {
            return auth_header.map(|_| ProtectedRoute);
        };

        let ctx = ctx.unwrap();
        let auth_header = auth_header.unwrap();

        let allowed_user_ids = &ctx.config.protected_routes.allowed_user_ids;
        let value = auth_header.0.as_str().replace("Bearer ", "");
        let jwt: Token<Header, JwtClaim, _> = jwt::Token::parse_unverified(&value).unwrap();
        let user_id = &jwt.claims().oid;

        if allowed_user_ids.iter().any(|id| id == user_id) {
            Outcome::Success(ProtectedRoute)
        } else {
            Outcome::Failure((Status::Unauthorized, ApiError::Unauthorized))
        }
    }
}
