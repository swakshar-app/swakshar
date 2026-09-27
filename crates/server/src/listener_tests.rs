//! The listener: stopping it closes open connections, not only the port.

use std::net::Ipv4Addr;
use std::sync::Arc;
use std::time::Duration;

use tokio::io::AsyncReadExt as _;
use tokio::net::{TcpListener, TcpStream};

use super::serve;
use crate::broker::{Broker, PortalRequest, ServerEvent};
use crate::settings::{ServerSettings, tls_config};
use crate::stop::{StopSignal, stop_pair};

/// Declines everything; the test never gets that far.
struct Decline;

impl Broker for Decline {
    /// Declines.
    async fn handle(&self, _request: PortalRequest) -> String {
        swakshar_protocol::REPLY_CANCELED.to_owned()
    }

    /// Ignores events.
    fn notify(&self, _event: ServerEvent) {}
}

/// Serves on a free loopback port and returns the task and the port.
async fn start(
    signal: StopSignal,
) -> (tokio::task::JoinHandle<Result<(), crate::ServerError>>, u16) {
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let identity = rcgen::generate_simple_self_signed(vec!["127.0.0.1".to_owned()]).unwrap();
    let tls = tls_config(identity.cert.der(), &identity.signing_key.serialize_der()).unwrap();
    let server = tokio::spawn(serve(
        listener,
        port,
        tls,
        ServerSettings::default(),
        Arc::new(Decline),
        signal,
    ));
    (server, port)
}

/// Turning signing off drops the server; a page already connected must be
/// cut off too, so it cannot send a request to a signer that is off.
#[tokio::test]
async fn stopping_the_server_closes_open_connections() {
    let (_stopper, signal) = stop_pair();
    let (server, port) = start(signal).await;
    let mut client = TcpStream::connect((Ipv4Addr::LOCALHOST, port))
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;
    server.abort();
    let mut byte = [0_u8; 1];
    let read = tokio::time::timeout(Duration::from_secs(2), client.read(&mut byte)).await;
    assert!(
        matches!(read, Ok(Ok(0) | Err(_))),
        "the connection stayed open after the server stopped"
    );
}

/// Asking the server to stop frees the port at once and ends the server
/// within the drain time, even with a page still connected mid-handshake.
#[tokio::test]
async fn stopping_frees_the_port_and_ends_the_server() {
    let (stopper, signal) = stop_pair();
    let (server, port) = start(signal).await;
    let _client = TcpStream::connect((Ipv4Addr::LOCALHOST, port))
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;
    stopper.stop();
    let ended = tokio::time::timeout(Duration::from_secs(3), server).await;
    assert!(
        matches!(ended, Ok(Ok(Ok(())))),
        "the server did not end: {ended:?}"
    );
    TcpListener::bind((Ipv4Addr::LOCALHOST, port))
        .await
        .expect("the port was still taken after stopping");
}
