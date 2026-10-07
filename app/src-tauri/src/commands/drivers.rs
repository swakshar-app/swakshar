//! Adding and removing driver files the automatic search does not cover.
//! The file picker opens from Rust, so the webview needs no dialog access.

use std::path::Path;

use swakshar_token::{ArchSupport, candidate_modules, driver_page};
use tauri::{AppHandle, Manager as _};
use tauri_plugin_dialog::DialogExt as _;
use tauri_plugin_opener::OpenerExt as _;
use tokio::sync::oneshot;

use crate::error::{CommandError, CommandResult};
use crate::settings::Settings;
use crate::state::{AppState, lock};

/// File extensions of PKCS#11 driver files on this OS.
const DRIVER_EXTENSIONS: &[&str] = if cfg!(target_os = "windows") {
    &["dll"]
} else {
    &["dylib", "so"]
};
/// Where most token installers put their driver.
const DRIVER_DIRECTORY: &str = if cfg!(target_os = "windows") {
    r"C:\Windows\System32"
} else {
    "/usr/local/lib"
};

/// Asks for a driver file and adds it. `None` when the user cancels.
#[tauri::command]
pub(crate) async fn add_driver(app: AppHandle) -> CommandResult<Option<Settings>> {
    let (sender, receiver) = oneshot::channel();
    app.dialog()
        .file()
        .set_title("Choose your token's driver file")
        .add_filter("Token driver", DRIVER_EXTENSIONS)
        .set_directory(DRIVER_DIRECTORY)
        .pick_file(move |picked| {
            let _ = sender.send(picked);
        });
    let Some(picked) = receiver.await.ok().flatten() else {
        return Ok(None);
    };
    let path = picked
        .into_path()
        .map_err(|error| CommandError::Message(error.to_string()))?;
    check_driver(&path)?;
    update_modules(&app, |modules| {
        let text = path.display().to_string();
        if !modules.contains(&text) {
            modules.push(text);
        }
    })
    .map(Some)
}

/// Removes a driver the user added.
#[tauri::command]
pub(crate) fn remove_driver(app: AppHandle, path: String) -> CommandResult<Settings> {
    update_modules(&app, |modules| modules.retain(|module| *module != path))
}

/// Opens the maker's official driver download page for a plugged-in token.
/// The page comes from the known-maker table, never from the webview.
#[tauri::command]
pub(crate) fn open_driver_page(app: AppHandle, vendor_id: u16) -> CommandResult<()> {
    let url = driver_page(vendor_id).ok_or_else(|| {
        CommandError::Message(
            "This token's maker has no public download page. Your Certifying Authority or \
             the shop that sold the token provides the driver."
                .to_owned(),
        )
    })?;
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|error| CommandError::Message(error.to_string()))
}

/// Refuses files that cannot be this Mac's driver, with a reason.
fn check_driver(path: &Path) -> CommandResult<()> {
    let candidates = candidate_modules(&[path.to_path_buf()]);
    let Some(candidate) = candidates.iter().find(|module| module.path == path) else {
        return Err(CommandError::Message(
            "That file could not be read.".to_owned(),
        ));
    };
    if !candidate.user_added {
        return Err(CommandError::Message(
            "Swakshar already looks for that driver on its own.".to_owned(),
        ));
    }
    if !candidate.exists {
        return Err(CommandError::Message(
            "That is not a driver file.".to_owned(),
        ));
    }
    if candidate.arch == ArchSupport::Incompatible {
        return Err(CommandError::Message(
            "That driver is built for a different processor than this computer's. Ask your token's supplier for the right one."
                .to_owned(),
        ));
    }
    Ok(())
}

/// Applies `change` to the saved driver list and returns the new settings.
fn update_modules(
    app: &AppHandle,
    change: impl FnOnce(&mut Vec<String>),
) -> CommandResult<Settings> {
    let state = app.state::<AppState>();
    let mut settings = state.settings();
    change(&mut settings.modules);
    settings.validate().map_err(CommandError::Message)?;
    settings.save(&state.data_dir)?;
    *lock(&state.settings) = settings.clone();
    Ok(settings)
}
