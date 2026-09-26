//! Binding one of the portal's ports and accepting connections.

use std::net::Ipv4Addr;
use std::sync::Arc;

use rustls::ServerConfig;
use swakshar_protocol::SIGNER_PORTS;
use tokio::net::TcpListener;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;
use tokio_rustls::TlsAcceptor;

use crate::broker::Broker;
use crate::connection::{Shared, handle_connection};
use crate::error::ServerError;
use crate::settings::ServerSettings;

/// Most connections served at once; the portal needs one.
const MAX_CONNECTIONS: usize = 32;

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

/// Accepts connections until the task is dropped or the socket fails.
/// Connections live in a `JoinSet` owned by this future, so dropping it
/// (turning signing off, quitting) closes every open connection as well as
/// the port, and no page can send a request to a signer that is off.
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
) -> Result<(), ServerError> {
    let shared = Arc::new(Shared {
        port,
        settings,
        broker,
        acceptor: TlsAcceptor::from(tls),
    });
    let slots = Arc::new(Semaphore::new(MAX_CONNECTIONS));
    let mut connections = JoinSet::new();
    loop {
        let (tcp, _) = listener.accept().await?;
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
}
