use self::queries::{health::HealthQuery, offers::OffersQuery};
use async_graphql::{http::GraphiQLSource, MergedObject};
use axum::response::IntoResponse;

pub mod queries;

#[derive(Default, MergedObject)]
pub struct QueryRoot(HealthQuery, OffersQuery);

pub async fn graphiql() -> impl IntoResponse {
    axum::response::Html(GraphiQLSource::build().endpoint("/graphql").finish())
}
