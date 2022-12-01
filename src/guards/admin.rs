use super::authorization::RequiredAuthorizationHeader;
use crate::{
    constants::X_JWT_BYPASS_HEADER,
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
            if !jwt_bypass_key.is_empty() && jwt_bypass_key == ctx.config.api.jwt.bypass_key {
                return Outcome::Success(AdminOnlyRoute);
            }
        }

        let auth_header = request.guard::<RequiredAuthorizationHeader>().await;
        if auth_header.is_failure() {
            return auth_header.map(|_| AdminOnlyRoute);
        };

        let auth_header = auth_header.unwrap();

        let value = auth_header.0.as_str().replace("Bearer ", "");
        let jwt: Token<Header, JwtClaim, _> = jwt::Token::parse_unverified(&value).unwrap();

        // if the admin extension is set and true, we allow
        let role = &jwt.claims().extension_role;
        let user_id = &jwt.claims().oid;

        if role.is_admin() {
            log::info!("allowing admin access to {user_id}, role = {:?}", role);
            Outcome::Success(AdminOnlyRoute)
        } else {
            Outcome::Failure((Status::Unauthorized, ApiError::Unauthorized))
        }
    }
}
