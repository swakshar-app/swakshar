//! A value kept for a short while, so pages polling at the same time share
//! one read of the token instead of each going through the driver.

use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::state::lock;

/// One value, remembered for `lifetime` under the key it was read for.
pub(crate) struct Recent<K, V> {
    /// How long a stored value is served.
    lifetime: Duration,
    /// When it was stored, what for, and the value.
    slot: Mutex<Option<(Instant, K, V)>>,
}

impl<K: PartialEq, V: Clone> Recent<K, V> {
    /// An empty cache serving values for `lifetime`.
    pub(crate) fn new(lifetime: Duration) -> Self {
        Self {
            lifetime,
            slot: Mutex::new(None),
        }
    }

    /// The value stored for `key`, while it is within its lifetime.
    pub(crate) fn get(&self, key: &K) -> Option<V> {
        lock(&self.slot)
            .as_ref()
            .filter(|(at, stored, _)| stored == key && at.elapsed() < self.lifetime)
            .map(|(_, _, value)| value.clone())
    }

    /// Stores `value` for `key`, replacing whatever was there.
    pub(crate) fn put(&self, key: K, value: V) {
        *lock(&self.slot) = Some((Instant::now(), key, value));
    }
}

#[cfg(test)]
#[path = "cache_tests.rs"]
mod tests;
