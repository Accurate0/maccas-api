use crate::constants::config::X_JWT_BYPASS_HEADER;
use crate::extensions::SecretsManagerExtensions;
use crate::{routes, types::error::ApiError};
use foundation::rocket::guards::authorization::RequiredAuthorizationHeader;
use rocket::{
    http::Status,
    outcome::Outcome,
    request::{self, FromRequest},
    Request, State,
};

pub struct AdminOnlyRoute;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AdminOnlyRoute {
    type Error = ApiError;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let ctx = request.guard::<&State<routes::Context>>().await;
        if ctx.is_failure() {
            return Outcome::Failure((Status::InternalServerError, ApiError::InternalServerError));
        };

        let ctx = ctx.unwrap();

        let jwt_bypass_key = request.headers().get_one(X_JWT_BYPASS_HEADER);
        if let Some(jwt_bypass_key) = jwt_bypass_key {
            if !jwt_bypass_key.is_empty()
                && jwt_bypass_key == ctx.secrets_client.get_jwt_bypass_key().await
            {
                return Outcome::Success(AdminOnlyRoute);
            }
        }

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
