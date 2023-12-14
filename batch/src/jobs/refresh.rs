use super::Job;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

#[derive(Debug)]
pub struct RefreshJob {}

#[async_trait::async_trait]
impl Job for RefreshJob {
    fn name(&self) -> String {
        "refresh".to_owned()
    }

    async fn prepare(&self) {}

    async fn execute(&self, _cancellation_token: CancellationToken) {
        loop {
            tracing::info!("running");
            tokio::time::sleep(Duration::from_secs(5)).await
        }
    }

    async fn cleanup(&self) {}
}
