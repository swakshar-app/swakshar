//! Shared state: the token read that polling pages share.

use swakshar_token::TokenService;

use super::AppState;
use crate::settings::Settings;

/// After one read, the next poll within the lifetime is served from memory
/// rather than reading the token again.
#[test]
fn a_recent_token_read_is_remembered() {
    let dir = std::env::temp_dir().join(format!("swakshar-state-{}", std::process::id()));
    let state = AppState::new(dir, TokenService::spawn().unwrap(), Settings::default());
    assert!(state.inventory.get(&Vec::new()).is_none());
    let first = tauri::async_runtime::block_on(state.recent_inventory()).unwrap();
    assert!(state.inventory.get(&Vec::new()).is_some());
    let second = tauri::async_runtime::block_on(state.recent_inventory()).unwrap();
    assert_eq!(first.tokens.len(), second.tokens.len());
}
