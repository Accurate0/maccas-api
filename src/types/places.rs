// literally a bunch of types, we won't always use everything so..
#![allow(dead_code)]

use serde_derive::Deserialize;
use serde_derive::Serialize;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaceResponse {
    pub status: String,
    pub result: Option<Place>,
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
    #[serde(skip_serializing, skip_deserializing)]
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
