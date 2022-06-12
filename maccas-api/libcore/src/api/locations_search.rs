use super::Context;
use crate::extensions::RequestExtensions;
use crate::types::{api::RestaurantInformation, places::PlaceResponse};
use crate::{
    client::{self},
    constants,
};
use async_trait::async_trait;
use http::Response;
use lambda_http::{Body, Error, Request, RequestExt};
use rand::{
    prelude::{SliceRandom, StdRng},
    SeedableRng,
};
use simple_dispatcher::Executor;

pub struct LocationsSearch;

#[async_trait]
impl Executor<Context, Request, Response<Body>> for LocationsSearch {
    async fn execute(&self, request: &Request, ctx: &Context) -> Result<Response<Body>, Error> {
        let correlation_id = request.get_correlation_id();
        let query_params = request.query_string_parameters();
        let text = query_params.first("text").expect("must have text");
        let http_client = client::get_http_client();

        let response = http_client
            .request(
                request.method().clone(),
                format!("{}/place?text={}", constants::PLACES_API_BASE, text,).as_str(),
            )
            .header(constants::CORRELATION_ID_HEADER, correlation_id)
            .header(constants::X_API_KEY_HEADER, &ctx.api_config.api_key)
            .send()
            .await?
            .json::<PlaceResponse>()
            .await?;

        // TODO: use a service account
        let account_name_list: Vec<String> = ctx.api_config.users.iter().map(|u| u.account_name.clone()).collect();
        let mut rng = StdRng::from_entropy();
        let choice = account_name_list.choose(&mut rng).ok_or("no choice")?.to_string();
        let user = ctx
            .api_config
            .users
            .iter()
            .find(|u| u.account_name == choice)
            .ok_or("no account")?;

        let api_client = client::get(
            &http_client,
            &ctx.dynamodb_client,
            &choice,
            &ctx.api_config,
            &user.login_username,
            &user.login_password,
        )
        .await?;
        let response = response.result;

        Ok(match response {
            Some(response) => {
                let lat = response.geometry.location.lat;
                let lng = response.geometry.location.lng;
                let resp = api_client
                    .restaurant_location(
                        Some(&constants::LOCATION_SEARCH_DISTANCE.to_string()),
                        Some(&lat.to_string()),
                        Some(&lng.to_string()),
                        None,
                    )
                    .await?;

                match resp.response {
                    Some(list) => match list.restaurants.first() {
                        Some(res) => Response::builder()
                            .status(200)
                            .body(serde_json::to_string(&RestaurantInformation::from(res.clone()))?.into())?,
                        None => Response::builder().status(404).body("".into())?,
                    },
                    None => Response::builder().status(404).body("".into())?,
                }
            }
            None => Response::builder().status(404).body("".into())?,
        })
    }
}
