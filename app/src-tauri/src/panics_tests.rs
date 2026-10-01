//! The panic hook.

use super::log_panics;

/// With the hook installed, a panic still unwinds normally and the hook
/// itself does not fail, even with no logger set.
#[test]
fn hook_keeps_panics_working() {
    log_panics();
    let caught = std::panic::catch_unwind(|| panic!("logged and then handled"));
    assert!(caught.is_err());
}
