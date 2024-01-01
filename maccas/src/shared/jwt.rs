use crate::{
    constants::config::{TOKEN_ACCESS_ISS, TOKEN_VALID_TIME},
    types::{role::UserRole, token::JwtClaim},
};
use chrono::Utc;
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
        iss: TOKEN_ACCESS_ISS.to_owned(),
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

pub fn rotate_refresh_tokens(refresh_tokens: &mut Vec<String>, new_refresh_token: String) {
    if refresh_tokens.len() > 10 {
        refresh_tokens.swap_remove(0);
    };

    refresh_tokens.push(new_refresh_token);
}
