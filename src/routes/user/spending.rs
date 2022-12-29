use crate::{
    guards::authorization::RequiredAuthorizationHeader,
    routes, shared,
    types::api::UserSpending,
    types::{error::ApiError, jwt::JwtClaim},
};
use jwt::{Header, Token};
use rocket::{serde::json::Json, State};

#[utoipa::path(
    responses(
        (status = 200, description = "Spending information for this user", body = UserSpending),
        (status = 500, description = "Internal Server Error"),
    ),
    tag = "user",
)]
#[get("/user/spending")]
pub async fn get_user_spending(
    ctx: &State<routes::Context<'_>>,
    auth: RequiredAuthorizationHeader,
) -> Result<Json<UserSpending>, ApiError> {
    let value = auth.0.as_str().replace("Bearer ", "");
    let jwt: Token<Header, JwtClaim, _> = jwt::Token::parse_unverified(&value)?;
    let user_id = &jwt.claims().oid;

    let entries = ctx
        .database
        .get_audit_entries_for(user_id.to_string())
        .await
        .unwrap_or_default();

    Ok(Json(shared::generate_spending_information(&entries)))
}
