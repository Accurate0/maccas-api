mod heap_entry;

use heap_entry::HeapEntry;
use serde::Serialize;
use std::{
    collections::BinaryHeap,
    fmt::Debug,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{
    sync::{Notify, RwLock},
    time::timeout_at,
};

struct DelayQueueInner<T>
where
    T: Send + Debug,
{
    pub(crate) heap: RwLock<BinaryHeap<HeapEntry<T>>>,
    pub(crate) notify: Notify,
}

pub struct DelayQueue<T>
where
    T: Send + Debug,
{
    inner: Arc<DelayQueueInner<T>>,
}

#[derive(Serialize)]
pub struct IntrospectionResult<T>
where
    T: Serialize,
{
    pub delay_util: Duration,
    pub value: T,
}

impl<T> DelayQueue<T>
where
    T: Send + Debug + Clone + Serialize,
{
    pub fn new() -> Self {
        Self {
            inner: DelayQueueInner {
                heap: Default::default(),
                notify: Default::default(),
            }
            .into(),
        }
    }

    pub async fn introspect(&self) -> Vec<IntrospectionResult<T>> {
        self.inner
            .heap
            .read()
            .await
            .iter()
            .map(|e| IntrospectionResult {
                delay_util: e.delay_util - Instant::now(),
                value: e.value.clone(),
            })
            .collect::<Vec<_>>()
    }

    pub async fn push(&self, item: T, delay: Duration) {
        let instant = Instant::now();
        let entry = HeapEntry {
            delay_util: instant + delay,
            value: item,
        };

        self.inner.heap.write().await.push(entry);
        self.inner.notify.notify_one();
    }

    pub async fn pop(&self) -> Option<T> {
        loop {
            let instant = Instant::now();
            let mut heap = self.inner.heap.write().await;
            if let Some(peeked_item) = heap.peek() {
                let peeked_instant = peeked_item.delay_util;
                if instant >= peeked_instant {
                    return Some(heap.pop().unwrap().value);
                } else {
                    // must release lock before going afk
                    drop(heap);
                    let _ = timeout_at(peeked_instant.into(), self.inner.notify.notified()).await;
                }
            } else {
                // must release lock before going afk
                drop(heap);
                self.inner.notify.notified().await;
            }
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
    T: Send + Debug + Clone + Serialize,
{
    fn default() -> Self {
        Self::new()
    }
}
