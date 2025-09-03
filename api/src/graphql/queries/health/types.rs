use crate::settings::Settings;
use async_graphql::Object;
use reqwest::StatusCode;
use sea_orm::DatabaseConnection;

pub struct HealthResponse;

#[Object]
impl HealthResponse {
    pub async fn database(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<bool> {
        let db = ctx.data::<DatabaseConnection>()?;
        Ok(db.ping().await.is_ok())
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
