use async_graphql::Object;
use event::Health;
use reqwest::StatusCode;
use reqwest_middleware::ClientWithMiddleware;
use sea_orm::DatabaseConnection;

use crate::settings::Settings;

pub struct HealthResponse;

#[Object]
impl HealthResponse {
    pub async fn database(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<bool> {
        let db = ctx.data::<DatabaseConnection>()?;
        Ok(db.ping().await.is_ok())
    }

    pub async fn event(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<bool> {
        let settings = ctx.data::<Settings>()?;
        let http_client = ctx.data::<ClientWithMiddleware>()?;

        let request_url = format!("{}/{}", settings.event_api_base, Health::path());
        let event_health_response = http_client.get(request_url).send().await;

        Ok(event_health_response.is_ok_and(|r| r.status() == StatusCode::NO_CONTENT))
    }
}
