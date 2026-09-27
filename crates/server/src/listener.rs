//! Binding one of the portal's ports and accepting connections.

use std::net::Ipv4Addr;
use std::sync::Arc;
use std::time::Duration;

use rustls::ServerConfig;
use swakshar_protocol::SIGNER_PORTS;
use tokio::net::TcpListener;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;
use tokio::time::timeout;
use tokio_rustls::TlsAcceptor;

use crate::broker::Broker;
use crate::connection::{Shared, handle_connection};
use crate::error::ServerError;
use crate::settings::ServerSettings;
use crate::stop::StopSignal;

/// Most connections served at once; the portal needs one.
const MAX_CONNECTIONS: usize = 32;
/// Longest open connections get, after a stop, to deliver their last reply
/// and close; the rest are cut off.
const DRAIN_TIMEOUT: Duration = Duration::from_secs(1);

/// Binds `127.0.0.1` on the first free portal port, trying `preferred` first
/// when it is one of them. The portal probes the ports in its own fixed order.
///
/// # Errors
///
/// Returns [`ServerError::NoFreePort`] when all five are taken.
pub async fn bind_signer_port(preferred: Option<u16>) -> Result<(TcpListener, u16), ServerError> {
    let preferred = preferred.filter(|port| SIGNER_PORTS.contains(port));
    let order = preferred.into_iter().chain(
        SIGNER_PORTS
            .into_iter()
            .filter(move |port| Some(*port) != preferred),
    );
    for port in order {
        match TcpListener::bind((Ipv4Addr::LOCALHOST, port)).await {
            Ok(listener) => return Ok((listener, port)),
            Err(error) => log::info!("port {port} is not available: {error}"),
        }
    }
    Err(ServerError::NoFreePort)
}

/// Accepts connections until `stop` fires, the task is dropped or the
/// socket fails. On a stop the port closes at once and open connections get
/// [`DRAIN_TIMEOUT`] to send their last reply and a close frame.
/// Connections live in a `JoinSet` owned by this future, so ending or
/// dropping it closes every connection still open, and no page can send a
/// request to a signer that is off.
///
/// # Errors
///
/// Returns [`ServerError::Io`] when accepting fails.
pub async fn serve<B: Broker>(
    listener: TcpListener,
    port: u16,
    tls: Arc<ServerConfig>,
    settings: ServerSettings,
    broker: Arc<B>,
    mut stop: StopSignal,
) -> Result<(), ServerError> {
    let shared = Arc::new(Shared {
        port,
        settings,
        broker,
        acceptor: TlsAcceptor::from(tls),
        stop: stop.clone(),
    });
    let slots = Arc::new(Semaphore::new(MAX_CONNECTIONS));
    let mut connections = JoinSet::new();
    loop {
        let accepted = tokio::select! {
            accepted = listener.accept() => accepted,
            () = stop.stopped() => break,
        };
        let (tcp, _) = accepted?;
        while connections.try_join_next().is_some() {}
        let Ok(permit) = Arc::clone(&slots).try_acquire_owned() else {
            log::warn!("too many open connections; refusing one");
            continue;
        };
        let shared = Arc::clone(&shared);
        connections.spawn(async move {
            if let Err(error) = handle_connection(tcp, &shared).await {
                log::debug!("connection ended: {error}");
            }
            drop(permit);
        });
    }
    drop(listener);
    drain(connections).await;
    Ok(())
}

/// Waits up to [`DRAIN_TIMEOUT`] for connections to finish; dropping the set
/// cuts off the rest.
async fn drain(mut connections: JoinSet<()>) {
    let finished = timeout(DRAIN_TIMEOUT, async {
        while connections.join_next().await.is_some() {}
    })
    .await;
    if finished.is_err() {
        log::info!(
            "cutting off {} connections that did not close in time",
            connections.len()
        );
    }
}

#[cfg(test)]
#[path = "listener_tests.rs"]
mod tests;
