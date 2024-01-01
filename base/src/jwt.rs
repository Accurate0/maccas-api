use hmac::Hmac;
use hmac::Mac;
use jwt::Header;
use jwt::Token;
use jwt::VerifyWithKey;
use serde::Deserialize;
use serde::Serialize;
use sha2::digest::InvalidLength;
use sha2::Sha256;
use thiserror::Error;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JwtClaims {
    pub user_id: String,
    pub session_id: String,
    pub iat: i64,
    pub exp: i64,
    pub aud: String,
    pub iss: String,
    pub sub: String,
}

#[derive(Error, Debug)]
pub enum JwtValidationError {
    #[error("Key length error has occurred: `{0}`")]
    InvalidKeyLength(#[from] InvalidLength),
    #[error("Key length error has occurred: `{0}`")]
    JwtError(#[from] jwt::Error),
}

pub fn verify_jwt(secret: &[u8], unverified_token: &str) -> Result<JwtClaims, JwtValidationError> {
    let key: Hmac<Sha256> = Hmac::new_from_slice(secret)?;
    let unverified: Token<Header, JwtClaims, jwt::Unverified<'_>> =
        Token::parse_unverified(unverified_token)?;

    let token: Token<_, _, jwt::Verified> = unverified.verify_with_key(&key)?;

    Ok(token.claims().clone())
}