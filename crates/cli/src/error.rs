//! One error type for every command.

use swakshar_cms::CmsError;
use swakshar_server::ServerError;
use swakshar_tls::TlsError;
use swakshar_token::TokenError;

/// Why a command failed.
#[derive(Debug, thiserror::Error)]
pub(crate) enum CliError {
    /// Terminal or file I/O.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// Token access.
    #[error(transparent)]
    Token(#[from] TokenError),
    /// Local certificate.
    #[error(transparent)]
    Tls(#[from] TlsError),
    /// Loopback server.
    #[error(transparent)]
    Server(#[from] ServerError),
    /// Signature structure.
    #[error(transparent)]
    Cms(#[from] CmsError),
    /// Anything else, already phrased for people.
    #[error("{0}")]
    Message(String),
}

impl From<rustls::Error> for CliError {
    /// TLS client configuration or handshake failure.
    fn from(error: rustls::Error) -> Self {
        Self::Message(format!("TLS error: {error}"))
    }
}

impl From<tokio_tungstenite::tungstenite::Error> for CliError {
    /// WebSocket failure while probing.
    fn from(error: tokio_tungstenite::tungstenite::Error) -> Self {
        Self::Message(format!("WebSocket error: {error}"))
    }
}
