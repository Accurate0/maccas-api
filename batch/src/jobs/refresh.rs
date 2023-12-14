use super::Job;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

#[derive(Debug)]
pub struct RefreshJob;

#[async_trait::async_trait]
impl Job for RefreshJob {
    fn name(&self) -> String {
        "refresh".to_owned()
    }

    async fn prepare(&self) {}

    async fn execute(&self, cancellation_token: CancellationToken) {
        let future = async move {
            loop {
                tracing::info!("running");
                tokio::time::sleep(Duration::from_secs(5)).await
            }
        };

        tokio::select! {
            _ = cancellation_token.cancelled() => {
                tracing::warn!("cancellation requested");
            }
            _ = future => {}
        }
    }

    async fn cleanup(&self) {}
}
