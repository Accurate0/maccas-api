use http::Method;
use reqwest_middleware::{ClientWithMiddleware, RequestBuilder};
use tracing::instrument;
use types::{ClientResponse, ClientResult, PlacesResponse};

pub mod types;

pub struct ApiClient {
    api_key: String,
    client: ClientWithMiddleware,
}

const BASE_URL: &str = "https://maps.googleapis.com/maps/api/place";

impl ApiClient {
    pub fn new(api_key: String, client: ClientWithMiddleware) -> Self {
        Self { api_key, client }
    }

    fn get_default_request(&self, resource: &str, method: Method) -> RequestBuilder {
        self.client
            .request(method, format!("{BASE_URL}/{resource}"))
            .query(&[("key", self.api_key.to_owned())])
    }

    // /findplacefromtext/json
    #[instrument(skip(self))]
    pub async fn get_place_by_text(
        &self,
        text: &str,
    ) -> ClientResult<ClientResponse<PlacesResponse>> {
        let request = self
            .get_default_request("findplacefromtext/json", Method::GET)
            .query(&[
                ("inputtype", "textquery"),
                ("fields", "formatted_address,name,geometry"),
                ("input", text),
            ]);

        let response = request.send().await?;
        ClientResponse::from_response(response).await
    }
}
