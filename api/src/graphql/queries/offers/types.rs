use async_graphql::Object;
use entity::{offer_details, offers};
use sea_orm::{
    prelude::{DateTime, Uuid},
    DatabaseConnection, EntityTrait,
};

pub struct Offer(pub offers::Model);

#[Object]
impl Offer {
    pub async fn name(&self) -> &str {
        &self.0.name
    }

    pub async fn id(&self) -> &Uuid {
        &self.0.id
    }

    pub async fn offer_id(&self) -> &i64 {
        &self.0.offer_id
    }

    pub async fn valid_from(&self) -> &DateTime {
        &self.0.valid_from
    }

    pub async fn valid_to(&self) -> &DateTime {
        &self.0.valid_to
    }

    pub async fn short_name(&self) -> &String {
        &self.0.short_name
    }

    pub async fn description(&self) -> &String {
        &self.0.description
    }

    pub async fn creation_date(&self) -> &DateTime {
        &self.0.creation_date
    }

    pub async fn image_base_name(&self) -> &String {
        &self.0.image_base_name
    }

    pub async fn original_image_base_name(&self) -> &String {
        &self.0.original_image_base_name
    }

    pub async fn created_at(&self) -> &DateTime {
        &self.0.created_at
    }

    pub async fn updated_at(&self) -> &DateTime {
        &self.0.updated_at
    }

    pub async fn offer_proposition_id(&self) -> &i64 {
        &self.0.offer_proposition_id
    }

    pub async fn account_name(&self) -> &String {
        &self.0.account_name
    }

    pub async fn price(
        &self,
        context: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<f64>> {
        let db = context.data::<DatabaseConnection>()?;

        Ok(
            offer_details::Entity::find_by_id(self.0.offer_proposition_id)
                .one(db)
                .await?
                .map(|o| o.price),
        )
    }
}
