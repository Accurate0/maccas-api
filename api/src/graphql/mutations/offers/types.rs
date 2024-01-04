use async_graphql::{InputObject, SimpleObject};
use sea_orm::prelude::Uuid;

#[derive(InputObject)]
pub struct AddOfferInput {
    pub offer_proposition_id: i64,
}

#[derive(InputObject)]
pub struct RemoveOfferInput {
    pub id: Uuid,
}

#[derive(SimpleObject)]
pub struct AddOfferResponse {
    pub id: Uuid,
    pub code: String,
}
