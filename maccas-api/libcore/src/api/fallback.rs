use super::Context;
use async_trait::async_trait;
use http::Response;
use lambda_http::{Body, Error, Request};
use simple_dispatcher::Executor;

pub struct Fallback;

#[async_trait]
impl Executor<Context, Request, Response<Body>> for Fallback {
    async fn execute(&self, _request: &Request, _ctx: &Context) -> Result<Response<Body>, Error> {
        Ok(Response::builder().status(404).body("".into())?)
    }
}
