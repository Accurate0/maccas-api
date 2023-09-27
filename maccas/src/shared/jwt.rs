use crate::{constants::config::TOKEN_VALID_TIME, types::token::JwtClaim};
use chrono::Utc;
use foundation::types::role::UserRole;
use hmac::{digest::KeyInit, Hmac};
use jwt::{AlgorithmType, Header, SignWithKey};
use sha2::Sha256;

pub fn generate_signed_jwt(
    secret_key: impl AsRef<[u8]>,
    user_id: &str,
    application_id: &str,
    role: &UserRole,
    username: &str,
) -> Result<String, anyhow::Error> {
    let dt = Utc::now();
    let key: Hmac<Sha256> = Hmac::new_from_slice(secret_key.as_ref())?;
    let timestamp: i64 = dt.timestamp() + TOKEN_VALID_TIME;
    let header = Header {
        algorithm: AlgorithmType::Hs256,
        ..Default::default()
    };

    let claims = JwtClaim {
        exp: timestamp,
        iss: "Maccas API".to_owned(),
        sub: user_id.to_owned(),
        aud: application_id.to_owned(),
        iat: Utc::now().timestamp(),
        oid: user_id.to_owned(),
        role: role.clone(),
        username: username.to_owned(),
    };

    Ok(jwt::Token::new(header, claims)
        .sign_with_key(&key)?
        .as_str()
        .to_owned())
}
