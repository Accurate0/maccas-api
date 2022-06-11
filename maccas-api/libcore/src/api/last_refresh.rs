use crate::dispatcher::Executor;
use crate::types::api::LastRefreshInformation;
use crate::{cache, config::ApiConfig};
use async_trait::async_trait;
use http::Response;
use lambda_http::{Body, Error, IntoResponse, Request};

pub struct LastRefresh;

#[async_trait]
impl Executor for LastRefresh {
    async fn execute(
        &self,
        _request: &Request,
        dynamodb_client: &aws_sdk_dynamodb::Client,
        config: &ApiConfig,
    ) -> Result<Response<Body>, Error> {
        let response = cache::get_refresh_time_for_offer_cache(&dynamodb_client, &config.cache_table_name).await?;
        let response = LastRefreshInformation { last_refresh: response };

        Ok(serde_json::to_string(&response)?.into_response())
    }
}
