use std::time::Instant;

#[derive(Debug)]
pub(crate) struct HeapEntry<T> {
    pub(crate) delay_util: Instant,
    pub(crate) value: T,
}

impl<T> Ord for HeapEntry<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.delay_util.cmp(&self.delay_util)
    }
}

impl<T> PartialOrd for HeapEntry<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> PartialEq for HeapEntry<T> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == std::cmp::Ordering::Equal
    }
}

impl<T> Eq for HeapEntry<T> {}
