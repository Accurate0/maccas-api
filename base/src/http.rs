use reqwest::{Proxy, Request, Response, StatusCode};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware, Extension};
use reqwest_retry::{
    default_on_request_failure, default_on_request_success, policies::ExponentialBackoff,
    RetryTransientMiddleware, Retryable, RetryableStrategy,
};
use reqwest_tracing::{
    default_on_request_end, reqwest_otel_span, DisableOtelPropagation, ReqwestOtelSpanBackend,
    TracingMiddleware,
};
use std::time::{Duration, Instant};
use thiserror::Error;
use tracing::Span;

pub struct AkamaiCdnRetryStrategy;

impl RetryableStrategy for AkamaiCdnRetryStrategy {
    fn handle(
        &self,
        res: &Result<reqwest::Response, reqwest_middleware::Error>,
    ) -> Option<Retryable> {
        let retry_decision = match res {
            Ok(success) if success.status() == StatusCode::FORBIDDEN => Some(Retryable::Transient),
            // Not sure the conditions on locked
            Ok(success) if success.status() == StatusCode::LOCKED => Some(Retryable::Transient),
            Ok(success) => default_on_request_success(success),
            Err(error) => default_on_request_failure(error),
        };

        let maybe_status_code = res.as_ref().map(|r| r.status());
        tracing::info!(
            "status: {:?}, retry: {:?}",
            maybe_status_code,
            matches!(retry_decision, Some(Retryable::Transient))
        );

        retry_decision
    }
}

pub struct TimeTrace;
impl ReqwestOtelSpanBackend for TimeTrace {
    fn on_request_start(req: &Request, extension: &mut http::Extensions) -> Span {
        let url = req.url().as_str();
        extension.insert(Instant::now());

        reqwest_otel_span!(
            name = format!("{} {}", req.method(), url),
            req,
            url = url,
            time_elapsed = tracing::field::Empty,
            time_elapsed_formatted = tracing::field::Empty
        )
    }

    fn on_request_end(
        span: &Span,
        outcome: &reqwest_middleware::Result<Response>,
        extension: &mut http::Extensions,
    ) {
        let time_elapsed = extension.get::<Instant>().unwrap().elapsed().as_millis() as i64;
        default_on_request_end(span, outcome);
        span.record("time_elapsed", time_elapsed);
        span.record("time_elapsed_formatted", format!("{time_elapsed}ms"));
    }
}

#[derive(Error, Debug)]
pub enum HttpCreationError {
    #[error("Request builder error has occurred: `{0}`")]
    ReqwestBuilderError(#[from] reqwest::Error),
}

pub fn get_proxied_maccas_http_client(
    proxy: Proxy,
) -> Result<ClientWithMiddleware, HttpCreationError> {
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

pub fn get_http_client() -> Result<ClientWithMiddleware, HttpCreationError> {
    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);

    Ok(ClientBuilder::new(reqwest::ClientBuilder::new().build()?)
        .with(TracingMiddleware::<TimeTrace>::new())
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build())
}

pub fn get_basic_http_client() -> Result<reqwest::Client, HttpCreationError> {
    Ok(reqwest::ClientBuilder::new()
        .timeout(Duration::from_secs(5))
        .build()?)
}
