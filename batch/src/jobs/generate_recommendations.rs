use super::{error::JobError, Job, JobContext, JobType};
use base::{http::get_http_client, jwt::generate_internal_jwt};
use entity::offer_audit;
use event::{CreateBulkEvents, CreateBulkEventsResponse, CreateEvent};
use itertools::Itertools;
use reqwest::StatusCode;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QuerySelect};
use std::time::Duration;
use tokio_util::sync::CancellationToken;

#[derive(Debug)]
pub struct GenerateRecommendationsJob {
    pub auth_secret: String,
    pub event_api_base: String,
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
        context: &JobContext,
        _cancellation_token: CancellationToken,
    ) -> Result<(), JobError> {
        let events = offer_audit::Entity::find()
            .filter(offer_audit::Column::UserId.is_not_null())
            .distinct_on([offer_audit::Column::UserId])
            .all(context.database)
            .await?
            .into_iter()
            .map(|m| CreateEvent {
                event: event::Event::GenerateRecommendations {
                    user_id: m.user_id.unwrap(),
                },
                delay: Duration::ZERO,
            })
            .collect_vec();

        let http_client = get_http_client()?;
        let token =
            generate_internal_jwt(self.auth_secret.as_ref(), "Maccas Batch", "Maccas Event")?;

        let request_url = format!(
            "{}/{}",
            self.event_api_base,
            event::CreateBulkEvents::path()
        );

        let request = http_client
            .post(&request_url)
            .json(&CreateBulkEvents { events })
            .bearer_auth(token);

        let response = request.send().await;

        match response {
            Ok(response) => match response.status() {
                StatusCode::CREATED => {
                    let id = response.json::<CreateBulkEventsResponse>().await?.ids;
                    tracing::info!("created events with id {:?}", id);
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
