//! User settings, stored as JSON in the data directory.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use swakshar_protocol::{DEFAULT_GREETING_VERSION, OriginPolicy, SIGNER_PORTS};
use swakshar_server::ServerSettings;

/// Settings file name.
const SETTINGS_FILE: &str = "settings.json";
/// Longest greeting version accepted.
const MAX_VERSION_LEN: usize = 16;

/// Everything the user can change.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct Settings {
    /// Extra PKCS#11 driver paths.
    pub(crate) modules: Vec<String>,
    /// Preferred signer port, one of the portal's five.
    pub(crate) preferred_port: Option<u16>,
    /// Greeting version the portal expects.
    pub(crate) greeting_version: String,
    /// Extra page origins allowed to connect.
    pub(crate) extra_origins: Vec<String>,
    /// Launch at login.
    pub(crate) start_at_login: bool,
    /// Guided setup finished.
    pub(crate) onboarding_complete: bool,
    /// The user turned signing on; off on a fresh install.
    pub(crate) signing_enabled: bool,
}

impl Default for Settings {
    /// Automatic port, the August 2026 greeting, GST origins only, and
    /// signing off until the user turns it on.
    fn default() -> Self {
        Self {
            modules: Vec::new(),
            preferred_port: None,
            greeting_version: DEFAULT_GREETING_VERSION.to_owned(),
            extra_origins: Vec::new(),
            start_at_login: false,
            onboarding_complete: false,
            signing_enabled: false,
        }
    }
}

impl Settings {
    /// Loads settings, falling back to defaults when missing or unreadable.
    pub(crate) fn load(dir: &Path) -> Self {
        let path = dir.join(SETTINGS_FILE);
        match fs::read_to_string(&path) {
            Ok(text) => serde_json::from_str(&text).unwrap_or_else(|error| {
                log::warn!("ignoring unreadable settings: {error}");
                Self::default()
            }),
            Err(_) => Self::default(),
        }
    }

    /// Writes settings.
    ///
    /// # Errors
    ///
    /// Returns the I/O error when the file cannot be written.
    pub(crate) fn save(&self, dir: &Path) -> std::io::Result<()> {
        fs::create_dir_all(dir)?;
        let text = serde_json::to_string_pretty(self).map_err(std::io::Error::other)?;
        fs::write(dir.join(SETTINGS_FILE), text)
    }

    /// Checks values before they are saved or applied.
    ///
    /// # Errors
    ///
    /// Returns a sentence describing the first invalid value.
    pub(crate) fn validate(&self) -> Result<(), String> {
        if let Some(port) = self
            .preferred_port
            .filter(|port| !SIGNER_PORTS.contains(port))
        {
            return Err(format!("Port {port} is not one the GST portal tries."));
        }
        let version = self.greeting_version.trim();
        let version_ok = !version.is_empty()
            && version.len() <= MAX_VERSION_LEN
            && version.chars().all(|ch| ch.is_ascii_digit() || ch == '.');
        if !version_ok {
            return Err("The greeting version must look like 2.8.".to_owned());
        }
        if let Some(origin) = self
            .extra_origins
            .iter()
            .find(|origin| !valid_origin(origin))
        {
            return Err(format!("{origin} is not an https:// origin."));
        }
        if let Some(module) = self
            .modules
            .iter()
            .find(|module| !Path::new(module).is_absolute())
        {
            return Err(format!("{module} is not a full path to a driver file."));
        }
        Ok(())
    }

    /// Extra driver paths.
    pub(crate) fn module_paths(&self) -> Vec<PathBuf> {
        self.modules.iter().map(PathBuf::from).collect()
    }

    /// The server's view of these settings.
    pub(crate) fn server_settings(&self) -> ServerSettings {
        ServerSettings {
            greeting_version: self.greeting_version.trim().to_owned(),
            origins: OriginPolicy::new(&self.extra_origins),
        }
    }
}

/// An `https://host` origin with a plain host name.
fn valid_origin(origin: &str) -> bool {
    origin.trim().strip_prefix("https://").is_some_and(|host| {
        !host.is_empty()
            && host
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | ':'))
    })
}
