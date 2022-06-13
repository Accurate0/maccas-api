use super::Context;
use crate::cache;
use crate::client;
use crate::constants::MCDONALDS_API_DEFAULT_OFFSET;
use crate::constants::MCDONALDS_API_DEFAULT_STORE_ID;
use async_trait::async_trait;
use http::Response;
use lambda_http::{Body, IntoResponse, Request, RequestExt};
use simple_dispatcher::Executor;
use simple_dispatcher::ExecutorResult;

pub struct Code;

#[async_trait]
impl Executor<Context, Request, Response<Body>> for Code {
    async fn execute(&self, ctx: &Context, request: &Request) -> ExecutorResult<Response<Body>> {
        let path_params = request.path_parameters();
        let query_params = request.query_string_parameters();

        let store = query_params.first("store");
        let deal_id = path_params.first("dealId").ok_or("must have id")?;
        let deal_id = &deal_id.to_owned();

        let (account_name, _offer) =
            cache::get_offer_by_id(deal_id, &ctx.dynamodb_client, &ctx.config.cache_table_name_v2).await?;
        let user = ctx
            .config
            .users
            .iter()
            .find(|u| u.account_name == account_name)
            .ok_or("no account found")?;

        let http_client = client::get_http_client();
        let api_client = client::get(
            &http_client,
            &ctx.dynamodb_client,
            &account_name,
            &ctx.config,
            &user.login_username,
            &user.login_password,
        )
        .await?;

        let resp = api_client
            .get_offers_dealstack(
                MCDONALDS_API_DEFAULT_OFFSET,
                store.unwrap_or(MCDONALDS_API_DEFAULT_STORE_ID),
            )
            .await?;
        Ok(serde_json::to_string(&resp).unwrap().into_response())
    }
}
