use crate::config::ApiConfig;
use crate::dispatcher::Executor;
use async_trait::async_trait;
use http::Response;
use lambda_http::{Body, Error, Request};

pub struct Fallback;

#[async_trait]
impl Executor for Fallback {
    async fn execute(
        &self,
        _request: &Request,
        _dynamodb_client: &aws_sdk_dynamodb::Client,
        _config: &ApiConfig,
    ) -> Result<Response<Body>, Error> {
        Ok(Response::builder().status(404).body("".into())?)
    }
}
