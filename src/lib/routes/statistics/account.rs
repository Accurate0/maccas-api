use crate::{routes::Context, types::api::AccountResponse};
use async_trait::async_trait;
use http::Response;
use lambda_http::{Body, IntoResponse, Request};
use simple_dispatcher::{Executor, ExecutorResult};

pub struct Account;

#[async_trait]
impl Executor<Context<'_>, Request, Response<Body>> for Account {
    async fn execute(&self, ctx: &Context, _request: &Request) -> ExecutorResult<Response<Body>> {
        let offers = ctx.database.get_all_offers_as_map().await?;
        Ok(serde_json::to_value(AccountResponse::from(offers))?.into_response())
    }
}
