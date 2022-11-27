use crate::types::jwt::JwtClaim;
use crate::{constants::X_JWT_BYPASS_HEADER, routes, types::error::ApiError};
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
        let ctx = request.guard::<&State<routes::Context>>().await;
        if ctx.is_failure() {
            return Outcome::Failure((Status::InternalServerError, ApiError::InternalServerError));
        };

        let ctx = ctx.unwrap();

        let jwt_bypass_key = request.headers().get_one(X_JWT_BYPASS_HEADER);
        if let Some(jwt_bypass_key) = jwt_bypass_key {
            if !jwt_bypass_key.is_empty() && jwt_bypass_key == ctx.config.api.jwt.bypass_key {
                return Outcome::Success(ProtectedRoute);
            }
        }

        let auth_header = request.guard::<RequiredAuthorizationHeader>().await;
        if auth_header.is_failure() {
            return auth_header.map(|_| ProtectedRoute);
        };

        let auth_header = auth_header.unwrap();

        let value = auth_header.0.as_str().replace("Bearer ", "");
        let jwt: Token<Header, JwtClaim, _> = jwt::Token::parse_unverified(&value).unwrap();
        let user_id = &jwt.claims().oid;
        let role = &jwt.claims().extension_role;

        // allow admin access to protected routes
        if role.is_allowed_protected_access() {
            log::info!("allowing protected access to {user_id}, role = {:?}", role);
            Outcome::Success(ProtectedRoute)
        } else {
            Outcome::Failure((Status::Unauthorized, ApiError::Unauthorized))
        }
    }
}
