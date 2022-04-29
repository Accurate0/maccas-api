use http::Method;
use types::places::PlaceResponse;

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
            .join("maccas/v1/")
            .unwrap()
            .join(endpoint)
            .unwrap();

        let resp = self
            .client
            .request(method, url)
            .send()
            .await
            .unwrap()
            .json::<T>()
            .await
            .unwrap();

        return resp;
    }

    pub async fn place_request(&self, text: &String) -> PlaceResponse {
        #![allow(dead_code)]

        let url = self.base_url.join("places/v1/place").unwrap();
        let params = Vec::from([("text", text)]);

        let resp = self
            .client
            .request(Method::GET, url)
            .query(&params)
            .send()
            .await
            .unwrap()
            .json::<PlaceResponse>()
            .await
            .unwrap();

        return resp;
    }

    pub async fn maccas_request_without_deserialize(
        &self,
        method: Method,
        endpoint: &str,
    ) -> reqwest::Response {
        let url = self
            .base_url
            .join("maccas/v1/")
            .unwrap()
            .join(endpoint)
            .unwrap();

        let resp = self.client.request(method, url).send().await.unwrap();

        return resp;
    }
}
