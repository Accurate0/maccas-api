use crate::{routes::Context, types::api::TotalAccountsResponse};
use async_trait::async_trait;
use http::Response;
use lambda_http::{Body, IntoResponse, Request};
use simple_dispatcher::{Executor, ExecutorResult};

pub struct TotalAccounts;

#[async_trait]
impl Executor<Context<'_>, Request, Response<Body>> for TotalAccounts {
    async fn execute(&self, ctx: &Context, _request: &Request) -> ExecutorResult<Response<Body>> {
        let offers = ctx.database.get_all_offers_as_map().await?;
        Ok(serde_json::to_value(&TotalAccountsResponse(offers.len() as i64))?.into_response())
    }
}
