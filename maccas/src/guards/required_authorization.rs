use foundation::extensions::SecretsManagerExtensions;
use hmac::{digest::KeyInit, Hmac};
use jwt::{Header, Token, VerifyWithKey};
use rocket::{
    http::hyper::header,
    http::Status,
    outcome::Outcome,
    request::{self, FromRequest},
    Request, State,
};
use sha2::Sha256;

use crate::{constants::config::CONFIG_SECRET_KEY_ID, routes::Context, types::token::JwtClaim};

const UNAUTHORIZED_MESSAGE: &str = "Unauthorized";

pub struct RequiredAuthorizationHeader {
    pub claims: JwtClaim,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for RequiredAuthorizationHeader {
    type Error = anyhow::Error;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let auth = request.headers().get_one(header::AUTHORIZATION.as_str());
        let ctx = request.guard::<&State<Context>>().await.unwrap();

        match auth {
            Some(auth) => {
                let auth = auth.replace("Bearer ", "");
                let secret = ctx
                    .secrets_client
                    .get_secret(CONFIG_SECRET_KEY_ID)
                    .await
                    .unwrap();
                let key: Hmac<Sha256> = Hmac::new_from_slice(secret.as_bytes()).unwrap();
                log::info!("checking token {:?}", auth);

                let unverified: Token<Header, JwtClaim, jwt::Unverified<'_>> =
                    Token::parse_unverified(&auth).unwrap();
                let token: Token<_, _, jwt::Verified> = unverified.verify_with_key(&key).unwrap();
                log::info!("validated token with claims: {:?}", token.claims());
                Outcome::Success(RequiredAuthorizationHeader {
                    claims: token.claims().clone(),
                })
            }
            None => Outcome::Error((
                Status::Unauthorized,
                anyhow::Error::msg(UNAUTHORIZED_MESSAGE),
            )),
        }
    }
}
