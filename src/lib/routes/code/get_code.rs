use crate::client;
use crate::constants::mc_donalds;
use crate::routes::Context;
use crate::types::api::Error;
use crate::types::api::OfferResponse;
use async_trait::async_trait;
use http::Response;
use http::StatusCode;
use lambda_http::{Body, IntoResponse, Request, RequestExt};
use simple_dispatcher::Executor;
use simple_dispatcher::ExecutorResult;

pub struct Code;

pub mod docs {
    #[utoipa::path(
        get,
        path = "/code/{dealId}",
        responses(
            (status = 200, description = "Random code for specified deal", body = OfferResponse),
            (status = 404, description = "Deal not found", body = Error),
            (status = 500, description = "Internal Server Error", body = Error),
        ),
        params(
            ("dealId" = String, path, description = "The deal id to add"),
            ("store" = Option<i64>, query, description = "The selected store"),
        ),
        tag = "deals",
    )]
    pub fn get_code() {}
}

#[async_trait]
impl Executor<Context<'_>, Request, Response<Body>> for Code {
    async fn execute(&self, ctx: &Context, request: &Request) -> ExecutorResult<Response<Body>> {
        let path_params = request.path_parameters();
        let query_params = request.query_string_parameters();

        let store = query_params.first("store");
        let deal_id = path_params.first("dealId").ok_or("must have id")?;
        let deal_id = &deal_id.to_owned();

        if let Ok((account, _offer)) = ctx.database.get_offer_by_id(deal_id).await {
            let http_client = client::get_http_client();
            let api_client = ctx
                .database
                .get_specific_client(
                    &http_client,
                    &ctx.config.client_id,
                    &ctx.config.client_secret,
                    &ctx.config.sensor_data,
                    &account,
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
