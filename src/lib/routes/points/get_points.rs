use crate::{routes::Context, types::api::AccountPointResponse};
use async_trait::async_trait;
use http::Response;
use lambda_http::{Body, IntoResponse, Request};
use simple_dispatcher::{Executor, ExecutorResult};

pub struct Points;

#[async_trait]
impl Executor<Context<'_>, Request, Response<Body>> for Points {
    async fn execute(&self, ctx: &Context, _request: &Request) -> ExecutorResult<Response<Body>> {
        let point_map = ctx.database.get_point_map().await?;
        Ok(serde_json::to_value(AccountPointResponse::from(point_map))?.into_response())
    }
}
