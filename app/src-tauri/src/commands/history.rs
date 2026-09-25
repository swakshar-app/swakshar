//! The Activity view.

use tauri::State;

use crate::activity::ActivityEntry;
use crate::state::AppState;

/// Entries shown in the Activity view.
const ACTIVITY_LIMIT: usize = 200;

/// Recent requests, newest first.
#[tauri::command]
pub(crate) fn get_activity(state: State<'_, AppState>) -> Vec<ActivityEntry> {
    state.activity.recent(ACTIVITY_LIMIT)
}

/// Deletes the history.
#[tauri::command]
pub(crate) fn clear_activity(state: State<'_, AppState>) {
    state.activity.clear();
}
