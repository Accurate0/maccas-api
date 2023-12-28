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
    pub distance: f64,
}

#[derive(SimpleObject)]
pub struct Location {
    pub name: String,
    pub store_number: i64,
    pub address: String,
}
