use async_graphql::Object;
use reqwest::StatusCode;
use sea_orm::DatabaseConnection;

use crate::settings::Settings;

pub struct HealthResponse;

#[Object]
impl HealthResponse {
    pub async fn database(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<bool> {
        let db = ctx.data::<DatabaseConnection>()?;
        Ok(db.ping().await.is_ok())
    }

    // checking event and batch health is done with basic clients which do not trace
    pub async fn event(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<bool> {
        let settings = ctx.data::<Settings>()?;
        let http_client = ctx.data::<reqwest::Client>()?;

        let request_url = format!("{}/{}", settings.event_api_base, event::Health::path());
        let event_health_response = http_client.get(request_url).send().await;

        Ok(event_health_response.is_ok_and(|r| r.status() == StatusCode::NO_CONTENT))
    }

    pub async fn batch(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<bool> {
        let settings = ctx.data::<Settings>()?;
        let http_client = ctx.data::<reqwest::Client>()?;

        let request_url = format!("{}/{}", settings.batch_api_base, batch::Health::path());
        let batch_health_response = http_client.get(request_url).send().await;

        Ok(batch_health_response.is_ok_and(|r| r.status() == StatusCode::NO_CONTENT))
    }

    pub async fn recommendations(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<bool> {
        let settings = ctx.data::<Settings>()?;
        let http_client = ctx.data::<reqwest::Client>()?;

        let request_url = format!(
            "{}/{}",
            settings.recommendations_api_base,
            recommendations::Health::path()
        );
        let batch_health_response = http_client.get(request_url).send().await;

        Ok(batch_health_response.is_ok_and(|r| r.status() == StatusCode::NO_CONTENT))
    }
}
