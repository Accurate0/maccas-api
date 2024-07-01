use super::{error::JobError, Job, JobContext, JobType};
use anyhow::Context;
use entity::offer_details;
use itertools::Itertools;
use openai::types::{OpenAIChatCompletionRequest, ResponseFormat, ResponseFormatOptions};
use sea_orm::{sea_query::Expr, ColumnTrait, EntityTrait, QueryFilter, QuerySelect};
use std::collections::HashMap;
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

    fn job_type(&self) -> JobType {
        JobType::Manual
    }

    async fn execute(
        &self,
        context: &JobContext,
        _cancellation_token: CancellationToken,
    ) -> Result<(), JobError> {
        let available_categories = entity::categories::Entity::find()
            .all(&context.database)
            .await?
            .into_iter()
            .map(|c| c.name)
            .join(",");

        let all_empty_offer_details = entity::offer_details::Entity::find()
            .filter(offer_details::Column::Categories.eq(Vec::<String>::new()))
            // just in case
            .limit(100)
            .all(&context.database)
            .await?
            .into_iter()
            .map(|o| o.short_name)
            .unique()
            .collect::<Vec<_>>();

        if all_empty_offer_details.is_empty() {
            tracing::info!("no offers with unpopulated categories");
            return Ok(());
        }

        let offer_details = all_empty_offer_details.join(",");

        let response = self
            .api_client
            .chat_completions(&OpenAIChatCompletionRequest {
                model: "gpt-4o".to_string(),
                messages: super::categorise_offers::get_prompt(
                    &available_categories,
                    &offer_details,
                ),
                max_tokens: None,
                response_format: Some(ResponseFormat {
                    type_field: ResponseFormatOptions::JsonObject,
                }),
            })
            .await?;

        // n = 1 by default
        let response = response
            .body
            .choices
            .first()
            .context("must have one choice")?;

        let response =
            serde_json::from_str::<HashMap<String, Option<String>>>(&response.message.content)?;

        // FIXME: bad...
        for (key, value) in response {
            let value = match value {
                Some(v) => vec![v],
                None => vec![],
            };

            entity::offer_details::Entity::update_many()
                .filter(entity::offer_details::Column::ShortName.eq(key))
                .col_expr(
                    entity::offer_details::Column::Categories,
                    Expr::value(value),
                )
                .exec(&context.database)
                .await?;
        }

        Ok(())
    }
}
