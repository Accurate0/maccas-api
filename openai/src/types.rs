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

impl<T> ClientResponse<T>
where
    T: for<'de> serde::Deserialize<'de> + Debug,
{
    pub async fn from_response(resp: reqwest::Response) -> Result<Self, ClientError> {
        tracing::Span::current().record("statusCode", resp.status().as_u16());

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
    #[error("request or middleware error: {0}")]
    RequestOrMiddlewareError(#[from] reqwest_middleware::Error),
    #[error("request error: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("serialization error: {0}")]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionsRequest {
    pub model: String,
    pub input: String,
    pub questions: Vec<DecisionQuestion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DecisionQuestion {
    Predicate {
        name: String,
        instructions: String,
    },
    Choice {
        name: String,
        instructions: String,
        choices: Vec<DecisionChoice>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionChoice {
    pub value: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionsResponse {
    pub answers: Vec<DecisionAnswer>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionAnswerType {
    Predicate,
    Choice,
    Score,
    Refusal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionAnswer {
    #[serde(rename = "type")]
    pub type_field: DecisionAnswerType,
    pub name: String,
    pub probability: Option<f64>,
    pub choice: Option<String>,
    pub score: Option<f64>,
    pub confidence: Option<f64>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenAIEmbeddingsRequest {
    pub input: String,
    pub dimensions: Option<i64>,
    pub model: String,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenAIEmbeddingsResponse {
    pub object: String,
    pub data: Vec<Embedding>,
    pub model: String,
    pub usage: EmbeddingUsage,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Embedding {
    pub object: String,
    pub embedding: Vec<f32>,
    pub index: i64,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmbeddingUsage {
    #[serde(rename = "prompt_tokens")]
    pub prompt_tokens: i64,
    #[serde(rename = "total_tokens")]
    pub total_tokens: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn decisions_request_serializes_to_api_shape() {
        let request = DecisionsRequest {
            model: "gpt-6-luna".to_string(),
            input: "I was charged twice for my order.".to_string(),
            questions: vec![
                DecisionQuestion::Predicate {
                    name: "is_billing".to_string(),
                    instructions: "Is this a billing complaint?".to_string(),
                },
                DecisionQuestion::Choice {
                    name: "department".to_string(),
                    instructions: "Which department should handle this complaint?".to_string(),
                    choices: vec![DecisionChoice {
                        value: "billing".to_string(),
                        description: "Payments, invoices, and refunds.".to_string(),
                    }],
                },
            ],
        };

        assert_eq!(
            serde_json::to_value(&request).unwrap(),
            json!({
                "model": "gpt-6-luna",
                "input": "I was charged twice for my order.",
                "questions": [
                    {
                        "type": "predicate",
                        "name": "is_billing",
                        "instructions": "Is this a billing complaint?"
                    },
                    {
                        "type": "choice",
                        "name": "department",
                        "instructions": "Which department should handle this complaint?",
                        "choices": [
                            { "value": "billing", "description": "Payments, invoices, and refunds." }
                        ]
                    }
                ]
            })
        );
    }

    #[test]
    fn decisions_response_deserializes() {
        let response: DecisionsResponse = serde_json::from_value(json!({
            "answers": [
                {
                    "type": "choice",
                    "name": "department",
                    "choice": "billing",
                    "probabilities": [
                        { "value": "billing", "probability": 0.95 },
                        { "value": "technical", "probability": 0.05 }
                    ],
                    "confidence": 0.93
                },
                { "type": "predicate", "name": "is_billing", "probability": 0.9 },
                { "type": "refusal", "name": "other" }
            ]
        }))
        .unwrap();

        assert_eq!(response.answers.len(), 3);
        assert_eq!(response.answers[0].choice.as_deref(), Some("billing"));
        assert_eq!(response.answers[1].probability, Some(0.9));
        assert_eq!(response.answers[2].type_field, DecisionAnswerType::Refusal);
    }
}
