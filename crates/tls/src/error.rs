//! Errors from creating, storing or trusting the local certificate.

/// Something went wrong with the local TLS identity.
#[derive(Debug, thiserror::Error)]
pub enum TlsError {
    /// Reading or writing the certificate files failed.
    #[error("file error: {0}")]
    Io(#[from] std::io::Error),
    /// Generating keys or certificates failed.
    #[error("could not generate the local certificate: {0}")]
    Generate(String),
    /// A stored file is not valid PEM.
    #[error("stored certificate files are unreadable: {0}")]
    Pem(String),
    /// A stored certificate does not parse.
    #[error("stored certificate is invalid: {0}")]
    Certificate(String),
    /// The per-user data directory could not be determined.
    #[error("could not find a per-user data directory")]
    NoDataDir,
    /// An OS trust command failed or was cancelled.
    #[error("{command} did not complete: {detail}")]
    Command {
        /// The command that ran.
        command: &'static str,
        /// Exit status or error text.
        detail: String,
    },
    /// Trust management is not implemented on this OS yet.
    #[error("installing the local certificate is not supported on this system yet")]
    Unsupported,
}

impl From<rcgen::Error> for TlsError {
    /// Key or certificate generation failure.
    fn from(error: rcgen::Error) -> Self {
        Self::Generate(error.to_string())
    }
}

impl From<time::error::ComponentRange> for TlsError {
    /// A validity date out of range.
    fn from(error: time::error::ComponentRange) -> Self {
        Self::Generate(error.to_string())
    }
}
