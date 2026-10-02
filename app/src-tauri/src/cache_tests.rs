//! A value that stays fresh for a short while.

use std::time::Duration;

use super::Recent;

/// A value is served again within its lifetime, and only for the key it
/// was stored under.
#[test]
fn serves_a_recent_value_for_the_same_key() {
    let recent: Recent<Vec<u8>, &str> = Recent::new(Duration::from_secs(60));
    assert_eq!(recent.get(&vec![1]), None);
    recent.put(vec![1], "tokens");
    assert_eq!(recent.get(&vec![1]), Some("tokens"));
    assert_eq!(recent.get(&vec![2]), None);
}

/// Once the lifetime has passed, the value is gone.
#[test]
fn forgets_an_old_value() {
    let recent: Recent<u8, &str> = Recent::new(Duration::from_millis(10));
    recent.put(1, "tokens");
    std::thread::sleep(Duration::from_millis(30));
    assert_eq!(recent.get(&1), None);
}
