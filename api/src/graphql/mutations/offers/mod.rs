use async_graphql::{Context, Object};

#[derive(Default)]
pub struct OffersMutation;

#[Object]
impl OffersMutation {
    async fn add_offer<'a>(&self, _ctx: &Context<'a>) -> async_graphql::Result<bool> {
        Ok(true)
    }
}
