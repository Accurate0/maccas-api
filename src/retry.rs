use http::StatusCode;
use reqwest_middleware::Error;
use reqwest_retry::{
    default_on_request_failure, default_on_request_success, policies::ExponentialBackoff,
    RetryTransientMiddleware, Retryable, RetryableStrategy,
};
use reqwest_tracing::TracingMiddleware;

pub struct MaccasRetryStrategy;

impl RetryableStrategy for MaccasRetryStrategy {
    fn handle(&self, outcome: &Result<reqwest::Response, Error>) -> Option<Retryable> {
        match outcome {
            Ok(response) => default_on_request_success(response),
            Err(e) => match e {
                Error::Reqwest(request_error) => match request_error.status() {
                    Some(status) => {
                        if let StatusCode::FORBIDDEN = status {
                            Some(Retryable::Transient)
                        } else {
                            default_on_request_failure(e)
                        }
                    }
                    _ => default_on_request_failure(e),
                },
                _ => default_on_request_failure(e),
            },
        }
    }
}

pub fn wrap_in_middleware(client: reqwest::Client) -> reqwest_middleware::ClientWithMiddleware {
    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(2);
    reqwest_middleware::ClientBuilder::new(client)
        .with(RetryTransientMiddleware::new_with_policy_and_strategy(
            retry_policy,
            MaccasRetryStrategy,
        ))
        .with(TracingMiddleware::default())
        .build()
}
