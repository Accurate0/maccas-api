use crate::{
    constants::config::CONFIG_SECRET_KEY_ID,
    routes,
    shared::jwt::generate_signed_jwt,
    types::{
        adb2c::Adb2cTokenResponse,
        api::{LoginRequest, TokenResponse},
        error::ApiError,
    },
};
use foundation::{extensions::SecretsManagerExtensions, types::jwt::Adb2cClaims};
use http::StatusCode;
use jwt::{Header, Token};
use rand::Rng;
use reqwest::multipart::Part;
use reqwest_tracing::TracingMiddleware;
use rocket::{serde::json::Json, State};

const ROPC_AUTH_PATH: &str = "https://apib2clogin.b2clogin.com/apib2clogin.onmicrosoft.com/B2C_1_ROPC_Auth/oauth2/v2.0/token";

#[utoipa::path(
    responses(
        (status = 200, description = "Login and fetch auth and refresh tokens", body = TokenResponse),
        (status = 401, description = "Account doesn't exist"),
        (status = 403, description = "Authentication failed"),
        (status = 500, description = "Internal Server Error"),
    ),
    tag = "auth",
)]
#[post("/auth/login", data = "<request>")]
pub async fn login(
    ctx: &State<routes::Context<'_>>,
    request: Json<LoginRequest>,
) -> Result<Json<TokenResponse>, ApiError> {
    if ctx
        .database
        .is_user_exist(request.username.to_owned())
        .await?
    {
        log::info!(
            "user: {} already exists, comparing hash and generating token",
            request.username
        );

        let password_hash = ctx
            .database
            .get_password_hash(request.username.clone())
            .await?;

        let is_password_correct = bcrypt::verify(request.password.as_bytes(), &password_hash)
            .map_err(|_| ApiError::Unauthorized)?;

        if is_password_correct {
            let user_id = ctx
                .database
                .get_user_id(request.username.to_owned())
                .await?;

            let role = ctx
                .database
                .get_user_role(request.username.to_owned())
                .await?;

            let secret = ctx.secrets_client.get_secret(CONFIG_SECRET_KEY_ID).await?;
            let new_jwt = generate_signed_jwt(
                secret,
                &user_id,
                &ctx.config.api.jwt.application_id,
                &role,
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

            return Ok(Json(TokenResponse {
                token: new_jwt,
                refresh_token,
                role,
            }));
        } else {
            return Err(ApiError::Unauthorized);
        }
    };

    let http_client = reqwest_middleware::ClientBuilder::new(reqwest::Client::new())
        .with(TracingMiddleware::default())
        .build();

    let form = reqwest::multipart::Form::new()
        .part("grant_type", Part::text("password"))
        .part(
            "client_id",
            Part::text(ctx.config.api.jwt.application_id.to_owned()),
        )
        .part(
            "scope",
            Part::text(format!(
                "openid {}",
                ctx.config.api.jwt.application_id.to_owned()
            )),
        )
        .part("username", Part::text(request.username.to_owned()))
        .part("password", Part::text(request.password.to_owned()))
        .part("response_type", Part::text("token id_token"));

    let resp = http_client
        .post(ROPC_AUTH_PATH)
        .multipart(form)
        .send()
        .await?
        .error_for_status()
        .map_err(|_| ApiError::Unauthorized)?;

    let status = resp.status();

    if status == StatusCode::OK {
        let token_response = resp.json::<Adb2cTokenResponse>().await?;

        let jwt: Result<Token<Header, Adb2cClaims, _>, _> =
            jwt::Token::parse_unverified(&token_response.id_token);

        match jwt {
            Ok(jwt) => {
                let claims = jwt.claims();
                let salt: [u8; 16] = rand::thread_rng().gen();
                let password_hash = bcrypt::hash_with_salt(request.password.clone(), 10, salt)
                    .map_err(|_| ApiError::InternalServerError)?;

                ctx.database
                    .create_user(
                        claims.oid.to_owned(),
                        request.username.to_owned(),
                        password_hash.to_string(),
                        salt.to_vec(),
                    )
                    .await?;

                let role = claims.extension_role.to_owned();
                ctx.database
                    .set_user_role(request.username.to_owned(), role.clone())
                    .await?;

                let secret = ctx.secrets_client.get_secret(CONFIG_SECRET_KEY_ID).await?;
                let new_jwt = generate_signed_jwt(
                    secret,
                    &claims.oid,
                    &ctx.config.api.jwt.application_id,
                    &role,
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
            Err(e) => {
                log::error!("error parsing token: {e}");
                Err(ApiError::Unauthorized)
            }
        }
    } else {
        Err(ApiError::Unauthorized)
    }
}
