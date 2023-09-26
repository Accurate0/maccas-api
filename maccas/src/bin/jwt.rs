use anyhow::Result;
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

#[tokio::main]
async fn main() -> Result<(), LambdaError> {
    foundation::log::init_logger(log::LevelFilter::Info, &[]);
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
    let secret = secrets_client.get_secret(CONFIG_SECRET_KEY_ID).await?;
    let key: Hmac<Sha256> = Hmac::new_from_slice(secret.as_bytes())?;
    log::info!("checking token {:?}", token);

    let unverified: Token<Header, JwtClaim, jwt::Unverified<'_>> = Token::parse_unverified(&token)?;
    let token: Token<_, _, jwt::Verified> = unverified.verify_with_key(&key)?;
    log::info!("validated token with claims: {:?}", token.claims());

    // TODO: check expiry etc

    Ok(LambdaAuthorizerResponse {
        is_authorized: token.claims().aud == config.api.jwt.application_id,
    })
}
