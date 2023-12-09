use crate::{
    database::audit::AuditRepository, guards::required_authorization::RequiredAuthorizationHeader,
    shared::spending, types::api::UserSpending, types::error::ApiError,
};
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
    audit_repo: &State<AuditRepository>,
    auth: RequiredAuthorizationHeader,
) -> Result<Json<UserSpending>, ApiError> {
    let user_id = auth.claims.oid;

    let entries = audit_repo
        .get_entries(user_id.to_string())
        .await
        .unwrap_or_default();

    Ok(Json(spending::generate_spending_information(&entries)))
}
