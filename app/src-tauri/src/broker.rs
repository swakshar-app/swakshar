//! The app's broker: requests go to the approval window, events to diagnostics.

use swakshar_server::{Broker, PortalRequest, ServerEvent};
use tauri::{AppHandle, Manager as _};

use crate::pending;
use crate::state::{AppState, lock, unix_now};

/// Decides requests through the approval window.
pub(crate) struct UiBroker {
    /// App handle.
    app: AppHandle,
}

impl UiBroker {
    /// A broker bound to this app.
    pub(crate) fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

impl Broker for UiBroker {
    /// Shows the approval window and waits for the user.
    async fn handle(&self, request: PortalRequest) -> String {
        pending::run(&self.app, request).await
    }

    /// Records what the Help view needs to explain problems.
    fn notify(&self, event: ServerEvent) {
        let state = self.app.state::<AppState>();
        let mut diagnostics = lock(&state.diagnostics);
        match event {
            ServerEvent::Connected { origin } => {
                log::info!("{origin} connected");
                diagnostics.last_connection = Some((origin, unix_now()));
            }
            ServerEvent::Rejected { origin, reason } => {
                log::warn!("refused a connection from {origin:?}: {reason}");
            }
            ServerEvent::TlsFailed { reason } => {
                log::warn!("a browser refused the local certificate: {reason}");
                diagnostics.last_tls_failure = Some((reason, unix_now()));
            }
            ServerEvent::StatusPageServed => {
                diagnostics.status_page_seen = true;
            }
        }
    }
}
