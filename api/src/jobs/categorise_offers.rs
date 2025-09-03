use super::{error::JobError, Job, JobContext};
use anyhow::Context;
use itertools::Itertools;
use openai::types::{
    ChatMessage, OpenAIChatCompletionRequest, ResponseFormat, ResponseFormatOptions,
};
use sea_orm::{sea_query::Expr, ColumnTrait, Condition, EntityTrait, QueryFilter};
use std::collections::HashMap;
use tokio_util::sync::CancellationToken;

#[derive(Debug)]
pub struct CategoriseOffersJob {
    pub api_client: openai::ApiClient,
}

pub fn get_prompt(available_categories: &str, offer_details: &str) -> Vec<ChatMessage> {
    [
        ChatMessage {
            role: "system".to_string(),
            content: "You are to categorise strings based on a preexisting category list, you must always response with valid JSON".to_string(),
        },
        ChatMessage {
            role: "user".to_string(),
            content: format!(r#"Give the following categories as comma separated: {available_categories}
            Categorise the following names which are also comma separated, you must select the categories that fit the best:
            {offer_details}

            You may select multiple categories, only if they match well however, single categories are preferred where possible.
            Provide single values as arrays as well. These names are from McDonald's, you must use your knowledge of their menu.

            If a name does not match any category, return an empty array value in the json instead.
            You must respond with a JSON dictionary that maps the name to the category selected."#,)
        }
    ].to_vec()
}

#[async_trait::async_trait]
impl Job for CategoriseOffersJob {
    fn name(&self) -> String {
        "categorise_offers".to_owned()
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
            .join(",");

        let offer_details = entity::offer_details::Entity::find()
            .filter(Condition::any().add(entity::offer_details::Column::Categories.is_null()))
            .all(context.database)
            .await?
            .into_iter()
            .map(|o| o.short_name)
            .unique()
            .collect::<Vec<_>>();

        if offer_details.is_empty() {
            tracing::info!("no offers with unpopulated categories");
            return Ok(());
        }

        let offer_details = offer_details.join(",");

        let response = self
            .api_client
            .chat_completions(&OpenAIChatCompletionRequest {
                model: "gpt-4o".to_string(),
                messages: get_prompt(&available_categories, &offer_details),
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
            serde_json::from_str::<HashMap<String, Vec<String>>>(&response.message.content)?;

        // FIXME: bad...
        for (key, value) in response {
            entity::offer_details::Entity::update_many()
                .filter(entity::offer_details::Column::ShortName.eq(key))
                .col_expr(
                    entity::offer_details::Column::Categories,
                    Expr::value(value),
                )
                .exec(context.database)
                .await?;
        }

        Ok(())
    }
}
