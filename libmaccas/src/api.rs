use crate::types::{
    LoginRefreshResponse, LoginResponse, OfferDealStackResponse, OfferDetailsResponse,
    OfferResponse, RestaurantLocationResponse, TokenResponse,
};
use http_auth_basic::Credentials;
use reqwest::Method;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware, RequestBuilder};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use std::time::Duration;
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

        let creds = serde_json::json!({
            "credentials": {
                "loginUsername": login_username,
                "password": login_password,
                "type": "email"
            },
            "deviceId": "51e1a16711be3f77"
        });

        let request = self
            .get_default_request("exp/v1/customer/login", Method::POST)
            .header("authorization", format!("Bearer {token}"))
            .header("x-acf-sensor-data", "1,a,TSmvTj4CNAh3fLD6qs0Aa6eSJlS1+p+lt4OSLD+ziisgn96NOdDjbvN6MSuqrUQ6XdKbu2YQaYdCRF1mrw8s9+fVRgtwywvH1VGIRFspU1j/TfzNRZvd9lJndoowp7UddYTWW0ESdH/cCoSsu642VfwB2cxSJ5H7JsSqQG29MO0=,BI1kbsVlh/VCfDOmg9qKIqqOpztaNey++Rd0JNpS3n6nD5uQOfgnvODCzJ7s8W8uhMzyHr5D8/qmqB8JbVMdJG5HYIaahqLksS7yfu6v3vbN2ULmzJ6mIufRjuVAgVinkauykALnylpyFXgmUeG/PsRviDuAgdmw2zWKQRW0cIw=$7b5auo3lqwHFqbFj17/kmupOqm9c7HRF7O8+0RWUEmBtfDxufRlb9QHwdplGFOxhHjcIfpOwNu/oVQgPIyM0rrt7bEZyyOuMN5GyskvwTMisTqn4GfhvwTKJbIjqk41B6SufyCYDP7K+apeSKAJHt6ol32Ws5yk5dCqOUgrkUemkgRJBRYTmUiMBiyF/xTVexyBbZXt8EmHGRj/SZ9zcXaBBhg6Cad1np/RqrKzsUYdG6TUhPOqI3LnMgXl1TMTldDWGQIOW+gxxZH8x/KhWgxMBmptWbR1ndRqHrINyfxIV8/3LoAA72nprWo4z6B9SUDPUcGee65y/eiTz3FfoE6OArtq/cPcUMyahWYZg7CJu0AJCR+RTd3MkEHDjsxCvpCFYAD8XMJyFDxGadzyApPHT8LyThL/T6nj5rLHH8qy6ozHAfngrOxezK3JdG1F1XQ0RQdNLlOIF+/Tm7PQEtgXFYpyi6yPjG2ipHCdZKNpVz9V3oI1vRtwRuhZYfzfK7WtMV9tbxjQ+Qv9KYiUgXj3ZYIDbpO3CIJ76bBhjEY7q866+slRK6uf8c+H7dKAiRCOeLejKhmKELj4QIahQZYxvTArZOZ1IJOh5s7EmuTg0P5e0JakUIr+dglayBuRK+Y5EAWbA22MbkrI3Sdf8Oh83ysmtzrTYuyUQCaPQ95Uuj+iSl3Tw2LIlm19snlX0gbu5uVFzwPTXch6FjQkf46aTteP54tk7mVSkPMWgohavyBKKnlRrPWHkOD/uP8ZtLMPfZAi7MVYm7ANA3OOz8kqCbCL2ya7ToJuvTVatPJoniJXDetBAHSY4Rj/Puuz5Kj4ozm7VzgphYeWXlrcuw8Pgehp1MJwu577kjZqPK4xfC5vb9IFdDUpRkVQ4Kd84CAcCI5WYtQ1OU36TrdExMeycD3/GCCffVKZdqRtbjShz/WgnwH8wOv7cU11pV1ANwnVRNfF2V9f5EuPa9TqRPMni+dvZNWAACf2GefgxviwwjKdFhr6QHxgzkuy6GVvWBqd8YRJAbsbUOPMSufbOmOLx9uFBPIlC4bw4PxftdaMPgHnzMITBRd0s3s1bFWRz8L9ZTlA9KqSSffwTOXoUS4qn5pSNnSHfWsPKZm9n+BEd9aloGH2Cs0p5ENv9bEgZzE8t8mPMYCk+p8lWo5jZ0I3abYfwXL2iTljl9bmkfdY2YdM9O9dScQ5+kUBAqsz+aA1GLjMKi9lS8QIb7UpbgY9nlPPIw+B/m8th/2vVKvHwLTQYD3lsqG1SFjcqapgNjgS3ag6hChsQUULJLMQLILwAN/Ivc6601L7fLr8oaNlGRsgMq6WkW+UiI3djauup7qx/PhFM/6q2NLOOYibG9ckwoL7MMjW4XGgd0QCtl7L2msOi+cJiDoL3JbcuQ4CLLb6Z3+9l9anO3qCQqDItPEoF7bCfVp2F2JTm8ZMvmbdu9satxlRU28c6ulmb8sYmziELPSW4vxtOrW1I4kUCyp7qhRJG0sN4k6SJ5BOjnoZkyGjBZgPXE+XnpOHzkt0DOMOIEZHSdn3EyYrS7FChsZN8dVh0rRzPnJaR1TEI/rEdxBwgJiGvPzmaXeJzAlEHQobHCIE/TkyBp0rBxopoMjZqHjIzJWB4r+GAKuBE+NcCkl8+GsmtfUtTHz6mO7qRe4s7XtCjIBic5d0Md4GdIVyAUFsO6ImdweaBUb0UmXKNL8to0gutjJ2gngA+Gv+KNztNxBlDMwTHlDTcMvoOLby/ja6QDmJk7mXLZGnpOmLLgYxb9IWTB3K3UmgDDz3jIiIEwQvd/hCPOS0GePTR7NLdDlwt8CE81F0wrnebg/4H0s9hlTcW2l0zeSI26tJ0l9jC1iXmd5elMlX0OrgWWJ9QmhWnaQsstNswaEQ//N7SvRn1NIrmdPQYgcuPV7IoeQB4BIJJHoDLSXuaotzBAhe14aSvuAk7pcTpd1WI8KxyvgpYTViYyOchovfY/TLcVlamUMOKMHoLetzImyIzjFaQpnANbUQJndE5kIDW6GkePyqAZorqhNugAB6tXXON9VIN66FuwWHgm2XgxP5pPsWiS7eyFmaG7DSX7mDJVJRqDOoDoqXKgtVG9ywgmEzZ9+gT3NZhkNB2bV9/YimWVsseRJIL4vK6/j6gZxGXG2onuv/+jQcxQinCZDVn1mdHeCrMzSGFbBabs+z4agEVwMFMGnyN6KUxbfSLXN7sSuPb8SF3qZgzkexdSJoRorg836lBHy12q3wGx2jKDi7FrlbTzRJbCVfZgUnBDFc4vJJIqXhylDzm/U+AfBxyBDL2bO3eLzzKzJGLSGexlN7JqHuHHkznx1bbcOh0YMup2kcz4R1bDRV5oNifRSHwv0dou7Y+jyppgS7cbHSdQmqtPbc1hnFry1rT05LafPPOstIz/i7tyF68A5lRbMH1/sVvko+apcXaPq9hPwJ56IJkAvGY61V7EOncrwbb+0wY5vQTE/RYe/Ay/XNqW80jlyW4AmQPUur9vDKcJcG1tIqo7ymc0ZHxDGzPmJCGCtgkfCKFDs8tZtqlTlDeccc95AETle321rGejHH9B7Vj/CBmtJTR0ne7Y+OTj+wRk4q75GkIDoJAIMLDlVTEBLk9LMbS0C+7Iqg7cg4Kxn8AkzlXxOwLuENeFuX/OEQqVs83CZ1/s255FS7975ayr39vRh0s7vwlmJRkZSoKKAwzKQ37Uz9SJsydnqCvn07Qch1CLNqzOkluwEvz0Olqt99rJl90kCHlxD1Sr8ZIDoSmsavRxOiaCatcGCvXFUteyMle75bVvmKADqKsPP/H0F77NYT3U7S2yisDPrkwRD1h1n1k/omlFb4QNEZQSN9N4FcNpjNqpxkN1JTpMBT3Lrpu2bl/CTGyQYVzjuJH6kOytO9qKWB5JZkX2dL3br6FMjE57j6PwZeKLfmbYatdOpF83r3/rO4imEDEU6WkbTl/0d7tR1O1GxvdYSk9vakCuv0CK2+4bCiJDBfOUOG8yU9pYkFu5eiCRCFLDV6SO/kyjkd+Ig==$0,0,0")
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
        distance: Option<String>,
        latitude: Option<String>,
        longitude: Option<String>,
        filter: Option<String>,
    ) -> reqwest_middleware::Result<RestaurantLocationResponse> {
        let params = Vec::from([
            (
                String::from("distance"),
                distance.unwrap_or("20".to_owned()),
            ),
            (
                String::from("latitude"),
                latitude.unwrap_or("-32.0117".to_owned()),
            ),
            (
                String::from("longitude"),
                longitude.unwrap_or("115.8845".to_owned()),
            ),
            (
                String::from("filter"),
                filter.unwrap_or("summary".to_owned()),
            ),
        ]);

        let token: &String = self.auth_token.as_ref().unwrap();

        let request = self
            .get_default_request("exp/v1/restaurant/location", Method::GET)
            .query(&params)
            .header("authorization", format!("Bearer {token}"));

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
        offset: Option<String>,
        store_id: Option<String>,
    ) -> reqwest_middleware::Result<OfferDealStackResponse> {
        let token: &String = self.auth_token.as_ref().unwrap();
        let params = Vec::from([
            (String::from("offset"), offset.unwrap_or("480".to_owned())),
            (
                String::from("storeId"),
                store_id.unwrap_or("951488".to_owned()),
            ),
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
        offset: Option<String>,
        store_id: Option<String>,
    ) -> reqwest_middleware::Result<OfferDealStackResponse> {
        let token: &String = self.auth_token.as_ref().unwrap();
        let params = Vec::from([
            (String::from("offset"), offset.unwrap_or("480".to_owned())),
            (
                String::from("storeId"),
                store_id.unwrap_or("951488".to_owned()),
            ),
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
        store_id: Option<String>,
    ) -> reqwest_middleware::Result<OfferDealStackResponse> {
        let store_id = store_id.unwrap_or("951488".to_owned());

        // the app sends a body, but this request works without it
        // but we're pretending to be the app :)
        let body = serde_json::json!(
            {
                "storeId": store_id,
                "offerId": offer_id,
                "offset": offset.unwrap_or(480)
            }
        );

        let token: &String = self.auth_token.as_ref().unwrap();
        let params = Vec::from([
            (String::from("offerId"), offer_id.to_string()),
            (String::from("offset"), offset.unwrap_or(480).to_string()),
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
