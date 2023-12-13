use self::queries::health::HealthQuery;
use async_graphql::{http::GraphiQLSource, MergedObject};
use axum::response::IntoResponse;

mod queries;

#[derive(Default, MergedObject)]
pub struct QueryRoot(HealthQuery);

pub async fn graphiql() -> impl IntoResponse {
    axum::response::Html(GraphiQLSource::build().endpoint("/graphql").finish())
}
