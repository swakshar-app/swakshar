//! Dock presence: an open window always shows, the setting keeps it there.

use super::shows_in_dock;

/// An open window puts Swakshar in the Dock whatever the setting says.
#[test]
fn an_open_window_shows_in_the_dock() {
    assert!(shows_in_dock(true, false));
    assert!(shows_in_dock(true, true));
}

/// With every window closed, only the setting keeps Swakshar in the Dock.
#[test]
fn closed_windows_leave_the_dock_unless_kept() {
    assert!(!shows_in_dock(false, false));
    assert!(shows_in_dock(false, true));
}
