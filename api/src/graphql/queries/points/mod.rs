use self::types::{FilterInput, Points};
use async_graphql::{Context, Object};
use entity::points;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

mod types;

#[derive(Default)]
pub struct PointsQuery;

#[Object]
impl PointsQuery {
    async fn points<'a>(
        &self,
        ctx: &Context<'a>,
        filter: Option<FilterInput>,
    ) -> async_graphql::Result<Vec<Points>> {
        let db = ctx.data::<DatabaseConnection>()?;

        Ok(points::Entity::find()
            .filter(
                points::Column::CurrentPoints
                    .gte(filter.map(|f| f.minimum_current_points).unwrap_or(0)),
            )
            .all(db)
            .await?
            .into_iter()
            .map(Points)
            .collect())
    }
}
