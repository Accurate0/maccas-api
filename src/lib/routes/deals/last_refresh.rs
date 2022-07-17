use crate::routes::Context;
use crate::types::api::LastRefreshInformation;
use async_trait::async_trait;
use http::Response;
use lambda_http::{Body, IntoResponse, Request};
use simple_dispatcher::{Executor, ExecutorResult};

pub struct LastRefresh;

pub mod docs {
    #[utoipa::path(
        get,
        path = "/deals/last-refresh",
        responses(
            (status = 200, description = "Last Refresh of Cache", body = LastRefreshInformation),
            (status = 500, description = "Internal Server Error", body = Error),
        ),
        tag = "deals",
    )]
    pub fn last_refresh() {}
}

#[async_trait]
impl Executor<Context<'_>, Request, Response<Body>> for LastRefresh {
    async fn execute(&self, ctx: &Context, _request: &Request) -> ExecutorResult<Response<Body>> {
        let response = ctx.database.get_refresh_time_for_offer_cache().await?;
        let response = LastRefreshInformation { last_refresh: response };

        Ok(serde_json::to_value(&response)?.into_response())
    }
}
