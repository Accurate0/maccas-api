use async_graphql::InputObject;

pub struct Location {}

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
