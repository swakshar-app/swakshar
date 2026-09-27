//! The rules behind update progress, idle restarts and stale checks.

use std::time::Duration;

use super::{IDLE_AFTER_SECONDS, STALE_AFTER, download_percent, is_idle, is_stale};

/// Progress is a whole percentage, and unknown without a total.
#[test]
fn download_percent_is_whole_and_bounded() {
    assert_eq!(download_percent(0, Some(200)), Some(0));
    assert_eq!(download_percent(50, Some(200)), Some(25));
    assert_eq!(download_percent(200, Some(200)), Some(100));
    assert_eq!(download_percent(300, Some(200)), Some(100));
    assert_eq!(download_percent(50, None), None);
    assert_eq!(download_percent(50, Some(0)), None);
}

/// Idle needs no waiting request and no recent GST connection.
#[test]
fn idle_needs_quiet() {
    let now = 10_000;
    assert!(is_idle(false, None, now));
    assert!(is_idle(false, Some(now - IDLE_AFTER_SECONDS), now));
    assert!(!is_idle(false, Some(now - IDLE_AFTER_SECONDS + 1), now));
    assert!(!is_idle(true, None, now));
}

/// A check is stale when there was none, or it is older than the limit.
#[test]
fn stale_after_the_limit() {
    assert!(is_stale(None));
    assert!(!is_stale(Some(Duration::from_secs(60))));
    assert!(is_stale(Some(STALE_AFTER + Duration::from_secs(1))));
}
