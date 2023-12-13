use async_graphql::SimpleObject;

#[derive(SimpleObject)]
pub struct HealthResponse {
    pub database_healthy: bool,
}
