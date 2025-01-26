use self::types::HealthResponse;
use async_graphql::Object;

mod types;

#[derive(Default)]
pub struct HealthQuery;

#[Object]
impl HealthQuery {
    async fn health<'a>(&self) -> async_graphql::Result<HealthResponse> {
        Ok(HealthResponse)
    }
}
