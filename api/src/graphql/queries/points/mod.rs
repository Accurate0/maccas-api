use self::types::{FilterInput, Points};
use async_graphql::{Context, Object};
use entity::points;
use sea_orm::{prelude::Uuid, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

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
            .map(|p| Points {
                model: p,
                store_id: None,
            })
            .collect())
    }

    async fn points_by_account_id<'a>(
        &self,
        ctx: &Context<'a>,
        account_id: Uuid,
        store_id: Option<String>,
    ) -> async_graphql::Result<Points> {
        let db = ctx.data::<DatabaseConnection>()?;

        Ok(Points {
            model: points::Entity::find_by_id(account_id)
                .one(db)
                .await?
                .ok_or(anyhow::Error::msg("no account found for that id"))?,
            store_id,
        })
    }
}
