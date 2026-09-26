//! Where Swakshar keeps per-user files. The app and the CLI share them, so
//! trusting the local CA once covers both.

use std::env;
use std::path::{Path, PathBuf};

/// Bundle identifier; also the name of the per-user data directory.
pub const APP_IDENTIFIER: &str = "app.swakshar.desktop";

/// Sub-directory holding the TLS files.
const TLS_DIR: &str = "tls";

/// Per-user data directory, matching Tauri's app data directory:
/// macOS `~/Library/Application Support/<id>`, Windows `%APPDATA%\<id>`,
/// Linux `$XDG_DATA_HOME/<id>` or `~/.local/share/<id>`.
pub fn data_dir() -> Option<PathBuf> {
    let base = if cfg!(target_os = "macos") {
        home()?.join("Library").join("Application Support")
    } else if cfg!(target_os = "windows") {
        PathBuf::from(env::var_os("APPDATA")?)
    } else {
        env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .filter(|path| path.is_absolute())
            .or_else(|| home().map(|home| home.join(".local").join("share")))?
    };
    Some(base.join(APP_IDENTIFIER))
}

/// The TLS directory inside a data directory.
pub fn tls_dir(data_dir: &Path) -> PathBuf {
    data_dir.join(TLS_DIR)
}

/// The user's home directory from `HOME`.
fn home() -> Option<PathBuf> {
    env::var_os("HOME")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
}
