use http::Method;
use reqwest_middleware::{ClientWithMiddleware, RequestBuilder};
use std::fmt::Debug;
use tracing::instrument;
use types::{
    ClientResponse, ClientResult, OpenAIChatCompletionRequest, OpenAIChatCompletionResponse,
};

pub mod types;

#[derive(Clone)]
pub struct ApiClient {
    api_key: String,
    client: ClientWithMiddleware,
}

impl Debug for ApiClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ApiClient")
            .field("client", &self.client)
            .finish()
    }
}

const BASE_URL: &str = "https://api.openai.com/v1";

impl ApiClient {
    pub fn new(api_key: String, client: ClientWithMiddleware) -> Self {
        Self { api_key, client }
    }

    fn get_default_request(&self, resource: &str, method: Method) -> RequestBuilder {
        self.client
            .request(method, format!("{}/{resource}", BASE_URL))
            .header(
                "Authorization",
                format!("Bearer {}", self.api_key.to_owned()),
            )
    }

    // chat/completions
    #[instrument(skip(self), fields(statusCode))]
    pub async fn chat_completions(
        &self,
        request: &OpenAIChatCompletionRequest,
    ) -> ClientResult<ClientResponse<OpenAIChatCompletionResponse>> {
        let request = self
            .get_default_request("chat/completions", Method::POST)
            .json(request);

        let response = request.send().await?;
        ClientResponse::from_response(response).await
    }
}
