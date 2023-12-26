use self::types::HealthResponse;
use crate::settings::Settings;
use async_graphql::{Context, Object};
use sea_orm::DatabaseConnection;

mod types;

#[derive(Default)]
pub struct HealthQuery;

#[Object]
impl HealthQuery {
    async fn health<'a>(&self, ctx: &Context<'a>) -> async_graphql::Result<HealthResponse> {
        let db = ctx.data::<DatabaseConnection>()?;
        let _settings = ctx.data::<Settings>()?;

        let db_ok = db.ping().await.is_ok();

        Ok(HealthResponse {
            database_healthy: db_ok,
        })
    }
}
