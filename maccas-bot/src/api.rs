use http::Method;
pub struct Api {
    pub base_url: reqwest::Url,
    pub client: reqwest::Client,
}

impl Api {
    pub async fn request<'de, T>(&self, method: Method, endpoint: &str) -> T
    where
        T: serde::de::DeserializeOwned,
    {
        let url = self.base_url.join(endpoint).unwrap();

        let resp = self
            .client
            .request(method, url.as_str())
            .send()
            .await
            .unwrap()
            .json::<T>()
            .await
            .unwrap();

        return resp;
    }

    pub async fn request_without_deserialize(
        &self,
        method: Method,
        endpoint: &str,
    ) -> reqwest::Response {
        let url = self.base_url.join(endpoint).unwrap();

        let resp = self
            .client
            .request(method, url.as_str())
            .send()
            .await
            .unwrap();

        return resp;
    }
}
