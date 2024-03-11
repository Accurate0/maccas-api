use reqwest::{Proxy, Request, Response, StatusCode};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware, Extension};
use reqwest_retry::{
    default_on_request_failure, default_on_request_success, policies::ExponentialBackoff,
    RetryTransientMiddleware, Retryable, RetryableStrategy,
};
use reqwest_tracing::{
    default_on_request_end, DisableOtelPropagation, ReqwestOtelSpanBackend, TracingMiddleware,
};
use std::time::Instant;
use task_local_extensions::Extensions;
use thiserror::Error;
use tracing::Span;

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

pub struct TimeTrace;
impl ReqwestOtelSpanBackend for TimeTrace {
    fn on_request_start(req: &Request, extension: &mut Extensions) -> Span {
        extension.insert(Instant::now());
        let url = req.url().to_string();

        let current = Span::current();

        current.record("url", &url);
        current.record("time_elapsed", tracing::field::Empty);

        current
    }

    fn on_request_end(
        span: &Span,
        outcome: &reqwest_middleware::Result<Response>,
        extension: &mut Extensions,
    ) {
        let time_elapsed = extension.get::<Instant>().unwrap().elapsed().as_millis() as i64;
        default_on_request_end(span, outcome);
        span.record("time_elapsed", format!("{time_elapsed}ms"));
        tracing::info!("finished request");
    }
}

#[derive(Error, Debug)]
pub enum HttpCreationError {
    #[error("Request builder error has occurred: `{0}`")]
    ReqwestBuilderError(#[from] reqwest::Error),
}

pub fn get_http_client(proxy: Proxy) -> Result<ClientWithMiddleware, HttpCreationError> {
    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(5);

    Ok(
        ClientBuilder::new(reqwest::ClientBuilder::new().proxy(proxy).build()?)
            .with_init(Extension(DisableOtelPropagation))
            .with(TracingMiddleware::<TimeTrace>::new())
            .with(RetryTransientMiddleware::new_with_policy_and_strategy(
                retry_policy,
                AkamaiCdnRetryStrategy,
            ))
            .build(),
    )
}

pub fn get_simple_http_client() -> Result<ClientWithMiddleware, HttpCreationError> {
    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);

    Ok(ClientBuilder::new(reqwest::ClientBuilder::new().build()?)
        .with(TracingMiddleware::<TimeTrace>::new())
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build())
}
