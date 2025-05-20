use std::hash::Hasher;

use async_graphql::{InputObject, SimpleObject};

pub struct QueriedLocation {}

#[derive(InputObject)]
pub struct TextSearchInput {
    pub query: String,
}

#[derive(InputObject)]
pub struct CoordinateSearchInput {
    pub lat: f64,
    pub lng: f64,
}

#[derive(InputObject)]
pub struct StoreIdInput {
    pub store_id: String,
}

#[derive(SimpleObject, Clone)]
pub struct Location {
    pub name: String,
    pub store_number: String,
    pub address: String,
    pub distance: Option<f64>,
}

#[derive(Clone)]
pub struct LocationRequest {
    pub lat: f64,
    pub long: f64,
}

impl std::hash::Hash for LocationRequest {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.lat.to_bits().hash(state);
        self.long.to_bits().hash(state);
    }
}

impl PartialEq for LocationRequest {
    fn eq(&self, other: &Self) -> bool {
        self.lat == other.lat && self.long == other.long
    }
}

impl Eq for LocationRequest {}
