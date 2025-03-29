use std::time::Duration;

use super::{error::JobError, Job, JobContext, JobType};
use base::{http::get_http_client, jwt::generate_internal_jwt};
use recommendations::{GenerateClusters, GenerateEmbeddings};
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

        let paths = vec![GenerateEmbeddings::path(), GenerateClusters::path()];

        for path in paths {
            let request_url = format!("{}/{}", self.recommendations_api_base, path);
            let request = http_client
                .post(&request_url)
                .bearer_auth(&token)
                .timeout(Duration::from_secs(10800));

            let response = request.send().await;

            match response {
                Ok(response) => match response.status() {
                    StatusCode::NO_CONTENT => {
                        tracing::info!("called success for {path}");
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
            }
        }
        Ok(())
    }
}
