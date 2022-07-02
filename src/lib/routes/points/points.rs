use async_trait::async_trait;
use http::Response;
use lambda_http::{Body, IntoResponse, Request};
use simple_dispatcher::{Executor, ExecutorResult};

use crate::{db, routes::Context, types::api::AccountPointResponse};

pub struct Points;

#[async_trait]
impl Executor<Context, Request, Response<Body>> for Points {
    async fn execute(&self, ctx: &Context, _request: &Request) -> ExecutorResult<Response<Body>> {
        let point_map = db::get_point_map(&ctx.dynamodb_client, &ctx.config.point_table_name).await?;
        Ok(serde_json::to_value(AccountPointResponse::from(point_map))?.into_response())
    }
}
