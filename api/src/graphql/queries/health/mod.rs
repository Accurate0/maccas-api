use self::types::HealthResponse;
use async_graphql::{Context, Object};
use entity::offers::{self, Entity as Offers};
use sea_orm::{prelude::Uuid, DatabaseConnection, EntityTrait};

mod types;

#[derive(Default)]
pub struct HealthQuery;

#[Object]
impl HealthQuery {
    async fn health<'a>(&self, ctx: &Context<'a>) -> async_graphql::Result<HealthResponse> {
        let db = ctx.data::<DatabaseConnection>()?;

        let db_ok = db.ping().await.is_ok();

        let offer: Option<offers::Model> =
            Offers::find_by_id("a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11".parse::<Uuid>()?)
                .one(db)
                .await?;

        tracing::info!("{:#?}", offer);

        Ok(HealthResponse {
            database_healthy: db_ok,
        })
    }
}
