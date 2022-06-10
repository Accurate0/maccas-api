use crate::config::ApiConfig;
use async_trait::async_trait;
use aws_sdk_dynamodb::Client;
use lambda_http::{Body, Error, Request, Response};

#[async_trait]
pub trait Route {
    async fn execute(
        request: &Request,
        dynamodb_client: &Client,
        config: &ApiConfig,
    ) -> Result<Response<Body>, Error>;
}
