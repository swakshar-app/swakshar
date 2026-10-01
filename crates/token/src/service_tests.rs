//! Shutting the token thread down before the process exits.

use std::time::Duration;

use super::TokenService;
use crate::error::TokenError;

/// Longest a test waits for the idle thread to stop.
const DEADLINE: Duration = Duration::from_secs(2);

/// An idle thread stops in time, and later jobs are refused instead of
/// reaching a driver while the process exits.
#[test]
fn shutdown_stops_the_thread() {
    let service = TokenService::spawn().unwrap();
    assert!(service.shutdown(DEADLINE));
    assert!(matches!(
        service.inventory_blocking(Vec::new()),
        Err(TokenError::ServiceStopped)
    ));
}

/// Every exit path runs the shutdown, so a second call must be harmless.
#[test]
fn shutdown_twice_is_harmless() {
    let service = TokenService::spawn().unwrap();
    assert!(service.shutdown(DEADLINE));
    assert!(service.shutdown(DEADLINE));
}
