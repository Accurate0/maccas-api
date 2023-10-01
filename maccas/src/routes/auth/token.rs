use crate::{
    constants::config::CONFIG_SECRET_KEY_ID,
    routes,
    shared::jwt::generate_signed_jwt,
    types::{
        api::{TokenRequest, TokenResponse},
        error::ApiError,
        token::JwtClaim,
    },
};
use foundation::extensions::SecretsManagerExtensions;
use hmac::{digest::KeyInit, Hmac};
use jwt::{Header, Token, VerifyWithKey};
use rocket::{serde::json::Json, State};
use sha2::Sha256;

#[utoipa::path(
    responses(
        (status = 200, description = "Trade previous access and refresh token for new ones", body = TokenResponse),
        (status = 500, description = "Internal Server Error"),
    ),
    tag = "auth",
)]
#[post("/auth/token", data = "<request>")]
pub async fn get_token(
    ctx: &State<routes::Context<'_>>,
    request: Json<TokenRequest>,
) -> Result<Json<TokenResponse>, ApiError> {
    let secret = ctx.secrets_client.get_secret(CONFIG_SECRET_KEY_ID).await?;
    let key: Hmac<Sha256> =
        Hmac::new_from_slice(secret.as_bytes()).map_err(|_| ApiError::Unauthorized)?;

    let unverified: Token<Header, JwtClaim, jwt::Unverified<'_>> =
        Token::parse_unverified(&request.token)?;
    let token: Token<_, _, jwt::Verified> = unverified.verify_with_key(&key)?;

    let username = token.claims().username.to_owned();
    let (_, refresh_token) = ctx.database.get_user_tokens(username.to_owned()).await?;
    log::info!("refresh token for {username}");

    // the token is verified and the refresh token matches the last one created
    log::info!(
        "saved: {refresh_token} compared to provided: {}",
        request.refresh_token
    );
    if refresh_token == request.refresh_token {
        log::info!("token matches last created refresh and access, generating new ones");
        let user_id = ctx.database.get_user_id(username.to_owned()).await?;
        let role = ctx.database.get_user_role(username.to_owned()).await?;

        let new_jwt = generate_signed_jwt(
            secret,
            &user_id,
            &ctx.config.api.jwt.application_id,
            &role,
            &username,
        )?;

        let refresh_token = uuid::Uuid::new_v4().as_hyphenated().to_string();

        ctx.database
            .set_user_tokens(
                &username,
                &new_jwt,
                &refresh_token,
                chrono::Duration::days(7),
            )
            .await?;

        Ok(Json(TokenResponse {
            token: new_jwt,
            refresh_token,
            role,
        }))
    } else {
        Err(ApiError::Unauthorized)
    }
}
