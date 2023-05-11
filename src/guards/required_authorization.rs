use aliri_oauth2::Authority;
use foundation::{extensions::AliriOAuth2Extensions, types::jwt::JwtClaim};
use jwt::{Header, Token};
use rocket::{
    http::hyper::header,
    http::Status,
    outcome::Outcome,
    request::{self, FromRequest},
    Request, State,
};

const UNAUTHORIZED_MESSAGE: &str = "Unauthorized";

pub struct RequiredAuthorizationHeader {
    pub claims: JwtClaim,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for RequiredAuthorizationHeader {
    type Error = anyhow::Error;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let auth = request.headers().get_one(header::AUTHORIZATION.as_str());
        let authority = request.guard::<&State<Authority>>().await;

        match auth {
            Some(auth) => {
                let token = auth.replace("Bearer ", "");
                match authority {
                    Outcome::Success(authority) => {
                        let jwt_result = authority.verify_token_from_str::<JwtClaim>(&token);
                        match jwt_result {
                            Ok(jwt) => {
                                log::info!("verified jwt for {}", jwt.oid);
                                Outcome::Success(Self { claims: jwt })
                            }
                            Err(e) => {
                                log::error!("error validating jwt: {e}");
                                Outcome::Failure((
                                    Status::Unauthorized,
                                    anyhow::Error::msg(UNAUTHORIZED_MESSAGE),
                                ))
                            }
                        }
                    }

                    _ => {
                        let jwt: Result<Token<Header, JwtClaim, _>, _> =
                            jwt::Token::parse_unverified(&token);
                        match jwt {
                            Ok(jwt) => {
                                log::info!(
                                    "no authority found, unverified jwt for {}",
                                    jwt.claims().oid
                                );
                                Outcome::Success(Self {
                                    claims: jwt.claims().clone(),
                                })
                            }
                            Err(_) => Outcome::Failure((
                                Status::Unauthorized,
                                anyhow::Error::msg(UNAUTHORIZED_MESSAGE),
                            )),
                        }
                    }
                }
            }
            None => Outcome::Failure((
                Status::Unauthorized,
                anyhow::Error::msg(UNAUTHORIZED_MESSAGE),
            )),
        }
    }
}
