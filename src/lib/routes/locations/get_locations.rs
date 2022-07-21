use crate::routes::Context;
use crate::{client, constants::mc_donalds, types::api::RestaurantInformation};
use async_trait::async_trait;
use http::Response;
use lambda_http::{Body, IntoResponse, Request, RequestExt};
use simple_dispatcher::{Executor, ExecutorResult};

pub struct Locations;

pub mod docs {
    #[utoipa::path(
        get,
        path = "/locations",
        responses(
            (status = 200, description = "List of locations near specified coordinates", body = [RestaurantInformation]),
            (status = 400, description = "Invalid/missing parameters"),
            (status = 500, description = "Internal Server Error", body = Error),
        ),
        params(
            ("distance" = f64, query, description = "search distance"),
            ("latitude" = f64, query, description = "latitude"),
            ("longitude" = f64, query, description = "longitude"),
        ),
        tag = "location",
    )]
    pub fn get_locations() {}
}

#[async_trait]
impl Executor<Context<'_>, Request, Response<Body>> for Locations {
    async fn execute(&self, ctx: &Context, request: &Request) -> ExecutorResult<Response<Body>> {
        let query_params = request.query_string_parameters();
        let distance = query_params.first("distance");
        let latitude = query_params.first("latitude");
        let longitude = query_params.first("longitude");

        if let (Some(distance), Some(latitude), Some(longitude)) = (distance, latitude, longitude) {
            let http_client = client::get_http_client();
            let api_client = ctx
                .database
                .get_specific_client(
                    &http_client,
                    &ctx.config.client_id,
                    &ctx.config.client_secret,
                    &ctx.config.sensor_data,
                    &ctx.config.service_account,
                    false,
                )
                .await?;
            let resp = api_client
                .restaurant_location(distance, latitude, longitude, mc_donalds::default::FILTER)
                .await?;

            let mut location_list = Vec::new();
            let response = resp.body.response;
            if let Some(response) = response {
                for restaurant in response.restaurants {
                    location_list.push(RestaurantInformation::from(restaurant));
                }
            }

            Ok(serde_json::to_value(&location_list)?.into_response())
        } else {
            Ok(Response::builder().status(400).body(Body::Empty)?)
        }
    }
}
