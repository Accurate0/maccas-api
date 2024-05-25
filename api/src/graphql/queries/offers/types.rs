use super::dataloader::OfferDetailsLoader;
use async_graphql::dataloader::*;
use async_graphql::InputObject;
use async_graphql::Object;
use async_graphql::SimpleObject;
use entity::offer_details;
use entity::offers;
use sea_orm::prelude::{DateTime, Uuid};

#[derive(InputObject)]
pub struct OfferByIdInput {
    pub id: Uuid,
    pub store_id: String,
}

#[derive(SimpleObject)]
pub struct OfferByIdResponse {
    pub code: String,
}

const IMAGE_BASE_URL: &str = "https://images.maccas.one";

pub struct Offer(pub offers::Model, pub Option<i64>);

impl Offer {
    async fn load_from_related_offer<T, F>(
        &self,
        context: &async_graphql::Context<'_>,
        mapping: F,
    ) -> async_graphql::Result<T>
    where
        F: Fn(offer_details::Model) -> T,
    {
        let loader = context.data::<DataLoader<OfferDetailsLoader>>()?;

        loader
            .load_one(self.0.offer_proposition_id)
            .await?
            .map(mapping)
            .ok_or(anyhow::Error::msg("no name found for this offer").into())
    }
}

#[Object]
impl Offer {
    pub async fn categories(
        &self,
        context: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Vec<String>> {
        Ok(self
            .load_from_related_offer(context, |o| o.categories)
            .await?
            .unwrap_or_default())
    }

    pub async fn name(
        &self,
        context: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<String> {
        self.load_from_related_offer(context, |o| o.name).await
    }

    pub async fn id(&self) -> &Uuid {
        &self.0.id
    }

    pub async fn offer_proposition_id(&self) -> &i64 {
        &self.0.offer_proposition_id
    }

    pub async fn valid_from(&self) -> &DateTime {
        &self.0.valid_from
    }

    pub async fn valid_to(&self) -> &DateTime {
        &self.0.valid_to
    }

    pub async fn count(&self) -> i64 {
        // this is safe because we look ahead to see if this field exists
        self.1.unwrap()
    }

    pub async fn short_name(
        &self,
        context: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<String> {
        self.load_from_related_offer(context, |o| o.short_name)
            .await
    }

    pub async fn description(
        &self,
        context: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<String> {
        self.load_from_related_offer(context, |o| o.description)
            .await
    }

    pub async fn creation_date(&self) -> &DateTime {
        &self.0.creation_date
    }

    #[graphql(deprecation = "use image_url instead")]
    pub async fn image_basename(
        &self,
        context: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<String> {
        self.load_from_related_offer(context, |o| o.image_base_name)
            .await
    }

    pub async fn image_url(
        &self,
        context: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<String> {
        let basename = self.image_basename(context).await?;
        Ok(format!("{IMAGE_BASE_URL}/{basename}"))
    }

    pub async fn price(
        &self,
        context: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<f64>> {
        self.load_from_related_offer(context, |o| o.price).await
    }
}
