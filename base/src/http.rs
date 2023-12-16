use reqwest::{Proxy, StatusCode};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{
    default_on_request_failure, default_on_request_success, policies::ExponentialBackoff,
    RetryTransientMiddleware, Retryable, RetryableStrategy,
};
use reqwest_tracing::TracingMiddleware;
use thiserror::Error;

pub struct AkamaiCdnRetryStrategy;

impl RetryableStrategy for AkamaiCdnRetryStrategy {
    fn handle(
        &self,
        res: &Result<reqwest::Response, reqwest_middleware::Error>,
    ) -> Option<Retryable> {
        match res {
            Ok(success) => {
                if success.status() == StatusCode::FORBIDDEN {
                    Some(Retryable::Transient)
                } else {
                    default_on_request_success(success)
                }
            }
            Err(error) => default_on_request_failure(error),
        }
    }
}

#[derive(Error, Debug)]
pub enum HttpCreationError {
    #[error("Request builder error has occurred: `{0}`")]
    ReqwestBuilderError(#[from] reqwest::Error),
}

pub fn get_http_client(proxy: Proxy) -> Result<ClientWithMiddleware, HttpCreationError> {
    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);

    Ok(
        ClientBuilder::new(reqwest::ClientBuilder::new().proxy(proxy).build()?)
            .with(TracingMiddleware::default())
            .with(RetryTransientMiddleware::new_with_policy_and_strategy(
                retry_policy,
                AkamaiCdnRetryStrategy,
            ))
            .build(),
    )
}
