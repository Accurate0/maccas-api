use crate::types::error::ApiError;
use rocket::{
    http::Status,
    outcome::Outcome,
    request::{self, FromRequest},
    Request,
};

use super::required_authorization::RequiredAuthorizationHeader;

pub struct ProtectedRoute;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ProtectedRoute {
    type Error = ApiError;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let auth_header = request.guard::<RequiredAuthorizationHeader>().await;
        if auth_header.is_failure() {
            return auth_header
                .map(|_| ProtectedRoute)
                .map_failure(|_| (Status::Unauthorized, ApiError::Unauthorized));
        };

        let auth_header = auth_header.unwrap();

        let user_id = auth_header.claims.oid;
        let role = auth_header.claims.extension_role;

        // allow admin access to protected routes
        if role.is_allowed_protected_access() {
            log::info!("allowing protected access to {user_id}, role = {:?}", role);
            Outcome::Success(ProtectedRoute)
        } else {
            Outcome::Failure((Status::Unauthorized, ApiError::Unauthorized))
        }
    }
}
