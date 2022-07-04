use crate::routes::Context;
use crate::types::jwt::JwtClaim;
use crate::types::user::UserOptions;
use async_trait::async_trait;
use http::{Method, Response};
use jwt::{Header, Token};
use lambda_http::{Body, IntoResponse, Request};
use simple_dispatcher::{Executor, ExecutorResult};

pub struct Config;

#[async_trait]
impl Executor<Context<'_>, Request, Response<Body>> for Config {
    async fn execute(&self, ctx: &Context, request: &Request) -> ExecutorResult<Response<Body>> {
        let auth_header = request.headers().get(http::header::AUTHORIZATION);
        Ok(match auth_header {
            Some(h) => {
                let value = h.to_str()?.replace("Bearer ", "");
                let jwt: Token<Header, JwtClaim, _> = jwt::Token::parse_unverified(&value)?;
                let user_id = &jwt.claims().oid;
                let user_name = &jwt.claims().name;
                let body = match request.body() {
                    lambda_http::Body::Text(s) => s.clone(),
                    _ => String::new(),
                };

                match *request.method() {
                    Method::GET => match ctx.database.get_config_by_user_id(user_id).await {
                        Ok(config) => serde_json::to_value(&config)?.into_response(),
                        Err(_) => Response::builder().status(404).body(Body::Empty)?,
                    },

                    Method::POST => match serde_json::from_str::<UserOptions>(&body) {
                        Ok(ref new_config) => {
                            ctx.database
                                .set_config_by_user_id(user_id, new_config, user_name)
                                .await?;
                            Response::builder().status(204).body(Body::Empty)?
                        }
                        Err(_) => Response::builder().status(400).body(Body::Empty)?,
                    },

                    _ => Response::builder().status(405).body(Body::Empty)?,
                }
            }
            None => Response::builder().status(401).body(Body::Empty)?,
        })
    }
}
