use crate::{
    client,
    constants::mc_donalds,
    routes::Context,
    types::api::{OfferPointsResponse, OfferResponse},
};
use async_trait::async_trait;
use http::Response;
use lambda_http::{Body, IntoResponse, Request, RequestExt};
use simple_dispatcher::{Executor, ExecutorResult};

pub struct GetById;

pub mod docs {
    #[utoipa::path(
        get,
        path = "/points/{accountId}",
        responses(
            (status = 200, description = "List of all account points", body = OfferResponse),
            (status = 500, description = "Internal Server Error", body = Error),
        ),
        params(
            ("accountId" = String, path, description = "The account id"),
            ("store" = Option<i64>, query, description = "The selected store"),
        ),
        tag = "points",
    )]
    pub fn get_points_by_id() {}
}

#[async_trait]
impl Executor<Context<'_>, Request, Response<Body>> for GetById {
    async fn execute(&self, ctx: &Context, request: &Request) -> ExecutorResult<Response<Body>> {
        let path_params = request.path_parameters();

        let account_id = path_params.first("accountId").expect("must have id");
        let account_id = &account_id.to_owned();

        Ok(
            if let Ok((account, points_response)) = ctx.database.get_points_by_account_hash(account_id).await {
                let query_params = request.query_string_parameters();
                let store = query_params.first("store");

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

                let response = api_client
                    .get_offers_dealstack(
                        mc_donalds::default::OFFSET,
                        store.unwrap_or(mc_donalds::default::STORE_ID),
                    )
                    .await?;

                let offer_response = OfferResponse::from(response.body);

                serde_json::to_value(OfferPointsResponse {
                    offer_response,
                    points_response,
                })?
                .into_response()
            } else {
                Response::builder().status(404).body(Body::Empty)?
            },
        )
    }
}
