use crate::{
    constants::config::{CONFIG_SECRET_KEY_ID, TOKEN_SHARED_REGISTER_ISS},
    routes,
    shared::jwt::generate_signed_jwt,
    types::{
        api::{RegistrationRequest, TokenResponse},
        error::ApiError,
        token::SharedTokenClaims,
    },
};
use foundation::extensions::SecretsManagerExtensions;
use hmac::{digest::KeyInit, Hmac};
use jwt::{Header, Token, VerifyWithKey};
use rand::Rng;
use rocket::{serde::json::Json, State};
use sha2::Sha256;

#[utoipa::path(
    responses(
        (status = 200, description = "Register a new account using a shared token", body = TokenResponse),
        (status = 401, description = "Account doesn't exist"),
        (status = 500, description = "Internal Server Error"),
    ),
    tag = "auth",
)]
#[post("/auth/register?<token>", data = "<request>")]
pub async fn register(
    ctx: &State<routes::Context<'_>>,
    request: Json<RegistrationRequest>,
    token: String,
) -> Result<Json<TokenResponse>, ApiError> {
    let secret = ctx.secrets_client.get_secret(CONFIG_SECRET_KEY_ID).await?;
    let key: Hmac<Sha256> =
        Hmac::new_from_slice(secret.as_bytes()).map_err(|_| ApiError::Unauthorized)?;

    let unverified: Token<Header, SharedTokenClaims, jwt::Unverified<'_>> =
        Token::parse_unverified(&token)?;
    let token: Token<_, _, jwt::Verified> = unverified.verify_with_key(&key)?;

    if token.claims().aud != ctx.config.api.jwt.application_id
        && token.claims().iss != TOKEN_SHARED_REGISTER_ISS
    {
        return Err(ApiError::Unauthorized);
    }

    if ctx
        .database
        .is_user_exist(request.username.to_owned())
        .await?
    {
        log::info!("user: {} already exists, can't register", request.username);
        return Err(ApiError::Conflict);
    }

    let role = &token.claims().role;

    let salt: [u8; 16] = rand::thread_rng().gen();
    let password_hash = bcrypt::hash_with_salt(request.password.clone(), 10, salt)
        .map_err(|_| ApiError::InternalServerError)?;

    let user_id = uuid::Uuid::new_v4().as_hyphenated().to_string();
    ctx.database
        .create_user(
            user_id.clone(),
            request.username.clone(),
            password_hash.to_string(),
            salt.to_vec(),
        )
        .await?;

    let new_jwt = generate_signed_jwt(
        secret,
        &user_id,
        &ctx.config.api.jwt.application_id,
        role,
        &request.username,
    )?;

    let refresh_token = uuid::Uuid::new_v4().as_hyphenated().to_string();

    ctx.database
        .set_user_tokens(
            &request.username,
            &new_jwt,
            &refresh_token,
            chrono::Duration::days(7),
        )
        .await?;

    Ok(Json(TokenResponse {
        token: new_jwt,
        refresh_token,
        role: role.clone(),
    }))
}
