use super::{Job, JobContext, error::JobError};
use base::{http::get_http_client, jwt::generate_internal_jwt};
use entity::offer_audit;
use itertools::Itertools;
use recommendations::{GenerateClusterScores, GenerateClusters, GenerateEmbeddings};
use reqwest::StatusCode;
use reqwest_middleware::RequestBuilder;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QuerySelect};
use std::time::Duration;
use tokio_util::sync::CancellationToken;

#[derive(Debug)]
pub struct GenerateRecommendationsJob {
    pub auth_secret: String,
    pub recommendations_api_base: String,
}

#[async_trait::async_trait]
impl Job for GenerateRecommendationsJob {
    fn name(&self) -> String {
        "generate_recommendations".to_owned()
    }

    async fn execute(
        &self,
        context: &JobContext,
        _cancellation_token: CancellationToken,
    ) -> Result<(), JobError> {
        let http_client = get_http_client()?;
        let token = generate_internal_jwt(
            self.auth_secret.as_ref(),
            "Maccas Event",
            "Maccas Recommendations",
        )?;

        async fn handle_response(path: &str, request: RequestBuilder) -> Result<(), JobError> {
            let response = request.send().await;

            match response {
                Ok(response) => match response.status() {
                    StatusCode::NO_CONTENT => {
                        tracing::info!("call success for {path}");
                    }
                    status => {
                        tracing::warn!(
                            "recommendations failed with {} - {}",
                            status,
                            response.text().await?
                        );
                    }
                },
                Err(e) => tracing::warn!("recommendations request failed with {}", e),
            };

            Ok(())
        }

        // TODO: disable recommendations when computing
        let paths = vec![GenerateEmbeddings::path(), GenerateClusters::path()];

        for path in paths {
            let request_url = format!("{}/{}", self.recommendations_api_base, path);
            let request = http_client
                .post(&request_url)
                .bearer_auth(&token)
                .timeout(Duration::from_secs(10800));
            handle_response(path, request).await?;
        }

        let user_ids = offer_audit::Entity::find()
            .filter(offer_audit::Column::UserId.is_not_null())
            .distinct_on([offer_audit::Column::UserId])
            .all(context.database)
            .await?
            .into_iter()
            .map(|m| m.user_id.unwrap())
            .collect_vec();

        let request_url = format!(
            "{}/{}",
            self.recommendations_api_base,
            GenerateClusterScores::path()
        );

        let request = http_client
            .post(&request_url)
            .bearer_auth(&token)
            .json(&GenerateClusterScores { user_ids });

        handle_response(GenerateClusterScores::path(), request).await?;

        Ok(())
    }
}
