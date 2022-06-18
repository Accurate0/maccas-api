use super::Context;
use crate::{client, constants::MCDONALDS_API_DEFAULT_FILTER, types::api::RestaurantInformation};
use async_trait::async_trait;
use http::Response;
use lambda_http::{Body, IntoResponse, Request, RequestExt};
use rand::{
    prelude::{SliceRandom, StdRng},
    SeedableRng,
};
use simple_dispatcher::{Executor, ExecutorResult};

pub struct Locations;

#[async_trait]
impl Executor<Context, Request, Response<Body>> for Locations {
    async fn execute(&self, ctx: &Context, request: &Request) -> ExecutorResult<Response<Body>> {
        let query_params = request.query_string_parameters();
        let distance = query_params.first("distance");
        let latitude = query_params.first("latitude");
        let longitude = query_params.first("longitude");

        if distance.is_some() && latitude.is_some() && longitude.is_some() {
            let distance = distance.unwrap();
            let latitude = latitude.unwrap();
            let longitude = longitude.unwrap();
            // TODO: use a service account
            let account_name_list: Vec<String> = ctx.config.users.iter().map(|u| u.account_name.clone()).collect();
            let mut rng = StdRng::from_entropy();
            let choice = account_name_list.choose(&mut rng).ok_or("no choice")?.to_string();
            let user = ctx
                .config
                .users
                .iter()
                .find(|u| u.account_name == choice)
                .ok_or("no account")?;

            let http_client = client::get_http_client();
            let api_client = client::get(
                &http_client,
                &ctx.dynamodb_client,
                &choice,
                &ctx.config,
                &user.login_username,
                &user.login_password,
            )
            .await?;
            let resp = api_client
                .restaurant_location(distance, latitude, longitude, MCDONALDS_API_DEFAULT_FILTER)
                .await?;

            let mut location_list = Vec::new();
            let response = resp.response;
            if let Some(response) = response {
                for restaurant in response.restaurants {
                    location_list.push(RestaurantInformation::from(restaurant));
                }
            }

            Ok(serde_json::to_value(&location_list)?.into_response())
        } else {
            Ok(Response::builder().status(400).body("".into())?)
        }
    }
}
