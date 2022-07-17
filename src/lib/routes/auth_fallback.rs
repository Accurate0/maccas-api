use super::Context;
use async_trait::async_trait;
use http::{header::WWW_AUTHENTICATE, Response};
use lambda_http::{Body, Request};
use simple_dispatcher::{Executor, ExecutorResult};

pub struct AuthFallback;

#[async_trait]
impl Executor<Context<'_>, Request, Response<Body>> for AuthFallback {
    async fn execute(&self, _ctx: &Context, _request: &Request) -> ExecutorResult<Response<Body>> {
        Ok(Response::builder()
            .status(401)
            .header(WWW_AUTHENTICATE, "bearer")
            .body(Body::Empty)?)
    }
}
