use crate::types::jwt::JwtClaim;
use crate::types::user::UserOptions;
use crate::{routes::Context, types::api::Error};
use async_trait::async_trait;
use http::{Method, Response, StatusCode};
use jwt::{Header, Token};
use lambda_http::{Body, IntoResponse, Request};
use simple_dispatcher::{Executor, ExecutorResult};

pub struct Config;

pub mod docs {
    #[utoipa::path(
        get,
        path = "/user/config",
        responses(
            (status = 200, description = "Config for current user", body = UserOptions),
            (status = 404, description = "No config for this user", body = Error),
            (status = 500, description = "Internal Server Error", body = Error),
        ),
        tag = "config",
    )]
    pub fn get_user_config() {}

    #[utoipa::path(
        post,
        path = "/user/config",
        request_body(
            content = UserOptions,
            content_type = "application/json",
        ),
        responses(
            (status = 204, description = "Updated/created config"),
            (status = 400, description = "Invalid configuration format", body = Error),
            (status = 500, description = "Internal Server Error", body = Error),
        ),
        tag = "config",
    )]
    pub fn update_user_config() {}
}

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
                        Err(_) => {
                            let status_code = StatusCode::NOT_FOUND;
                            Response::builder().status(status_code.as_u16()).body(
                                serde_json::to_string(&Error {
                                    message: status_code.canonical_reason().ok_or("no value")?.to_string(),
                                })?
                                .into(),
                            )?
                        }
                    },

                    Method::POST => match serde_json::from_str::<UserOptions>(&body) {
                        Ok(ref new_config) => {
                            ctx.database
                                .set_config_by_user_id(user_id, new_config, user_name)
                                .await?;
                            Response::builder().status(204).body(Body::Empty)?
                        }
                        Err(_) => {
                            let status_code = StatusCode::BAD_REQUEST;
                            Response::builder().status(status_code.as_u16()).body(
                                serde_json::to_string(&Error {
                                    message: status_code.canonical_reason().ok_or("no value")?.to_string(),
                                })?
                                .into(),
                            )?
                        }
                    },

                    _ => Response::builder().status(405).body(Body::Empty)?,
                }
            }
            None => Response::builder().status(401).body(Body::Empty)?,
        })
    }
}
