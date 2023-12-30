use super::heap_entry::HeapEntry;
use async_condvar_fair::Condvar;
use std::{
    collections::BinaryHeap,
    fmt::Debug,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{sync::Mutex, time::timeout_at};

struct DelayQueueInner<T>
where
    T: Send + Debug,
{
    pub(crate) heap: Mutex<BinaryHeap<HeapEntry<T>>>,
    pub(crate) cond_var: Condvar,
}

pub struct DelayQueue<T>
where
    T: Send + Debug,
{
    inner: Arc<DelayQueueInner<T>>,
}

impl<T> DelayQueue<T>
where
    T: Send + Debug,
{
    pub fn new() -> Self {
        Self {
            inner: DelayQueueInner {
                heap: Default::default(),
                cond_var: Default::default(),
            }
            .into(),
        }
    }

    pub async fn push(&self, item: T, delay: Duration) {
        let instant = Instant::now();
        let entry = HeapEntry {
            delay_util: instant + delay,
            value: item,
        };

        self.inner.heap.lock().await.push(entry);
        self.inner.cond_var.notify_one();
    }

    pub async fn pop(&self) -> Option<T> {
        let mut baton = None;

        loop {
            let instant = Instant::now();
            let mut heap = self.inner.heap.lock().await;
            if let Some(peeked_item) = heap.peek() {
                let peeked_instant = peeked_item.delay_util;
                if instant >= peeked_instant {
                    return Some(heap.pop().unwrap().value);
                } else {
                    let _ = timeout_at(
                        peeked_instant.into(),
                        self.inner.cond_var.wait_no_relock((heap, &self.inner.heap)),
                    )
                    .await;
                }
            } else {
                baton = self
                    .inner
                    .cond_var
                    .wait_no_relock((heap, &self.inner.heap))
                    .await;
            }

            if let Some(baton) = baton.take() {
                baton.dispose()
            };
        }
    }
}

impl<T> Clone for DelayQueue<T>
where
    T: Send + Debug,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> Default for DelayQueue<T>
where
    T: Send + Debug,
{
    fn default() -> Self {
        Self::new()
    }
}
