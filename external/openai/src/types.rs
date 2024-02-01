use http::HeaderMap;
use http::StatusCode;
use serde::Deserialize;
use serde::Serialize;
use std::fmt::Debug;
use thiserror::Error;

#[derive(Debug)]
pub struct ClientResponse<T> {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: T,
}

pub type ClientResult<T> = Result<T, ClientError>;

impl<'a, T> ClientResponse<T>
where
    T: for<'de> serde::Deserialize<'de> + Debug,
{
    pub async fn from_response(resp: reqwest::Response) -> Result<Self, ClientError> {
        // return the status error before trying to decode the response to propogate correct error
        let resp = resp.error_for_status()?;
        Ok(Self {
            status: resp.status(),
            headers: resp.headers().clone(),
            body: resp.json::<T>().await?,
        })
    }
}

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("request or middleware error")]
    RequestOrMiddlewareError(#[from] reqwest_middleware::Error),
    #[error("request error")]
    RequestError(#[from] reqwest::Error),
    #[error("serialization error")]
    SerializationError(#[from] serde_json::Error),
    #[error("unknown error")]
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseFormatOptions {
    Text,
    JsonObject,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseFormat {
    #[serde(rename = "type")]
    pub type_field: ResponseFormatOptions,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenAIChatCompletionRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(rename = "max_tokens", skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<i64>,
    #[serde(rename = "response_format", skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormat>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenAIChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub choices: Vec<ChatChoice>,
    pub usage: Usage,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatChoice {
    pub index: i64,
    pub message: ChatMessage,
    #[serde(rename = "finish_reason")]
    pub finish_reason: Option<String>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Usage {
    #[serde(rename = "prompt_tokens")]
    pub prompt_tokens: i64,
    #[serde(rename = "completion_tokens")]
    pub completion_tokens: i64,
    #[serde(rename = "total_tokens")]
    pub total_tokens: i64,
}
