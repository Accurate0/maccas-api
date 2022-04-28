use http_auth_basic::Credentials;
use rand::distributions::{Alphanumeric, DistString};
use rand::rngs::StdRng;
use rand::SeedableRng;
use reqwest::Method;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware, RequestBuilder};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use std::time::Duration;
use types::maccas::{
    LoginRefreshResponse, LoginResponse, OfferDealStackResponse, OfferDetailsResponse,
    OfferResponse, RestaurantLocationResponse, TokenResponse,
};
use uuid::Uuid;

const BASE_URL: &str = "https://ap-prod.api.mcd.com";

pub struct ApiClient {
    client: ClientWithMiddleware,
    auth_token: Option<String>,
    login_token: Option<String>,
    client_id: String,
    client_secret: String,
    login_username: String,
    login_password: String,
}

impl ApiClient {
    pub fn new(
        client_id: String,
        client_secret: String,
        login_username: String,
        login_password: String,
    ) -> ApiClient {
        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
        let client = ClientBuilder::new(
            reqwest::ClientBuilder::new()
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap(),
        )
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build();

        ApiClient {
            client,
            login_token: None,
            auth_token: None,
            client_id,
            client_secret,
            login_username,
            login_password,
        }
    }

    fn get_default_request(&self, resource: &str, method: Method) -> RequestBuilder {
        let client_id = &self.client_id;
        return self
            .client
            .request(method, format!("{BASE_URL}/{resource}"))
            .header("accept-encoding", "gzip")
            .header("accept-charset", "UTF-8")
            .header("accept-language", "en-AU")
            .header("content-type", "application/json; charset=UTF-8")
            .header("mcd-clientid", client_id)
            .header("mcd-uuid", Uuid::new_v4().to_hyphenated().to_string())
            .header("user-agent", "MCDSDK/20.0.14 (Android; 31; en-AU) GMA/6.2")
            .header("mcd-sourceapp", "GMA")
            .header("mcd-marketid", "AU");
    }

    pub fn set_auth_token(&mut self, auth_token: &String) {
        self.auth_token = Some(auth_token.to_string());
    }

    pub async fn security_auth_token(&mut self) -> reqwest_middleware::Result<TokenResponse> {
        let default_params = [("grantType", "client_credentials")];

        let client_id = &self.client_id;
        let client_secret = &self.client_secret;

        let credentials = Credentials::new(client_id, client_secret);

        let request = self
            .get_default_request("v1/security/auth/token", Method::POST)
            .query(&default_params)
            .header("authorization", credentials.as_http_header())
            .header("mcd-clientsecret", client_secret)
            .header(
                "content-type",
                "application/x-www-form-urlencoded; charset=UTF-8",
            );

        let response = request
            .send()
            .await?
            .json::<TokenResponse>()
            .await
            .expect("error deserializing payload");

        self.login_token = Some(response.response.token.clone());

        Ok(response)
    }

    pub async fn customer_login(&mut self) -> reqwest_middleware::Result<LoginResponse> {
        let token: &String = self.login_token.as_ref().unwrap();
        let login_username = &self.login_username;
        let login_password = &self.login_password;
        let mut rng = StdRng::from_entropy();
        let device_id = Alphanumeric.sample_string(&mut rng, 16);

        let creds = serde_json::json!({
            "credentials": {
                "loginUsername": login_username,
                "password": login_password,
                "type": "email"
            },
            "deviceId": device_id
        });

        let request = self
            .get_default_request("exp/v1/customer/login", Method::POST)
            .header("authorization", format!("Bearer {token}"))
            .header("x-acf-sensor-data", std::include_str!("sensor.data"))
            .json(&creds);

        let response = request
            .send()
            .await?
            .json::<LoginResponse>()
            .await
            .expect("error deserializing payload");

        self.auth_token = Some(response.response.access_token.clone());

        Ok(response)
    }

    // https://ap-prod.api.mcd.com/exp/v1/offers?distance=10000&exclude=14&latitude=-32.0117&longitude=115.8845&optOuts=&timezoneOffsetInMinutes=480
    pub async fn get_offers(
        &self,
        params: Option<Vec<(String, String)>>,
    ) -> reqwest_middleware::Result<OfferResponse> {
        let default_params = match params {
            Some(p) => p,
            None => Vec::from([
                (String::from("distance"), String::from("10000")),
                (String::from("latitude"), String::from("37.4219")),
                (String::from("longitude"), String::from("-122.084")),
                (String::from("optOuts"), String::from("")),
                (String::from("timezoneOffsetInMinutes"), String::from("480")),
            ]),
        };

        let token: &String = self.auth_token.as_ref().unwrap();

        let request = self
            .get_default_request("exp/v1/offers", Method::GET)
            .query(&default_params)
            .header("authorization", format!("Bearer {token}"));

        let response = request
            .send()
            .await?
            .json::<OfferResponse>()
            .await
            .expect("error deserializing payload");

        Ok(response)
    }

    // https://ap-prod.api.mcd.com/exp/v1/restaurant/location?distance=20&filter=summary&latitude=-32.0117&longitude=115.8845
    pub async fn restaurant_location(
        &self,
        distance: Option<&str>,
        latitude: Option<&str>,
        longitude: Option<&str>,
        filter: Option<&str>,
    ) -> reqwest_middleware::Result<RestaurantLocationResponse> {
        let params = Vec::from([
            (String::from("distance"), distance.unwrap_or("20")),
            (String::from("latitude"), latitude.unwrap_or("-32.0117")),
            (String::from("longitude"), longitude.unwrap_or("115.8845")),
            (String::from("filter"), filter.unwrap_or("summary")),
        ]);

        let token: &String = self.auth_token.as_ref().unwrap();

        let request = self
            .get_default_request("exp/v1/restaurant/location", Method::GET)
            .query(&params)
            .bearer_auth(token);

        let response = request
            .send()
            .await?
            .json::<RestaurantLocationResponse>()
            .await
            .expect("error deserializing payload");

        Ok(response)
    }

    // https://ap-prod.api.mcd.com/exp/v1/offers/details/166870
    pub async fn offer_details(
        &self,
        offer_id: &String,
    ) -> reqwest_middleware::Result<OfferDetailsResponse> {
        let token: &String = self.auth_token.as_ref().unwrap();

        let request = self
            .get_default_request(
                format!("exp/v1/offers/details/{offer_id}").as_str(),
                Method::GET,
            )
            .header("authorization", format!("Bearer {token}"));

        let response = request
            .send()
            .await?
            .json::<OfferDetailsResponse>()
            .await
            .expect("error deserializing payload");

        Ok(response)
    }

    // GET https://ap-prod.api.mcd.com/exp/v1/offers/dealstack?offset=480&storeId=951488
    pub async fn offers_dealstack(
        &self,
        offset: Option<&str>,
        store_id: Option<&str>,
    ) -> reqwest_middleware::Result<OfferDealStackResponse> {
        let token: &String = self.auth_token.as_ref().unwrap();
        let params = Vec::from([
            (String::from("offset"), offset.unwrap_or("480")),
            (String::from("storeId"), store_id.unwrap_or("951488")),
        ]);

        let request = self
            .get_default_request("exp/v1/offers/dealstack", Method::GET)
            .query(&params)
            .header("authorization", format!("Bearer {token}"));

        let response = request
            .send()
            .await?
            .json::<OfferDealStackResponse>()
            .await
            .expect("error deserializing payload");

        Ok(response)
    }

    // POST https://ap-prod.api.mcd.com/exp/v1/offers/dealstack/166870?offerId=1139347703&offset=480&storeId=951488
    pub async fn add_offer_to_offers_dealstack(
        &self,
        offer_id: &String,
        offset: Option<&str>,
        store_id: Option<&str>,
    ) -> reqwest_middleware::Result<OfferDealStackResponse> {
        let token: &String = self.auth_token.as_ref().unwrap();
        let store_id = store_id.unwrap_or("951488");
        let offset = offset.unwrap_or("480").to_string();

        let params = Vec::from([
            (String::from("offset"), offset.as_str()),
            (String::from("storeId"), store_id),
        ]);

        let request = self
            .get_default_request(
                format!("exp/v1/offers/dealstack/{offer_id}").as_str(),
                Method::POST,
            )
            .query(&params)
            .bearer_auth(token);

        let response = request
            .send()
            .await?
            .json::<OfferDealStackResponse>()
            .await
            .expect("error deserializing payload");

        Ok(response)
    }

    // DELETE https://ap-prod.api.mcd.com/exp/v1/offers/dealstack/offer/166870?offerId=1139347703&offset=480&storeId=951488
    pub async fn remove_offer_from_offers_dealstack(
        &self,
        offer_id: i64,
        offer_proposition_id: &String,
        offset: Option<i64>,
        store_id: Option<&str>,
    ) -> reqwest_middleware::Result<OfferDealStackResponse> {
        let store_id = store_id.unwrap_or("951488");
        let offer_id = offer_id.to_string();
        let offset = offset.unwrap_or(480).to_string();
        // the app sends a body, but this request works without it
        // but we're pretending to be the app :)
        let body = serde_json::json!(
            {
                "storeId": store_id,
                "offerId": offer_id,
                "offset": offset,
            }
        );

        let token: &String = self.auth_token.as_ref().unwrap();
        let params = Vec::from([
            (String::from("offerId"), offer_id.as_str()),
            (String::from("offset"), offset.as_str()),
            (String::from("storeId"), store_id),
        ]);

        let request = self
            .get_default_request(
                format!("exp/v1/offers/dealstack/offer/{offer_proposition_id}").as_str(),
                Method::DELETE,
            )
            .json(&body)
            .query(&params)
            .bearer_auth(token);

        let response = request
            .send()
            .await?
            .json::<OfferDealStackResponse>()
            .await
            .expect("error deserializing payload");

        Ok(response)
    }

    // https://ap-prod.api.mcd.com/exp/v1/customer/login/refresh
    pub async fn customer_login_refresh(
        &self,
        refresh_token: &String,
    ) -> reqwest_middleware::Result<LoginRefreshResponse> {
        let token: &String = self.auth_token.as_ref().unwrap();

        let body = serde_json::json!({ "refreshToken": refresh_token });

        let request = self
            .get_default_request("exp/v1/customer/login/refresh", Method::POST)
            .bearer_auth(token)
            .json(&body);

        let response = request
            .send()
            .await?
            .json::<LoginRefreshResponse>()
            .await
            .expect("error deserializing payload");

        Ok(response)
    }
}
