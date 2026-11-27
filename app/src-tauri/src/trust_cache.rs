//! Nonblocking, single-flight cache for expensive operating-system trust probes.

use std::time::{Duration, Instant};

use crate::commands::overview::TrustView;

/// A completed probe is reusable briefly; refreshes never delay overview replies.
const LIFETIME: Duration = Duration::from_secs(10);

/// Generation protects manual trust changes from an older in-flight result.
#[derive(Default)]
pub(crate) struct TrustCache {
    value: Option<(Instant, TrustView)>,
    pending: bool,
    generation: u64,
}

impl TrustCache {
    /// Returns the last result and grants at most one background refresh ticket.
    pub(crate) fn read(&mut self) -> (TrustView, Option<u64>) {
        let fresh = self
            .value
            .as_ref()
            .is_some_and(|(at, _)| at.elapsed() < LIFETIME);
        let ticket = if fresh || self.pending {
            None
        } else {
            self.pending = true;
            Some(self.generation)
        };
        let view = self.value.as_ref().map_or_else(
            || TrustView {
                status: "checking",
                valid_until: None,
                command: None,
            },
            |(_, view)| view.clone(),
        );
        (view, ticket)
    }

    /// Saves only a result started after the most recent trust mutation.
    pub(crate) fn finish(&mut self, ticket: u64, view: TrustView) {
        self.pending = false;
        if ticket == self.generation {
            self.value = Some((Instant::now(), view));
        }
    }

    /// Invalidates the result without spawning a second concurrent OS probe.
    pub(crate) fn forget(&mut self) {
        self.value = None;
        self.generation = self.generation.wrapping_add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Concurrent misses return immediately and share one OS probe.
    #[test]
    fn misses_share_a_refresh() {
        let mut cache = TrustCache::default();
        let (view, ticket) = cache.read();
        assert_eq!(view.status, "checking");
        assert_eq!(ticket, Some(0));
        assert_eq!(cache.read().1, None);
        cache.finish(
            0,
            TrustView {
                status: "trusted",
                valid_until: None,
                command: None,
            },
        );
        assert_eq!(cache.read().0.status, "trusted");
        assert_eq!(cache.read().1, None);
    }

    /// Installing or removing trust cannot be undone by an old probe reply.
    #[test]
    fn mutation_discards_old_result() {
        let mut cache = TrustCache::default();
        cache.read();
        cache.forget();
        assert_eq!(cache.read().1, None);
        cache.finish(
            0,
            TrustView {
                status: "trusted",
                valid_until: None,
                command: None,
            },
        );
        let (view, ticket) = cache.read();
        assert_eq!(view.status, "checking");
        assert_eq!(ticket, Some(1));
    }
}
