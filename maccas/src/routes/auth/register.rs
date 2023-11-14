use crate::{
    constants::config::CONFIG_SECRET_KEY_ID,
    database::user::UserRepository,
    routes::{self},
    shared::jwt::generate_signed_jwt,
    types::{
        api::{RegistrationRequest, TokenResponse},
        error::ApiError,
    },
};
use foundation::extensions::SecretsManagerExtensions;
use rand::Rng;
use rocket::{form::Form, serde::json::Json, State};

#[utoipa::path(
    responses(
        (status = 200, description = "Register a new account using a shared token", body = TokenResponse),
        (status = 404, description = "Token has expired"),
        (status = 409, description = "Account with this username already exists"),
        (status = 500, description = "Internal Server Error"),
    ),
    tag = "auth",
)]
#[post("/auth/register", data = "<request>")]
pub async fn register(
    ctx: &State<routes::Context>,
    user_repo: &State<UserRepository>,
    request: Form<RegistrationRequest>,
) -> Result<Json<TokenResponse>, ApiError> {
    let secret = ctx.secrets_client.get_secret(CONFIG_SECRET_KEY_ID).await?;

    let registration_token = &request.token.as_hyphenated().to_string();
    let metadata = user_repo.get_registration_token(registration_token).await?;

    if metadata.is_single_use && metadata.use_count > 0 {
        return Err(ApiError::NotFound);
    }

    if user_repo.is_user_exist(request.username.to_owned()).await? {
        log::info!("user: {} already exists, can't register", request.username);
        return Err(ApiError::Conflict);
    }

    let salt: [u8; 16] = rand::thread_rng().gen();
    let password_hash = bcrypt::hash_with_salt(request.password.clone(), 10, salt)
        .map_err(|_| ApiError::InternalServerError)?;

    let user_id = uuid::Uuid::new_v4().as_hyphenated().to_string();
    user_repo
        .create_user(
            user_id.clone(),
            request.username.clone(),
            password_hash.to_string(),
            salt.to_vec(),
            false,
            Some(registration_token),
        )
        .await?;

    user_repo
        .set_user_role(request.username.clone(), metadata.role.clone())
        .await?;

    let new_jwt = generate_signed_jwt(
        secret,
        &user_id,
        &ctx.config.api.jwt.application_id,
        &metadata.role,
        &request.username,
    )?;

    let refresh_token = uuid::Uuid::new_v4().as_hyphenated().to_string();

    user_repo
        .set_user_tokens(
            &request.username,
            &new_jwt,
            vec![refresh_token.clone()],
            chrono::Duration::days(7),
        )
        .await?;

    user_repo
        .set_registration_token_use_count(registration_token, metadata.use_count + 1)
        .await?;

    Ok(Json(TokenResponse {
        token: new_jwt,
        refresh_token,
        role: metadata.role.clone(),
    }))
}
