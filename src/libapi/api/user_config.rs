use super::Context;
use crate::constants::api_base;
use crate::extensions::RequestExtensions;
use crate::types::jwt::JwtClaim;
use crate::{client, constants};
use async_trait::async_trait;
use http::Response;
use jwt::{Header, Token};
use lambda_http::{Body, Request};
use simple_dispatcher::{Executor, ExecutorResult};

pub struct UserConfig;

#[async_trait]
impl Executor<Context, Request, Response<Body>> for UserConfig {
    async fn execute(&self, ctx: &Context, request: &Request) -> ExecutorResult<Response<Body>> {
        // TODO: this is slow...
        let correlation_id = request.get_correlation_id();
        let auth_header = request.headers().get(http::header::AUTHORIZATION);
        Ok(match auth_header {
            Some(h) => {
                let value = h.to_str()?.replace("Bearer ", "");
                let jwt: Token<Header, JwtClaim, _> = jwt::Token::parse_unverified(&value)?;
                let user_id = &jwt.claims().oid;
                let http_client = client::get_http_client();
                let body = match request.body() {
                    lambda_http::Body::Text(s) => s.clone(),
                    _ => String::new(),
                };

                let response = http_client
                    .request(
                        request.method().clone(),
                        format!("{}/{}{}", api_base::KVP, constants::MACCAS_WEB_API_PREFIX, user_id).as_str(),
                    )
                    .body(body)
                    .header(constants::CORRELATION_ID_HEADER, correlation_id)
                    .header(constants::X_API_KEY_HEADER, &ctx.config.api_key)
                    .send()
                    .await?;

                Response::builder()
                    .status(response.status())
                    .body(response.text().await?.into())?
            }
            None => Response::builder().status(401).body(Body::Empty).unwrap(),
        })
    }
}
