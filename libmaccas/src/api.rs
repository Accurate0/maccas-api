use std::fmt::Display;

use crate::types::{
    LoginRefreshResponse, LoginResponse, OfferDealStackResponse, OfferDetailsResponse,
    OfferResponse, RestaurantLocationResponse, TokenResponse,
};
use crate::Response;
use rand::distributions::{Alphanumeric, DistString};
use rand::rngs::StdRng;
use rand::SeedableRng;
use reqwest::Method;
use reqwest_middleware::{ClientWithMiddleware, RequestBuilder};
use uuid::Uuid;

pub struct ApiClient<'a> {
    base_url: String,
    client: &'a ClientWithMiddleware,
    auth_token: Option<String>,
    login_token: Option<String>,
    client_id: String,
    client_secret: String,
    login_username: String,
    login_password: String,
}

impl<'a> ApiClient<'a> {
    pub fn new(
        base_url: String,
        client: &ClientWithMiddleware,
        client_id: String,
        client_secret: String,
        login_username: String,
        login_password: String,
    ) -> ApiClient {
        ApiClient {
            base_url,
            client,
            login_token: None,
            auth_token: None,
            client_id,
            client_secret,
            login_username,
            login_password,
        }
    }

    fn get_default_request(&'a self, resource: &str, method: Method) -> RequestBuilder {
        let ref client_id = self.client_id;
        let ref base_url = self.base_url;

        return self
            .client
            .request(method, format!("{base_url}/{resource}"))
            .header("accept-encoding", "gzip")
            .header("accept-charset", "UTF-8")
            .header("accept-language", "en-AU")
            .header("content-type", "application/json; charset=UTF-8")
            .header("mcd-clientid", client_id)
            .header("mcd-uuid", Self::get_uuid())
            .header("user-agent", "MCDSDK/20.0.14 (Android; 31; en-AU) GMA/6.2")
            .header("mcd-sourceapp", "GMA")
            .header("mcd-marketid", "AU");
    }

    pub fn get_uuid() -> String {
        Uuid::new_v4().to_hyphenated().to_string()
    }

    pub fn username(&self) -> &String {
        &self.login_username
    }

    pub fn set_login_token<S>(&mut self, login_token: S)
    where
        S: Display,
    {
        self.login_token = Some(login_token.to_string());
    }

    pub fn set_auth_token<S>(&mut self, auth_token: S)
    where
        S: Display,
    {
        self.auth_token = Some(auth_token.to_string());
    }

    pub async fn security_auth_token(&'a self) -> Response<TokenResponse> {
        let default_params = [("grantType", "client_credentials")];

        let request = self
            .get_default_request("v1/security/auth/token", Method::POST)
            .query(&default_params)
            .basic_auth(&self.client_id, Some(&self.client_secret))
            .header("mcd-clientsecret", &self.client_secret)
            .header(
                "content-type",
                "application/x-www-form-urlencoded; charset=UTF-8",
            );

        let response = request.send().await?.json::<TokenResponse>().await?;

        Ok(response)
    }

    pub async fn customer_login(&'a self) -> Response<LoginResponse> {
        let token = self.login_token.as_ref().ok_or("no login token set")?;
        let login_username = &self.login_username;
        let login_password = &self.login_password;
        let mut rng = StdRng::from_entropy();
        let device_id = Alphanumeric.sample_string(&mut rng, 16);

        let credentials = serde_json::json!({
            "credentials": {
                "loginUsername": login_username,
                "password": login_password,
                "type": "email"
            },
            "deviceId": device_id
        });

        let request = self
            .get_default_request("exp/v1/customer/login", Method::POST)
            .bearer_auth(token)
            .header("x-acf-sensor-data", std::include_str!("sensor.data"))
            .json(&credentials);

        let response = request.send().await?.json::<LoginResponse>().await?;

        Ok(response)
    }

    // https://ap-prod.api.mcd.com/exp/v1/offers?distance=10000&exclude=14&latitude=-32.0117&longitude=115.8845&optOuts=&timezoneOffsetInMinutes=480
    pub async fn get_offers<A, B, C, D, E>(
        &'a self,
        distance: &A,
        latitude: &B,
        longitude: &C,
        opt_outs: &D,
        timezone_offset_in_minutes: &E,
    ) -> Response<OfferResponse>
    where
        A: Display + ?Sized,
        B: Display + ?Sized,
        C: Display + ?Sized,
        D: Display + ?Sized,
        E: Display + ?Sized,
    {
        let params = Vec::from([
            (String::from("distance"), distance.to_string()),
            (String::from("latitude"), latitude.to_string()),
            (String::from("longitude"), longitude.to_string()),
            (String::from("optOuts"), opt_outs.to_string()),
            (
                String::from("timezoneOffsetInMinutes"),
                timezone_offset_in_minutes.to_string(),
            ),
        ]);

        let token = self.auth_token.as_ref().ok_or("no auth token set")?;
        let request = self
            .get_default_request("exp/v1/offers", Method::GET)
            .query(&params)
            .bearer_auth(token);

        let response = request.send().await?.json::<OfferResponse>().await?;

        Ok(response)
    }

    // https://ap-prod.api.mcd.com/exp/v1/restaurant/location?distance=20&filter=summary&latitude=-32.0117&longitude=115.8845
    pub async fn restaurant_location<A, B, C, D>(
        &'a self,
        distance: &A,
        latitude: &B,
        longitude: &C,
        filter: &D,
    ) -> Response<RestaurantLocationResponse>
    where
        A: Display + ?Sized,
        B: Display + ?Sized,
        C: Display + ?Sized,
        D: Display + ?Sized,
    {
        let params = Vec::from([
            (String::from("distance"), distance.to_string()),
            (String::from("latitude"), latitude.to_string()),
            (String::from("longitude"), longitude.to_string()),
            (String::from("filter"), filter.to_string()),
        ]);

        let token = self.auth_token.as_ref().ok_or("no auth token set")?;
        let request = self
            .get_default_request("exp/v1/restaurant/location", Method::GET)
            .query(&params)
            .bearer_auth(token);

        let response = request
            .send()
            .await?
            .json::<RestaurantLocationResponse>()
            .await?;

        Ok(response)
    }

    // https://ap-prod.api.mcd.com/exp/v1/offers/details/166870
    pub async fn offer_details<S>(&'a self, offer_id: S) -> Response<OfferDetailsResponse>
    where
        S: Display,
    {
        let token = self.auth_token.as_ref().ok_or("no auth token set")?;

        let request = self
            .get_default_request(
                format!("exp/v1/offers/details/{offer_id}").as_str(),
                Method::GET,
            )
            .bearer_auth(token);

        let response = request.send().await?.json::<OfferDetailsResponse>().await?;

        Ok(response)
    }

    // GET https://ap-prod.api.mcd.com/exp/v1/offers/dealstack?offset=480&storeId=951488
    pub async fn get_offers_dealstack<A, B>(
        &'a self,
        offset: &A,
        store_id: &B,
    ) -> Response<OfferDealStackResponse>
    where
        A: Display + ?Sized,
        B: Display + ?Sized,
    {
        let token = self.auth_token.as_ref().ok_or("no auth token set")?;
        let params = Vec::from([
            (String::from("offset"), offset.to_string()),
            (String::from("storeId"), store_id.to_string()),
        ]);

        let request = self
            .get_default_request("exp/v1/offers/dealstack", Method::GET)
            .query(&params)
            .bearer_auth(token);

        let response = request
            .send()
            .await?
            .json::<OfferDealStackResponse>()
            .await?;

        Ok(response)
    }

    // POST https://ap-prod.api.mcd.com/exp/v1/offers/dealstack/166870?offerId=1139347703&offset=480&storeId=951488
    pub async fn add_to_offers_dealstack<A, B, C>(
        &'a self,
        offer_id: &A,
        offset: &B,
        store_id: &C,
    ) -> Response<OfferDealStackResponse>
    where
        A: Display + ?Sized,
        B: Display + ?Sized,
        C: Display + ?Sized,
    {
        let token = self.auth_token.as_ref().ok_or("no auth token set")?;
        let params = Vec::from([
            (String::from("offset"), offset.to_string()),
            (String::from("storeId"), store_id.to_string()),
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
            .await?;

        Ok(response)
    }

    // DELETE https://ap-prod.api.mcd.com/exp/v1/offers/dealstack/offer/166870?offerId=1139347703&offset=480&storeId=951488
    pub async fn remove_from_offers_dealstack<A, B, C, D>(
        &'a self,
        offer_id: &A,
        offer_proposition_id: &B,
        offset: &C,
        store_id: &D,
    ) -> Response<OfferDealStackResponse>
    where
        A: Display + ?Sized,
        B: Display + ?Sized,
        C: Display + ?Sized,
        D: Display + ?Sized,
    {
        // the app sends a body, but this request works without it
        // but we're pretending to be the app :)
        let body = serde_json::json!(
            {
                "storeId": store_id.to_string(),
                "offerId": offer_id.to_string().parse::<i64>()?,
                "offset": offset.to_string().parse::<i64>()?,
            }
        );

        let token = self.auth_token.as_ref().ok_or("no auth token set")?;
        let params = Vec::from([
            (String::from("offerId"), offer_id.to_string()),
            (String::from("offset"), offset.to_string()),
            (String::from("storeId"), store_id.to_string()),
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
            .await?;

        Ok(response)
    }

    // https://ap-prod.api.mcd.com/exp/v1/customer/login/refresh
    pub async fn customer_login_refresh<S>(
        &'a self,
        refresh_token: &S,
    ) -> Response<LoginRefreshResponse>
    where
        S: Display + ?Sized,
    {
        let token = self.auth_token.as_ref().ok_or("no auth token set")?;
        let body = serde_json::json!({ "refreshToken": refresh_token.to_string() });

        let request = self
            .get_default_request("exp/v1/customer/login/refresh", Method::POST)
            .bearer_auth(token)
            .json(&body);

        let response = request.send().await?.json::<LoginRefreshResponse>().await?;

        Ok(response)
    }
}
