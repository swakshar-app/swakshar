//! Errors from the loopback server.

/// Something went wrong serving a connection.
#[derive(Debug, thiserror::Error)]
pub enum ServerError {
    /// Socket I/O failed.
    #[error("network error: {0}")]
    Io(#[from] std::io::Error),
    /// TLS configuration or handshake failed.
    #[error("TLS error: {0}")]
    Tls(String),
    /// None of the portal's ports is free.
    #[error("all signer ports (1585, 2095, 2568, 2868, 4587) are in use")]
    NoFreePort,
    /// A peer was too slow.
    #[error("the connection timed out")]
    Timeout,
    /// The peer broke the HTTP or WebSocket protocol.
    #[error("protocol error: {0}")]
    Protocol(&'static str),
    /// The WebSocket failed after the upgrade.
    #[error("WebSocket error: {0}")]
    WebSocket(String),
}

impl From<rustls::Error> for ServerError {
    /// TLS configuration failure.
    fn from(error: rustls::Error) -> Self {
        Self::Tls(error.to_string())
    }
}

impl From<tokio_tungstenite::tungstenite::Error> for ServerError {
    /// WebSocket failure after the upgrade.
    fn from(error: tokio_tungstenite::tungstenite::Error) -> Self {
        Self::WebSocket(error.to_string())
    }
}
