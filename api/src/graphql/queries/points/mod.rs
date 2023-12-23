use self::types::Points;
use async_graphql::{Context, Object};
use entity::points;
use sea_orm::{DatabaseConnection, EntityTrait};

mod types;

#[derive(Default)]
pub struct PointsQuery;

#[Object]
impl PointsQuery {
    async fn points<'a>(&self, ctx: &Context<'a>) -> async_graphql::Result<Vec<Points>> {
        let db = ctx.data::<DatabaseConnection>()?;

        Ok(points::Entity::find()
            .all(db)
            .await?
            .into_iter()
            .map(Points)
            .collect())
    }
}
