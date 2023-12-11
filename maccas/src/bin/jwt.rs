use anyhow::Result;
use chrono::Utc;
use foundation::aws;
use foundation::extensions::SecretsManagerExtensions;
use hmac::digest::KeyInit;
use hmac::Hmac;
use jwt::Header;
use jwt::Token;
use jwt::VerifyWithKey;
use lambda_http::service_fn;
use lambda_http::Error as LambdaError;
use lambda_runtime::LambdaEvent;
use maccas::constants::config::CONFIG_SECRET_KEY_ID;
use maccas::logging;
use maccas::types::config::GeneralConfig;
use maccas::types::token::JwtClaim;
use maccas::types::token::LambdaAuthorizerPayload;
use maccas::types::token::LambdaAuthorizerResponse;
use sha2::Sha256;

macro_rules! return_jwt_unauthorized {
    ($res:expr) => {
        match $res {
            Ok(val) => val,
            Err(e) => {
                log::warn!("return unauthorized: {}", e);
                return Ok(LambdaAuthorizerResponse {
                    is_authorized: false,
                });
            }
        }
    };
}

#[tokio::main]
async fn main() -> Result<(), LambdaError> {
    foundation::log::init_logger();
    logging::dump_build_details();
    lambda_runtime::run(service_fn(run)).await?;
    Ok(())
}

async fn run(
    event: LambdaEvent<LambdaAuthorizerPayload>,
) -> Result<LambdaAuthorizerResponse, anyhow::Error> {
    let shared_config = aws::config::get_shared_config().await;
    let config = GeneralConfig::load(&shared_config).await?;
    let secrets_client = aws_sdk_secretsmanager::Client::new(&shared_config);

    let token = event.payload.headers.authorization.replace("Bearer ", "");
    let secret = return_jwt_unauthorized!(secrets_client.get_secret(CONFIG_SECRET_KEY_ID).await);
    let key: Hmac<Sha256> = return_jwt_unauthorized!(Hmac::new_from_slice(secret.as_bytes()));

    log::info!("checking token {:?}", token);

    let unverified: Token<Header, JwtClaim, jwt::Unverified<'_>> =
        return_jwt_unauthorized!(Token::parse_unverified(&token));

    // verify token
    let token: Token<_, _, jwt::Verified> =
        return_jwt_unauthorized!(unverified.verify_with_key(&key));

    let claims = token.claims();

    log::info!("validated token with claims: {:?}", claims);

    let now = Utc::now().timestamp();
    let is_authorized = claims.aud == config.api.jwt.application_id && now <= claims.exp;

    Ok(LambdaAuthorizerResponse { is_authorized })
}
