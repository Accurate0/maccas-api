use self::types::Offer;
use async_graphql::{Context, Object};
use entity::offers;
use sea_orm::{DatabaseConnection, EntityTrait};

pub mod dataloader;
mod types;

#[derive(Default)]
pub struct OffersQuery;

#[Object]
impl OffersQuery {
    async fn offers<'a>(&self, ctx: &Context<'a>) -> async_graphql::Result<Vec<Offer>> {
        let db = ctx.data::<DatabaseConnection>()?;

        Ok(offers::Entity::find()
            .all(db)
            .await?
            .into_iter()
            .map(Offer)
            .collect::<Vec<_>>())
    }
}
