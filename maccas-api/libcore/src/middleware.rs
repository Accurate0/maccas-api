use reqwest::Request;
use reqwest::Response;
use reqwest_middleware::{Next, Result};
use reqwest_retry::policies::ExponentialBackoff;
use reqwest_retry::RetryTransientMiddleware;
use reqwest_tracing::TracingMiddleware;
use task_local_extensions::Extensions;

pub struct LoggingMiddleware;

#[async_trait::async_trait]
impl reqwest_middleware::Middleware for LoggingMiddleware {
    async fn handle(&self, req: Request, extensions: &mut Extensions, next: Next<'_>) -> Result<Response> {
        log::warn!("Sending request {} {}", req.method(), req.url());
        let res = next.run(req, extensions).await?;
        log::warn!("Got response {}", res.status());
        Ok(res)
    }
}

pub fn get_middleware_http_client(client: reqwest::Client) -> reqwest_middleware::ClientWithMiddleware {
    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
    reqwest_middleware::ClientBuilder::new(client)
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        // .with(LoggingMiddleware)
        .with(TracingMiddleware)
        .build()
}
