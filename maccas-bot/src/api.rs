use http::Method;
use types::bot::UsageLog;
use types::bot::UserOptions;
use types::places::PlaceResponse;

const MACCAS_BOT_PREFIX: &'static str = "MACCAS_BOT_";
const MACCAS_BOT_SYSTEM: &'static str = "MaccasBot";

pub struct Api {
    pub base_url: reqwest::Url,
    pub client: reqwest::Client,
}

impl Api {
    pub async fn maccas_request<'de, T>(&self, method: Method, endpoint: &str) -> T
    where
        T: serde::de::DeserializeOwned,
    {
        let url = self
            .base_url
            .join("maccas/v2/")
            .unwrap()
            .join(endpoint)
            .unwrap();

        let resp = self
            .client
            .request(method, url)
            .header("Content-Length", "0")
            .send()
            .await
            .unwrap()
            .json::<T>()
            .await
            .unwrap();

        return resp;
    }

    pub async fn place_request(&self, text: &str) -> PlaceResponse {
        #![allow(dead_code)]

        let url = self.base_url.join("places/v1/place").unwrap();
        let params = Vec::from([("text", text)]);

        let resp = self
            .client
            .request(Method::GET, url)
            .header("Content-Length", "0")
            .query(&params)
            .send()
            .await
            .unwrap()
            .json::<PlaceResponse>()
            .await
            .unwrap();

        return resp;
    }

    pub async fn kvp_get(&self, key: &String) -> reqwest::Response {
        let url = self
            .base_url
            .join(format!("kvp/v1/{MACCAS_BOT_PREFIX}{key}").as_str())
            .unwrap();

        let resp = self
            .client
            .request(Method::GET, url)
            .header("Content-Length", "0")
            .send()
            .await
            .unwrap();

        return resp;
    }

    pub async fn kvp_set(&self, key: &String, value: &UserOptions) -> reqwest::Response {
        let url = self
            .base_url
            .join(format!("kvp/v1/{MACCAS_BOT_PREFIX}{key}").as_str())
            .unwrap();

        let resp = self
            .client
            .request(Method::POST, url)
            .body(serde_json::to_string(value).unwrap())
            .send()
            .await
            .unwrap();

        return resp;
    }

    pub async fn log(&self, value: &UsageLog<'_>) {
        let url = self.base_url.join(format!("log/v1/log").as_str()).unwrap();

        self.client
            .request(Method::POST, url)
            .header("X-Source", MACCAS_BOT_SYSTEM)
            .body(serde_json::to_string(value).unwrap())
            .send()
            .await
            .unwrap();
    }

    pub async fn maccas_request_without_deserialize(
        &self,
        method: Method,
        endpoint: &str,
    ) -> reqwest::Response {
        let url = self
            .base_url
            .join("maccas/v2/")
            .unwrap()
            .join(endpoint)
            .unwrap();

        let resp = self
            .client
            .request(method, url)
            .header("Content-Length", "0")
            .send()
            .await
            .unwrap();

        return resp;
    }
}
