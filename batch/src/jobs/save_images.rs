use super::{error::JobError, Job, JobContext, JobType};
use ::entity::offer_details;
use base::jwt::generate_internal_jwt;
use event::{CreateEventResponse, Event};
use reqwest::StatusCode;
use reqwest_middleware::ClientWithMiddleware;
use sea_orm::EntityTrait;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

#[derive(Debug)]
pub struct SaveImagesJob {
    pub http_client: ClientWithMiddleware,
    pub auth_secret: String,
    pub event_api_base: String,
}

#[async_trait::async_trait]
impl Job for SaveImagesJob {
    fn name(&self) -> String {
        "save_images".to_owned()
    }

    fn job_type(&self) -> JobType {
        JobType::Manual
    }

    async fn execute(
        &self,
        context: &JobContext,
        _cancellation_token: CancellationToken,
    ) -> Result<(), JobError> {
        let token =
            generate_internal_jwt(self.auth_secret.as_ref(), "Maccas Batch", "Maccas Event")?;
        let request_url = format!("{}/{}", self.event_api_base, event::CreateEvent::path());
        let offer_details = offer_details::Entity::find().all(context.database).await?;

        for offer in offer_details {
            let save_image_event = event::CreateEvent {
                event: Event::SaveImage {
                    basename: offer.image_base_name,
                },
                delay: Duration::from_secs(0),
            };

            let request = self
                .http_client
                .post(&request_url)
                .json(&save_image_event)
                .bearer_auth(&token);

            let response = request.send().await;

            match response {
                Ok(response) => match response.status() {
                    StatusCode::CREATED => {
                        let id = response.json::<CreateEventResponse>().await?.id;
                        tracing::info!("created image event with id {}", id);
                    }
                    status => {
                        tracing::warn!("event failed with {} - {}", status, response.text().await?);
                    }
                },
                Err(e) => tracing::warn!("event request failed with {}", e),
            }
        }

        Ok(())
    }
}
