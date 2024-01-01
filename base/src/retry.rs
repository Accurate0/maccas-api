use futures::Future;
use std::time::Duration;
use tokio::time::sleep;

pub enum OperationResult<R, E> {
    Ok(R),
    Err(E),
    Retry(E),
}

pub enum RetryDecision<R, E> {
    Retry(Duration, E),
    RetryExhausted(E),
    Ok(R),
    Err(E),
}

pub enum RetryResult<V, E> {
    Ok { attempts: u64, value: V },
    Err { attempts: u64, value: E },
}

impl<R, E> From<Result<R, E>> for OperationResult<R, E> {
    fn from(val: Result<R, E>) -> Self {
        match val {
            Ok(v) => OperationResult::Ok(v),
            Err(e) => OperationResult::Retry(e),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ExponentialBackoff {
    pub(crate) delay: Duration,
    pub(crate) max_attempts: u64,
    pub(crate) current_attempt: u64,
}

impl ExponentialBackoff {
    /// Create a new `ExponentialBackoff` with an initial delay.
    pub fn new(initial_delay: Duration, max_attempts: u64) -> Self {
        Self {
            delay: initial_delay,
            max_attempts,
            current_attempt: 0,
        }
    }
}

impl Iterator for ExponentialBackoff {
    type Item = Duration;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_attempt >= self.max_attempts {
            None
        } else {
            let prev_delay = self.delay;
            self.delay = self.delay.saturating_mul(2);
            self.current_attempt += 1;
            Some(prev_delay)
        }
    }
}

pub struct RetryContext<I>
where
    I: Iterator<Item = Duration>,
{
    iterator: I,
    pub attempts: u64,
}

impl<I> RetryContext<I>
where
    I: Iterator<Item = Duration>,
{
    pub fn new(it: I) -> Self {
        Self {
            iterator: it,
            attempts: 1,
        }
    }

    pub async fn check<R, E>(&mut self, result: OperationResult<R, E>) -> RetryDecision<R, E> {
        match result {
            OperationResult::Ok(v) => RetryDecision::Ok(v),
            OperationResult::Err(e) => RetryDecision::Err(e),
            OperationResult::Retry(e) => {
                if let Some(delay) = self.iterator.next() {
                    self.attempts += 1;
                    RetryDecision::Retry(delay, e)
                } else {
                    RetryDecision::RetryExhausted(e)
                }
            }
        }
    }
}

pub async fn retry_async<I, O, R, E, FOR, OR>(it: I, mut operation: O) -> RetryResult<R, E>
where
    I: IntoIterator<Item = Duration>,
    O: FnMut() -> FOR,
    FOR: Future<Output = OR>,
    OR: Into<OperationResult<R, E>>,
    E: std::fmt::Display,
{
    let mut ctx = RetryContext::new(it.into_iter());

    loop {
        match ctx.check(operation().await.into()).await {
            RetryDecision::Retry(d, e) => {
                tracing::info!("sleeping: {:?} before retrying due to {}", d, e);
                sleep(d).await;
            }
            RetryDecision::Ok(v) => {
                return RetryResult::Ok {
                    attempts: ctx.attempts,
                    value: v,
                }
            }
            RetryDecision::RetryExhausted(e) => {
                return RetryResult::Err {
                    attempts: ctx.attempts,
                    value: e,
                }
            }
            RetryDecision::Err(e) => {
                return RetryResult::Err {
                    attempts: ctx.attempts,
                    value: e,
                }
            }
        }
    }
}
