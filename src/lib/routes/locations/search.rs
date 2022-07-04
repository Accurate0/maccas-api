use crate::constants::{api_base, mc_donalds, LOCATION_SEARCH_DISTANCE};
use crate::extensions::RequestExtensions;
use crate::routes::Context;
use crate::types::{api::RestaurantInformation, places::PlaceResponse};
use crate::{
    client::{self},
    constants,
};
use async_trait::async_trait;
use http::Response;
use lambda_http::{Body, IntoResponse, Request, RequestExt};
use simple_dispatcher::{Executor, ExecutorResult};

pub struct Search;

#[async_trait]
impl Executor<Context<'_>, Request, Response<Body>> for Search {
    async fn execute(&self, ctx: &Context, request: &Request) -> ExecutorResult<Response<Body>> {
        let correlation_id = request.get_correlation_id();
        let query_params = request.query_string_parameters();
        let text = query_params.first("text").expect("must have text");
        let http_client = client::get_http_client();

        let response = http_client
            .request(
                request.method().clone(),
                format!("{}/place?text={}", api_base::PLACES, text,).as_str(),
            )
            .header(constants::CORRELATION_ID_HEADER, correlation_id)
            .header(constants::X_API_KEY_HEADER, &ctx.config.api_key)
            .send()
            .await?
            .json::<PlaceResponse>()
            .await?;

        let api_client = ctx
            .database
            .get_specific_client(
                &http_client,
                &ctx.config.client_id,
                &ctx.config.client_secret,
                &ctx.config.sensor_data,
                &ctx.config.service_account,
            )
            .await?;
        let response = response.result;

        Ok(match response {
            Some(response) => {
                let lat = response.geometry.location.lat;
                let lng = response.geometry.location.lng;
                let resp = api_client
                    .restaurant_location(&LOCATION_SEARCH_DISTANCE, &lat, &lng, mc_donalds::default::FILTER)
                    .await?;

                match resp.body.response {
                    Some(list) => match list.restaurants.first() {
                        Some(res) => serde_json::to_value(RestaurantInformation::from(res.clone()))?.into_response(),
                        None => Response::builder().status(404).body(Body::Empty)?,
                    },
                    None => Response::builder().status(404).body(Body::Empty)?,
                }
            }
            None => Response::builder().status(404).body(Body::Empty)?,
        })
    }
}
