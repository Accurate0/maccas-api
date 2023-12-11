use async_graphql::InputObject;

pub struct Deal {}

#[derive(InputObject)]
pub struct DealCodeInput {
    pub uuid: String,
    pub store_id: String,
}
