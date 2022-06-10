use crate::types::jwt::JwtClaim;
use crate::{client, client::get_correlation_id, config::ApiConfig, constants, routes::Route};
use async_trait::async_trait;
use http::Response;
use jwt::{Header, Token};
use lambda_http::{Body, Error, Request};

pub struct UserConfig;

#[async_trait]
impl Route for UserConfig {
    async fn execute(
        request: &Request,
        _dynamodb_client: &aws_sdk_dynamodb::Client,
        config: &ApiConfig,
    ) -> Result<Response<Body>, Error> {
        let correlation_id = get_correlation_id(&request);
        let auth_header = request.headers().get(http::header::AUTHORIZATION);
        Ok(match auth_header {
            Some(h) => {
                let value = h.to_str().unwrap().replace("Bearer ", "");
                let jwt: Token<Header, JwtClaim, _> = jwt::Token::parse_unverified(&value).unwrap();
                let user_id = &jwt.claims().oid;
                let http_client = client::get_http_client();
                let body = request.body().clone();
                let body = match body {
                    lambda_http::Body::Text(s) => s,
                    _ => String::new(),
                };

                let response = http_client
                    .request(
                        request.method().clone(),
                        format!(
                            "{}/{}{}",
                            constants::KVP_API_BASE,
                            constants::MACCAS_WEB_API_PREFIX,
                            user_id
                        )
                        .as_str(),
                    )
                    .body(body)
                    .header(constants::CORRELATION_ID_HEADER, correlation_id)
                    .header(constants::X_API_KEY_HEADER, &config.api_key)
                    .send()
                    .await
                    .unwrap();

                Response::builder()
                    .status(response.status())
                    .body(response.text().await?.into())
                    .unwrap()
            }
            None => Response::builder().status(401).body("".into()).unwrap(),
        })
    }
}
