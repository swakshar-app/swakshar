//! The exit keeps going when one of its steps fails.

use super::guarded;

/// A step that panics neither aborts the exit nor skips the steps after it.
#[test]
fn a_failing_step_does_not_stop_the_rest() {
    let mut ran_after = false;
    guarded("first", || panic!("a step blew up"));
    guarded("second", || ran_after = true);
    assert!(ran_after);
}
