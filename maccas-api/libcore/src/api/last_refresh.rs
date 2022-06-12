use super::Context;
use crate::cache;
use crate::types::api::LastRefreshInformation;
use async_trait::async_trait;
use http::Response;
use lambda_http::{Body, Error, IntoResponse, Request};
use simple_dispatcher::Executor;

pub struct LastRefresh;

#[async_trait]
impl Executor<Context, Request, Response<Body>> for LastRefresh {
    async fn execute(&self, ctx: &Context, _request: &Request) -> Result<Response<Body>, Error> {
        let response =
            cache::get_refresh_time_for_offer_cache(&ctx.dynamodb_client, &ctx.api_config.cache_table_name).await?;
        let response = LastRefreshInformation { last_refresh: response };

        Ok(serde_json::to_string(&response)?.into_response())
    }
}
