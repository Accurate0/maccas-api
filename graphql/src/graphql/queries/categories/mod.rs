use async_graphql::Object;
use sea_orm::{DatabaseConnection, EntityTrait};

#[derive(Default)]
pub struct CategoryQuery;

#[Object]
impl CategoryQuery {
    async fn categories<'a>(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Vec<String>> {
        let db = ctx.data::<DatabaseConnection>()?;

        Ok(entity::categories::Entity::find()
            .all(db)
            .await?
            .into_iter()
            .map(|c| c.name)
            .collect())
    }
}
