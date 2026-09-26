//! Build script: Tauri's context, plus one permission per app command so
//! each window's capability can list exactly the commands it may call.

/// Every command the webview may invoke. Keep in step with `app.rs`.
const COMMANDS: &[&str] = &[
    "get_overview",
    "list_tokens",
    "open_status_page",
    "set_paused",
    "complete_onboarding",
    "run_doctor",
    "get_pending_request",
    "approve_request",
    "cancel_request",
    "refresh_request",
    "get_settings",
    "save_settings",
    "add_driver",
    "remove_driver",
    "install_trust",
    "remove_trust",
    "get_activity",
    "clear_activity",
];

/// Runs tauri-build with the app manifest.
fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tauri_build::try_build(
        tauri_build::Attributes::new()
            .app_manifest(tauri_build::AppManifest::new().commands(COMMANDS)),
    )?;
    Ok(())
}
