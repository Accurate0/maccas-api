use std::collections::HashMap;

use super::{error::JobError, Job, JobContext};
use crate::settings::{McDonalds, Proxy};
use anyhow::Context;
use entity::accounts;
use sea_orm::{sea_query::OnConflict, EntityTrait, Set};
use tokio_util::sync::CancellationToken;

#[derive(Debug)]
pub struct PreloadProductsJob {
    pub proxy_config: Proxy,
    pub mcdonalds_config: McDonalds,
}

#[async_trait::async_trait]
impl Job for PreloadProductsJob {
    fn name(&self) -> String {
        "preload_products".to_owned()
    }

    async fn execute(
        &self,
        context: &JobContext,
        _cancellation_token: CancellationToken,
    ) -> Result<(), JobError> {
        let account_to_use = accounts::Entity::find()
            .one(&context.database)
            .await?
            .ok_or_else(|| anyhow::Error::msg("no account found"))?;

        let proxy = reqwest::Proxy::all(self.proxy_config.url.clone())?
            .basic_auth(&self.proxy_config.username, &self.proxy_config.password);

        let api_client = base::maccas::get_activated_maccas_api_client(
            account_to_use,
            proxy,
            &self.mcdonalds_config.client_id,
            &context.database,
        )
        .await?;

        // FIXME: what does the id mean?
        let categories_response = api_client
            .get_menu_categories("1")
            .await?
            .body
            .response
            .categories;

        let category_models = categories_response
            .into_iter()
            .map(|c| entity::categories::ActiveModel {
                id: Set(c.id),
                name: Set(c
                    .names
                    .into_iter()
                    .find(|n| n.locale == *"en-AU")
                    .map(|n| n.short_name)),
                ..Default::default()
            })
            .collect::<Vec<_>>();

        entity::categories::Entity::insert_many(category_models)
            .on_empty_do_nothing()
            .on_conflict(OnConflict::new().do_nothing().to_owned())
            .exec_without_returning(&context.database)
            .await?;

        let categories_map = entity::categories::Entity::find()
            .all(&context.database)
            .await?
            .into_iter()
            .map(|c| (c.id, c.name))
            .collect::<HashMap<_, _>>();

        // FIXME: use something else? or more than one?
        let store_id = "950735";
        let response = api_client
            .get_menu_catalog("AU", store_id, "summary")
            .await?;

        let catalog_response = response
            .body
            .store
            .first()
            .context("must have store in response")?;

        let mut models = Vec::new();

        for product in &catalog_response.products {
            let name = product
                .names
                .names
                .iter()
                .find(|n| n.language_id == *"en-AU")
                .map(|p| p.name.clone());

            let id = product.product_code;
            let energy = product.nutrition.as_ref().map(|n| n.energy);

            // compete category names
            let category_names = product
                .categories
                .clone()
                .unwrap_or_default()
                .iter()
                .filter_map(|c| categories_map.get(&c.display_category_id).cloned())
                .flatten()
                .collect::<Vec<_>>();

            models.push(entity::products::ActiveModel {
                id: Set(id),
                name: Set(name),
                categories: Set(category_names),
                energy: Set(energy),
                ..Default::default()
            })
        }

        entity::products::Entity::insert_many(models)
            .on_empty_do_nothing()
            .on_conflict(OnConflict::new().do_nothing().to_owned())
            .exec_without_returning(&context.database)
            .await?;

        Ok(())
    }

    async fn cleanup(&self, _context: &JobContext) {}
}
