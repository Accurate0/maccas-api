use super::{Job, JobContext, error::JobError};
use itertools::Itertools;
use openai::types::{DecisionAnswerType, DecisionQuestion, DecisionsRequest};
use sea_orm::{
    ColumnTrait, Condition, DatabaseTransaction, EntityTrait, QueryFilter, sea_query::Expr,
};
use tokio_util::sync::CancellationToken;

const MODEL: &str = "gpt-6-luna";
const CATEGORY_THRESHOLD: f64 = 0.5;

#[derive(Debug)]
pub struct CategoriseOffersJob {
    pub api_client: openai::ApiClient,
}

pub async fn categorise_offer_names(
    api_client: &openai::ApiClient,
    database: &DatabaseTransaction,
    available_categories: &[String],
    short_names: Vec<String>,
) -> Result<(), JobError> {
    let questions = available_categories
        .iter()
        .map(|category| DecisionQuestion::Predicate {
            name: category.clone(),
            instructions: format!(
                "This is the name of an offer from McDonald's, use your knowledge of their menu. Does it belong in the '{category}' category?"
            ),
        })
        .collect::<Vec<_>>();

    for short_name in short_names {
        let response = api_client
            .decisions(&DecisionsRequest {
                model: MODEL.to_string(),
                input: short_name.clone(),
                questions: questions.clone(),
            })
            .await?;

        let answers = response.body.answers;

        // every category must have a predicate answer, otherwise leave the offer for the next run
        let probabilities = available_categories
            .iter()
            .map(|category| {
                answers
                    .iter()
                    .find(|a| a.type_field == DecisionAnswerType::Predicate && &a.name == category)
                    .and_then(|a| a.probability)
                    .map(|probability| (category, probability))
            })
            .collect::<Option<Vec<_>>>();

        let Some(probabilities) = probabilities else {
            tracing::warn!("incomplete decision for {short_name}, skipping: {answers:?}");
            continue;
        };

        let categories = probabilities
            .into_iter()
            .filter(|(_, probability)| *probability >= CATEGORY_THRESHOLD)
            .map(|(category, _)| category.clone())
            .collect::<Vec<_>>();

        entity::offer_details::Entity::update_many()
            .filter(entity::offer_details::Column::ShortName.eq(short_name))
            .col_expr(
                entity::offer_details::Column::Categories,
                Expr::value(categories),
            )
            .exec(database)
            .await?;
    }

    Ok(())
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
            .collect::<Vec<_>>();

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

        categorise_offer_names(
            &self.api_client,
            context.database,
            &available_categories,
            offer_details,
        )
        .await
    }
}
