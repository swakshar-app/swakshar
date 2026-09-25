//! The seam between the network and the user: who decides each request.

use std::future::Future;

use swakshar_protocol::SignRequest;

/// A sign request from an allowed page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortalRequest {
    /// The page's origin, already checked against the allowlist.
    pub origin: String,
    /// Port the portal connected to.
    pub port: u16,
    /// The parsed request.
    pub request: SignRequest,
}

/// Connection-level events, for diagnostics and the activity view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerEvent {
    /// An allowed page opened a signer connection.
    Connected {
        /// Page origin.
        origin: String,
    },
    /// A connection was refused before the WebSocket upgrade.
    Rejected {
        /// Page origin, when the request had one.
        origin: Option<String>,
        /// Which check failed.
        reason: &'static str,
    },
    /// The TLS handshake failed, usually because the browser does not trust
    /// the local certificate yet.
    TlsFailed {
        /// The TLS error text.
        reason: String,
    },
    /// A browser loaded the status page, proving it trusts the certificate.
    StatusPageServed,
}

/// Decides sign requests. Implemented by the desktop app (approval window)
/// and by the command-line tool (terminal prompt).
pub trait Broker: Send + Sync + 'static {
    /// Returns the reply frame for a request: a success frame, or
    /// `signing canceled` / `signing failed`. The future is dropped when the
    /// page disconnects first, so implementations must clean up on drop.
    fn handle(&self, request: PortalRequest) -> impl Future<Output = String> + Send;

    /// Observes connection-level events.
    fn notify(&self, event: ServerEvent);
}
