use http::Method;
use reqwest_middleware::{ClientWithMiddleware, RequestBuilder};
use tracing::instrument;
use types::{ClientResponse, ClientResult, PlacesRequest, PlacesResponse};

pub mod types;

pub struct ApiClient {
    api_key: String,
    client: ClientWithMiddleware,
}

const BASE_URL: &str = "https://places.googleapis.com/v1/places:searchText";

impl ApiClient {
    pub fn new(api_key: String, client: ClientWithMiddleware) -> Self {
        Self { api_key, client }
    }

    fn get_default_request(&self, method: Method) -> RequestBuilder {
        self.client
            .request(method, BASE_URL.to_string())
            .header("X-Goog-Api-Key", self.api_key.to_owned())
    }

    // /findplacefromtext/json
    #[instrument(skip(self))]
    pub async fn get_place_by_text(
        &self,
        request: &PlacesRequest,
    ) -> ClientResult<ClientResponse<PlacesResponse>> {
        let request = self
            .get_default_request(Method::POST)
            .header("X-Goog-FieldMask", "places.location,places.displayName")
            .body(serde_json::to_string(request)?);

        let response = request.send().await?;
        ClientResponse::from_response(response).await
    }
}
