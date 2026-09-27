//! Reopening the main window after a restart into an update. The restart
//! reuses the old process's arguments, so a copy started at login would
//! come back hidden; a marker in the data directory records that the user
//! pressed Restart and expects the window back.

use std::fs;
use std::path::Path;

/// Marker file written just before the restart.
const MARKER_FILE: &str = "reopen-after-update";

/// Whether the main window opens at start: until setup is done, when the
/// user started the app by hand, and after a restart into an update. A
/// start at login otherwise stays in the menu bar.
pub(crate) fn shows_window(onboarding_complete: bool, at_login: bool, after_update: bool) -> bool {
    !onboarding_complete || !at_login || after_update
}

/// Records that the next start should open the main window.
///
/// # Errors
///
/// Returns the I/O error when the marker cannot be written.
pub(crate) fn mark(dir: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dir)?;
    fs::write(dir.join(MARKER_FILE), b"")
}

/// True once after [`mark`]: removes the marker and reports whether it was
/// there.
pub(crate) fn take(dir: &Path) -> bool {
    fs::remove_file(dir.join(MARKER_FILE)).is_ok()
}

#[cfg(test)]
#[path = "relaunch_tests.rs"]
mod tests;
