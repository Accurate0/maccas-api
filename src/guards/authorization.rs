use crate::{
    routes,
    types::{error::ApiError, jwt::JwtClaim},
};
use aliri_oauth2::ScopePolicy;
use http::header;
use jwt::{Header, Token};
use rocket::{
    http::Status,
    outcome::Outcome,
    request::{self, FromRequest},
    Request, State,
};
use std::convert::Infallible;

pub struct AuthorizationHeader(pub Option<String>);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthorizationHeader {
    type Error = Infallible;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let auth = request.headers().get_one(header::AUTHORIZATION.as_str());
        Outcome::Success(Self(auth.map(|s| s.to_string())))
    }
}

pub struct RequiredAuthorizationHeader {
    pub claims: JwtClaim,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for RequiredAuthorizationHeader {
    type Error = ApiError;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let auth = request.headers().get_one(header::AUTHORIZATION.as_str());
        let ctx = request.guard::<&State<routes::Context>>().await;
        if ctx.is_failure() {
            return Outcome::Failure((Status::InternalServerError, ApiError::InternalServerError));
        };

        let ctx = ctx.unwrap();
        let auth_outcome = match auth {
            Some(auth) => {
                let token = auth.replace("Bearer ", "");
                if let Some(authority) = &ctx.authority {
                    let jwt = aliri::jwt::Jwt::new(token);
                    let jwt_result =
                        authority.verify_token::<JwtClaim>(jwt.as_ref(), &ScopePolicy::allow_any());
                    match jwt_result {
                        Ok(jwt) => {
                            log::info!("verified jwt for {}", jwt.oid);
                            Outcome::Success(Self { claims: jwt })
                        }
                        Err(e) => {
                            log::error!("error validating jwt: {e}");
                            Outcome::Failure((Status::Unauthorized, ApiError::Unauthorized))
                        }
                    }
                } else {
                    let jwt: Token<Header, JwtClaim, _> =
                        jwt::Token::parse_unverified(&token).unwrap();
                    Outcome::Success(Self {
                        claims: jwt.claims().clone(),
                    })
                }
            }
            None => Outcome::Failure((Status::Unauthorized, ApiError::Unauthorized)),
        };

        auth_outcome
    }
}
