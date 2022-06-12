use super::Context;
use crate::cache;
use crate::client;
use async_trait::async_trait;
use http::Response;
use lambda_http::{Body, Error, IntoResponse, Request, RequestExt};
use simple_dispatcher::Executor;

pub struct Code;

#[async_trait]
impl Executor<Context, Request, Response<Body>> for Code {
    async fn execute(&self, request: &Request, ctx: &Context) -> Result<Response<Body>, Error> {
        let path_params = request.path_parameters();
        let query_params = request.query_string_parameters();

        let store = query_params.first("store");
        let deal_id = path_params.first("dealId").ok_or("must have id")?;
        let deal_id = &deal_id.to_owned();

        let (account_name, _offer) =
            cache::get_offer_by_id(deal_id, &ctx.dynamodb_client, &ctx.api_config.cache_table_name_v2).await?;
        let user = ctx
            .api_config
            .users
            .iter()
            .find(|u| u.account_name == account_name)
            .ok_or("no account found")?;

        let http_client = client::get_http_client();
        let api_client = client::get(
            &http_client,
            &ctx.dynamodb_client,
            &account_name,
            &ctx.api_config,
            &user.login_username,
            &user.login_password,
        )
        .await?;

        let resp = api_client.offers_dealstack(None, store).await?;
        Ok(serde_json::to_string(&resp).unwrap().into_response())
    }
}
