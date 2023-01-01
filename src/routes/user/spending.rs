use crate::{routes, shared, types::api::UserSpending, types::error::ApiError};
use foundation::rocket::guards::authorization::RequiredAuthorizationHeader;
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
    let user_id = auth.claims.oid;

    let entries = ctx
        .database
        .get_audit_entries_for(user_id.to_string())
        .await
        .unwrap_or_default();

    Ok(Json(shared::generate_spending_information(&entries)))
}
