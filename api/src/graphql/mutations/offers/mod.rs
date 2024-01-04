use self::types::{AddOfferInput, AddOfferResponse, RemoveOfferInput};
use async_graphql::{Context, Object};
use sea_orm::prelude::Uuid;

mod types;

#[derive(Default)]
pub struct OffersMutation;

#[Object]
impl OffersMutation {
    async fn add_offer<'a>(
        &self,
        _ctx: &Context<'a>,
        _input: AddOfferInput,
    ) -> async_graphql::Result<AddOfferResponse> {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        Ok(AddOfferResponse {
            id: Uuid::new_v4(),
            code: "1111".into(),
        })
    }

    async fn remove_offer<'a>(
        &self,
        _ctx: &Context<'a>,
        input: RemoveOfferInput,
    ) -> async_graphql::Result<Uuid> {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        Ok(input.id)
    }
}
