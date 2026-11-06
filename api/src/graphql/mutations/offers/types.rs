use async_graphql::{InputObject, SimpleObject};
use sea_orm::prelude::Uuid;

#[derive(InputObject)]
#[graphql(name = "MaccasAddOfferInput")]
pub struct AddOfferInput {
    pub offer_proposition_id: i64,
    pub store_id: String,
}

#[derive(InputObject)]
#[graphql(name = "MaccasRemoveOfferInput")]
pub struct RemoveOfferInput {
    pub id: Uuid,
    pub store_id: String,
}

#[derive(SimpleObject)]
#[graphql(name = "MaccasAddOfferResponse")]
pub struct AddOfferResponse {
    pub id: Uuid,
    pub code: String,
}
