use std::str::FromStr;

use super::dataloader::OfferCountDataLoader;
use super::dataloader::OfferDetailsLoader;
use crate::graphql::ValidatedClaims;
use anyhow::Context;
use async_graphql::dataloader::*;
use async_graphql::InputObject;
use async_graphql::Object;
use async_graphql::SimpleObject;
use base::constants::IMAGE_BASE_URL;
use base::constants::IMAGE_EXT;
use entity::offer_cluster_score;
use entity::offer_details;
use entity::offer_name_cluster_association;
use entity::offers;
use sea_orm::prelude::{DateTime, Uuid};
use sea_orm::ColumnTrait;
use sea_orm::DatabaseConnection;
use sea_orm::EntityTrait;
use sea_orm::FromQueryResult;
use sea_orm::JoinType;
use sea_orm::QueryFilter;
use sea_orm::QuerySelect;
use sea_orm::SelectColumns;

#[derive(InputObject)]
pub struct OfferByIdInput {
    pub id: Uuid,
    pub store_id: String,
}

#[derive(SimpleObject)]
pub struct OfferByIdResponse {
    pub code: String,
}

#[derive(FromQueryResult, Debug)]
pub struct OfferCount {
    pub short_name: String,
    pub count: i64,
}

pub struct Offer(pub offers::Model);

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

    pub async fn count(&self, context: &async_graphql::Context<'_>) -> async_graphql::Result<i64> {
        let loader = context.data::<DataLoader<OfferCountDataLoader>>()?;
        let count = loader.load_one(self.short_name(context).await?).await?;

        count.ok_or(anyhow::Error::msg("must find a count for this").into())
    }

    pub async fn recommendation_score(
        &self,
        context: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<f64> {
        let db = context.data::<DatabaseConnection>()?;
        let claims = context.data_opt::<ValidatedClaims>().map(|c| c.0.clone());
        if claims.is_none() {
            return Err(anyhow::Error::msg("must have valid claims").into());
        }

        let claims = claims.unwrap();

        let score =
            offer_name_cluster_association::Entity::find_by_id(self.short_name(context).await?)
                .select_only()
                .select_column(offer_cluster_score::Column::Score)
                .join(
                    JoinType::LeftJoin,
                    offer_name_cluster_association::Entity::belongs_to(offer_cluster_score::Entity)
                        .from(offer_name_cluster_association::Column::ClusterId)
                        .to(offer_cluster_score::Column::ClusterId)
                        .into(),
                )
                .filter(offer_cluster_score::Column::UserId.eq(Uuid::from_str(&claims.user_id)?))
                .into_tuple::<f64>()
                .one(db)
                .await?;

        Ok(score.context("must find score")?)
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
        Ok(format!("{IMAGE_BASE_URL}/{basename}.{IMAGE_EXT}"))
    }

    pub async fn price(
        &self,
        context: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<f64>> {
        self.load_from_related_offer(context, |o| o.price).await
    }
}
