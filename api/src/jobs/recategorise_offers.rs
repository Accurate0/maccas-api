use super::{Job, JobContext, error::JobError};
use entity::offer_details;
use itertools::Itertools;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QuerySelect};
use tokio_util::sync::CancellationToken;

#[derive(Debug)]
pub struct RecategoriseOffersJob {
    pub api_client: openai::ApiClient,
}

#[async_trait::async_trait]
impl Job for RecategoriseOffersJob {
    fn name(&self) -> String {
        "recategorise_offers".to_owned()
    }

    async fn execute(
        &self,
        context: &JobContext,
        _cancellation_token: CancellationToken,
    ) -> Result<(), JobError> {
        let available_categories = entity::categories::Entity::find()
            .all(context.database)
            .await?
            .into_iter()
            .map(|c| c.name)
            .collect::<Vec<_>>();

        let all_empty_offer_details = entity::offer_details::Entity::find()
            .filter(offer_details::Column::Categories.eq(Vec::<String>::new()))
            // just in case
            .limit(100)
            .all(context.database)
            .await?
            .into_iter()
            .map(|o| o.short_name)
            .unique()
            .collect::<Vec<_>>();

        if all_empty_offer_details.is_empty() {
            tracing::info!("no offers with unpopulated categories");
            return Ok(());
        }

        tracing::info!("{count}", count = all_empty_offer_details.len());

        super::categorise_offers::categorise_offer_names(
            &self.api_client,
            context.database,
            &available_categories,
            all_empty_offer_details,
        )
        .await
    }
}
