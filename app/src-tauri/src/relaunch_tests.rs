//! Reopening after a restart into an update.

use super::{mark, shows_window, take};

/// The window opens until setup is done, on a start by hand, and after a
/// restart into an update; a start at login otherwise stays hidden.
#[test]
fn window_opens_unless_started_at_login() {
    assert!(shows_window(false, true, false));
    assert!(shows_window(true, false, false));
    assert!(shows_window(true, true, true));
    assert!(!shows_window(true, true, false));
}

/// The marker is seen once, then gone.
#[test]
fn marker_is_taken_once() {
    let dir = std::env::temp_dir().join(format!("swakshar-relaunch-{}", std::process::id()));
    assert!(!take(&dir));
    mark(&dir).unwrap();
    assert!(take(&dir));
    assert!(!take(&dir));
    std::fs::remove_dir_all(&dir).unwrap();
}
