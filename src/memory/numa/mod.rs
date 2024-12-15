use std::{fmt::Debug, thread, time::Duration};

use rand::{thread_rng, Rng};

use super::pool::{MemoryPtrGuard, SyncMemoryPtr};

/// A pointer that provides non-uniform memory access,
/// there will be slight delays.
///
/// Upon calling lock there is a delay between acquiring a lock and
/// then a delay after.
#[derive(Clone)]
pub struct NumaPtr<T>(SyncMemoryPtr<T>);

impl<T: Debug> Debug for NumaPtr<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> NumaPtr<T> {
    pub fn upgrade(obj: SyncMemoryPtr<T>) -> Self {
        Self(obj)
    }
    pub fn lock(&self) -> MemoryPtrGuard<T> {
        let delay = thread_rng().gen_range(0..50);
        thread::sleep(Duration::from_millis(delay));
        let guard = self.0.lock();
        let delay = thread_rng().gen_range(0..50);
        thread::sleep(Duration::from_millis(delay));
        guard
    }
}