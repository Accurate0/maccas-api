use crate::{
    constants::config::CONFIG_SECRET_KEY_ID,
    guards::admin::AdminOnlyRoute,
    routes,
    shared::jwt,
    types::{error::ApiError, role::UserRole},
};
use foundation::extensions::SecretsManagerExtensions;
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
    let secret = ctx.secrets_client.get_secret(CONFIG_SECRET_KEY_ID).await?;
    let application_id = &ctx.config.api.jwt.application_id;
    Ok(jwt::generate_shared_registration_token(
        secret,
        application_id,
        &role,
    )?)
}
