use crate::{
    guards::admin::AdminOnlyRoute,
    routes,
    types::{error::ApiError, role::UserRole},
};
use rocket::State;

#[utoipa::path(
    responses(
        (status = 200, description = "Token that can be used for registration", body = String),
        (status = 500, description = "Internal Server Error"),
    ),
    tag = "admin",
)]
#[post("/admin/auth/register?<role>")]
pub async fn registration_token(
    ctx: &State<routes::Context<'_>>,
    _admin: AdminOnlyRoute,
    role: UserRole,
) -> Result<String, ApiError> {
    let registration_token = uuid::Uuid::new_v4().as_hyphenated().to_string();

    ctx.database
        .create_registration_token(&registration_token, role)
        .await?;

    Ok(registration_token)
}
