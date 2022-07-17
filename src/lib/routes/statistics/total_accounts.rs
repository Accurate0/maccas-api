use crate::{routes::Context, types::api::TotalAccountsResponse};
use async_trait::async_trait;
use http::Response;
use lambda_http::{Body, IntoResponse, Request};
use simple_dispatcher::{Executor, ExecutorResult};

pub struct TotalAccounts;

pub mod docs {
    #[utoipa::path(
        get,
        path = "/statistics/total-account",
        responses(
            (status = 200, description = "Total account count", body = i64),
            (status = 500, description = "Internal Server Error", body = Error),
        ),
        tag = "statistics",
    )]
    pub fn statistics_total_accounts() {}
}

#[async_trait]
impl Executor<Context<'_>, Request, Response<Body>> for TotalAccounts {
    async fn execute(&self, ctx: &Context, _request: &Request) -> ExecutorResult<Response<Body>> {
        let offers = ctx.database.get_all_offers_as_map().await?;
        Ok(serde_json::to_value(&TotalAccountsResponse(offers.len() as i64))?.into_response())
    }
}
