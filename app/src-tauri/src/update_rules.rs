//! Small rules the update flow applies, kept pure so they are tested.

use std::time::Duration;

/// Opening the main window checks again when the last check is this old.
pub(crate) const STALE_AFTER: Duration = Duration::from_secs(15 * 60);
/// Idle means no GST page has connected for this many seconds.
pub(crate) const IDLE_AFTER_SECONDS: i64 = 5 * 60;

/// Whole-percent download progress, or `None` without a known total.
pub(crate) fn download_percent(received: u64, total: Option<u64>) -> Option<u8> {
    total.filter(|total| *total > 0).map(|total| {
        u8::try_from(received.saturating_mul(100) / total)
            .unwrap_or(100)
            .min(100)
    })
}

/// No request waiting and no GST connection within `IDLE_AFTER_SECONDS`.
pub(crate) fn is_idle(waiting: bool, last_connection_at: Option<i64>, now: i64) -> bool {
    !waiting && last_connection_at.is_none_or(|at| now - at >= IDLE_AFTER_SECONDS)
}

/// True when there was no check yet, or the last one is older than
/// `STALE_AFTER`.
pub(crate) fn is_stale(since_last_check: Option<Duration>) -> bool {
    since_last_check.is_none_or(|elapsed| elapsed > STALE_AFTER)
}

#[cfg(test)]
#[path = "update_rules_tests.rs"]
mod tests;
