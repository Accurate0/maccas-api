use super::Context;
use crate::cache;
use crate::client;
use crate::constants::mc_donalds;
use crate::types::api::Error;
use crate::types::api::OfferResponse;
use async_trait::async_trait;
use http::Response;
use http::StatusCode;
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

        if let Ok((account_name, _offer)) =
            cache::get_offer_by_id(deal_id, &ctx.dynamodb_client, &ctx.config.cache_table_name_v2).await
        {
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
                    mc_donalds::default::OFFSET,
                    store.unwrap_or(mc_donalds::default::STORE_ID),
                )
                .await?;

            let resp = OfferResponse::from(resp.body);
            Ok(serde_json::to_value(&resp).unwrap().into_response())
        } else {
            let status_code = StatusCode::NOT_FOUND;
            Ok(Response::builder().status(status_code.as_u16()).body(
                serde_json::to_string(&Error {
                    message: status_code.canonical_reason().ok_or("no value")?.to_string(),
                })?
                .into(),
            )?)
        }
    }
}
