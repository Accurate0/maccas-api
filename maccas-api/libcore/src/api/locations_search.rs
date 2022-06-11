use crate::extensions::RequestExtensions;
use crate::types::{api::RestaurantInformation, places::PlaceResponse};
use crate::{
    client::{self},
    config::ApiConfig,
    constants,
    dispatcher::Executor,
};
use async_trait::async_trait;
use http::Response;
use lambda_http::{Body, Error, Request, RequestExt};
use rand::{
    prelude::{SliceRandom, StdRng},
    SeedableRng,
};

pub struct LocationsSearch;

#[async_trait]
impl Executor for LocationsSearch {
    async fn execute(
        &self,
        request: &Request,
        dynamodb_client: &aws_sdk_dynamodb::Client,
        config: &ApiConfig,
    ) -> Result<Response<Body>, Error> {
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
            .header(constants::X_API_KEY_HEADER, &config.api_key)
            .send()
            .await
            .unwrap()
            .json::<PlaceResponse>()
            .await
            .unwrap();

        // TODO: use a service account
        let account_name_list: Vec<String> = config.users.iter().map(|u| u.account_name.clone()).collect();
        let mut rng = StdRng::from_entropy();
        let choice = account_name_list.choose(&mut rng).unwrap().to_string();
        let user = config.users.iter().find(|u| u.account_name == choice).unwrap();

        let api_client = client::get(
            &http_client,
            &dynamodb_client,
            &choice,
            &config,
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
                            .body(
                                serde_json::to_string(&RestaurantInformation::from(res.clone()))
                                    .unwrap()
                                    .into(),
                            )
                            .unwrap(),
                        None => Response::builder().status(404).body("".into()).unwrap(),
                    },
                    None => Response::builder().status(404).body("".into()).unwrap(),
                }
            }
            None => Response::builder().status(404).body("".into()).unwrap(),
        })
    }
}
