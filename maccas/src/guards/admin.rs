use crate::types::error::ApiError;
use rocket::{
    http::Status,
    outcome::Outcome,
    request::{self, FromRequest},
    Request,
};

use super::required_authorization::RequiredAuthorizationHeader;

pub struct AdminOnlyRoute;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AdminOnlyRoute {
    type Error = ApiError;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let auth_header = request.guard::<RequiredAuthorizationHeader>().await;
        if auth_header.is_failure() {
            return auth_header
                .map(|_| AdminOnlyRoute)
                .map_failure(|_| (Status::Unauthorized, ApiError::Unauthorized));
        };

        let auth_header = auth_header.unwrap();

        // if the admin extension is set and true, we allow
        let role = auth_header.claims.extension_role;
        let user_id = auth_header.claims.oid;

        if role.is_admin() {
            log::info!("allowing admin access to {user_id}, role = {:?}", role);
            Outcome::Success(AdminOnlyRoute)
        } else {
            Outcome::Failure((Status::Unauthorized, ApiError::Unauthorized))
        }
    }
}
