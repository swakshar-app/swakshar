//! Errors returned to the webview, always as plain sentences.

use serde::{Serialize, Serializer};

/// A command failed; the webview receives the message text.
#[derive(Debug, thiserror::Error)]
pub(crate) enum CommandError {
    /// Already phrased for people.
    #[error("{0}")]
    Message(String),
    /// Tauri failure.
    #[error(transparent)]
    Tauri(#[from] tauri::Error),
    /// Token failure.
    #[error(transparent)]
    Token(#[from] swakshar_token::TokenError),
    /// Local certificate failure.
    #[error(transparent)]
    Tls(#[from] swakshar_tls::TlsError),
    /// File failure.
    #[error("could not save: {0}")]
    Io(#[from] std::io::Error),
}

impl Serialize for CommandError {
    /// Serialises as the message string.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

/// Result type of every command.
pub(crate) type CommandResult<T> = Result<T, CommandError>;
