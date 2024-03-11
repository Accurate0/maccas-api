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
    #[error("request or middleware error")]
    RequestOrMiddlewareError(#[from] reqwest_middleware::Error),
    #[error("request error")]
    RequestError(#[from] reqwest::Error),
    #[error("serialization error")]
    SerializationError(#[from] serde_json::Error),
    #[error("unknown error")]
    Unknown,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlacesRequest {
    pub text_query: String,
    pub location_bias: Area,
    pub max_result_count: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Area {
    pub rectangle: Rectangle,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rectangle {
    pub low: Location,
    pub high: Location,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlacesResponse {
    pub places: Vec<Place>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Place {
    pub location: Location,
    pub display_name: DisplayName,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Location {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisplayName {
    pub text: String,
    pub language_code: String,
}
