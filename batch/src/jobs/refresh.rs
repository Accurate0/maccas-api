use super::{Job, JobContext};
use std::time::Duration;
use tokio_util::sync::CancellationToken;

#[derive(Debug)]
pub struct RefreshJob;

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct RefreshContext {
    pub test: bool,
}

#[async_trait::async_trait]
impl Job for RefreshJob {
    fn name(&self) -> String {
        "refresh".to_owned()
    }

    async fn prepare(&self) {
        tracing::info!("PREPARE");
    }

    async fn execute(&self, context: &JobContext, cancellation_token: CancellationToken) {
        let future = async move {
            loop {
                let ctx = context.get::<RefreshContext>().await;
                tracing::info!("{:#?}", ctx);

                let result = context.set(RefreshContext { test: true }).await;
                tracing::info!("{:#?}", result);

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

    async fn cleanup(&self, context: &JobContext) {
        let result = context.reset().await;
        tracing::info!("CLEAN UP {:#?}", result);
    }
}
