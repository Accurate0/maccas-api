use crate::{
    client,
    constants::mc_donalds,
    db,
    routes::Context,
    types::api::{OfferPointsResponse, OfferResponse},
};
use async_trait::async_trait;
use http::Response;
use lambda_http::{Body, IntoResponse, Request, RequestExt};
use simple_dispatcher::{Executor, ExecutorResult};

pub struct GetById;

#[async_trait]
impl Executor<Context, Request, Response<Body>> for GetById {
    async fn execute(&self, ctx: &Context, request: &Request) -> ExecutorResult<Response<Body>> {
        let path_params = request.path_parameters();

        let account_id = path_params.first("accountId").expect("must have id");
        let account_id = &account_id.to_owned();

        Ok(
            if let Ok((account, points_response)) =
                db::get_points_by_account_hash(&ctx.dynamodb_client, &&ctx.config.point_table_name, &account_id).await
            {
                let query_params = request.query_string_parameters();
                let store = query_params.first("store");

                let http_client = client::get_http_client();
                let api_client = client::get(&http_client, &ctx.dynamodb_client, &ctx.config, &account).await?;

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
