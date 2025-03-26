use super::{error::JobError, Job, JobContext, JobType};
use base::{http::get_http_client, jwt::generate_internal_jwt};
use recommendations::GenerateEmbeddings;
use reqwest::StatusCode;
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

    fn job_type(&self) -> JobType {
        JobType::Manual
    }

    async fn execute(
        &self,
        _context: &JobContext,
        _cancellation_token: CancellationToken,
    ) -> Result<(), JobError> {
        let http_client = get_http_client()?;
        let token = generate_internal_jwt(
            self.auth_secret.as_ref(),
            "Maccas Batch",
            "Maccas Recommendations",
        )?;

        let request_url = format!(
            "{}/{}",
            self.recommendations_api_base,
            GenerateEmbeddings::path()
        );

        let request = http_client.post(&request_url).bearer_auth(token);

        let response = request.send().await;

        match response {
            Ok(response) => match response.status() {
                StatusCode::ACCEPTED => {
                    tracing::info!("started generating embeddings task");
                }
                status => {
                    tracing::warn!("event failed with {} - {}", status, response.text().await?);
                }
            },
            Err(e) => tracing::warn!("event request failed with {}", e),
        }

        Ok(())
    }
}
