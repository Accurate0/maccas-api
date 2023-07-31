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
    #[error("unknown error")]
    Unknown,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlacesResponse {
    pub status: String,
    pub candidates: Vec<Place>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Place {
    #[serde(rename = "formatted_address")]
    pub formatted_address: String,
    pub geometry: Geometry,
    pub name: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Geometry {
    pub location: Location,
    pub viewport: Viewport,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Location {
    pub lat: f64,
    pub lng: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Viewport {
    pub northeast: Location,
    pub southwest: Location,
}
