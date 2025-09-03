use super::{Job, JobContext, error::JobError};
use ::entity::offer_details;
use api::Event;
use opentelemetry::trace::TraceContextExt;
use sea_orm::EntityTrait;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

#[derive(Debug)]
pub struct SaveImagesJob;

#[async_trait::async_trait]
impl Job for SaveImagesJob {
    fn name(&self) -> String {
        "save_images".to_owned()
    }

    async fn execute(
        &self,
        context: &JobContext,
        _cancellation_token: CancellationToken,
    ) -> Result<(), JobError> {
        let offer_details = offer_details::Entity::find().all(context.database).await?;

        let trace_id = opentelemetry::Context::current()
            .span()
            .span_context()
            .trace_id()
            .to_string();

        for offer in offer_details {
            let save_image_event = Event::SaveImage {
                basename: offer.image_base_name,
                force: true,
            };

            context
                .event_manager
                .create_event(save_image_event, Duration::from_secs(30), trace_id.clone())
                .await?;
        }

        Ok(())
    }
}
