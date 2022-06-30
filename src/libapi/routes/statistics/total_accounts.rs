use crate::{db, routes::Context, types::api::TotalAccountsResponse};
use async_trait::async_trait;
use http::Response;
use lambda_http::{Body, IntoResponse, Request};
use simple_dispatcher::{Executor, ExecutorResult};

pub struct TotalAccounts;

#[async_trait]
impl Executor<Context, Request, Response<Body>> for TotalAccounts {
    async fn execute(&self, ctx: &Context, _request: &Request) -> ExecutorResult<Response<Body>> {
        let offers = db::get_all_offers_as_map(&ctx.dynamodb_client, &ctx.config.cache_table_name).await?;
        Ok(serde_json::to_value(&TotalAccountsResponse(offers.len() as i64))?.into_response())
    }
}
