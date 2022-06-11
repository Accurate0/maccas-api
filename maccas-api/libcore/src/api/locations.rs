use crate::{client, config::ApiConfig, dispatcher::Executor};
use async_trait::async_trait;
use http::Response;
use lambda_http::{Body, Error, IntoResponse, Request, RequestExt};
use rand::{
    prelude::{SliceRandom, StdRng},
    SeedableRng,
};

pub struct Locations;

#[async_trait]
impl Executor for Locations {
    async fn execute(
        &self,
        request: &Request,
        dynamodb_client: &aws_sdk_dynamodb::Client,
        config: &ApiConfig,
    ) -> Result<Response<Body>, Error> {
        let query_params = request.query_string_parameters();
        let distance = query_params.first("distance");
        let latitude = query_params.first("latitude");
        let longitude = query_params.first("longitude");

        if distance.is_some() && latitude.is_some() && longitude.is_some() {
            // TODO: use a service account
            let account_name_list: Vec<String> = config.users.iter().map(|u| u.account_name.clone()).collect();
            let mut rng = StdRng::from_entropy();
            let choice = account_name_list.choose(&mut rng).unwrap().to_string();
            let user = config.users.iter().find(|u| u.account_name == choice).unwrap();

            let http_client = client::get_http_client();
            let api_client = client::get(
                &http_client,
                &dynamodb_client,
                &choice,
                &config,
                &user.login_username,
                &user.login_password,
            )
            .await?;
            let resp = api_client
                .restaurant_location(distance, latitude, longitude, None)
                .await?;

            Ok(serde_json::to_string(&resp).unwrap().into_response())
        } else {
            Ok(Response::builder().status(400).body("".into()).unwrap())
        }
    }
}
