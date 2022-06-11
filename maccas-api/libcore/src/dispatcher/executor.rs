use crate::config::ApiConfig;
use async_trait::async_trait;
use aws_sdk_dynamodb::Client;
use http::{Request, Response};
use lambda_http::{Body, Error};

#[async_trait]
pub trait Executor {
    async fn execute(
        &self,
        request: &Request<Body>,
        dynamodb_client: &Client,
        config: &ApiConfig,
    ) -> Result<Response<Body>, Error>;
}
