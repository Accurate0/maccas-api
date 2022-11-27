use crate::{
    constants::X_JWT_BYPASS_HEADER,
    extensions::BoolExtensions,
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

        let allowed_user_ids = &ctx.config.api.protected_routes.allowed_user_ids;
        let value = auth_header.0.as_str().replace("Bearer ", "");
        let jwt: Token<Header, JwtClaim, _> = jwt::Token::parse_unverified(&value).unwrap();
        let user_id = &jwt.claims().oid;
        let is_admin = jwt.claims().extension_admin_user.unwrap_or_false();

        // allow admin access to protected routes
        if is_admin || allowed_user_ids.iter().any(|id| id == user_id) {
            log::info!("allowing protected access to {user_id}, is_admin = {is_admin}");
            Outcome::Success(ProtectedRoute)
        } else {
            Outcome::Failure((Status::Unauthorized, ApiError::Unauthorized))
        }
    }
}
