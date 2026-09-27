//! One connection: TLS, the request head, then upgrade, status page or refusal.

use std::sync::Arc;
use std::time::Duration;

use swakshar_protocol::MAX_REQUEST_BYTES;
use tokio::io::AsyncWriteExt as _;
use tokio::net::TcpStream;
use tokio::time::timeout;
use tokio_rustls::TlsAcceptor;
use tokio_rustls::server::TlsStream;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::protocol::{Role, WebSocketConfig};

use crate::broker::{Broker, ServerEvent};
use crate::error::ServerError;
use crate::http::{
    Route, STATUS_FORBIDDEN, STATUS_NOT_FOUND, STATUS_OK, read_head, route, switching_protocols,
    write_response,
};
use crate::session::{SessionContext, run_session};
use crate::settings::ServerSettings;
use crate::status_page::status_page;
use crate::stop::StopSignal;

/// Time allowed for the TLS handshake and the request head.
const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(15);

/// State shared by every connection of one server.
pub(crate) struct Shared<B> {
    /// Port the server listens on.
    pub(crate) port: u16,
    /// Greeting version and origin policy.
    pub(crate) settings: ServerSettings,
    /// Who decides requests.
    pub(crate) broker: Arc<B>,
    /// TLS acceptor for the loopback certificate.
    pub(crate) acceptor: TlsAcceptor,
    /// Fires when the signer stops.
    pub(crate) stop: StopSignal,
}

/// Serves one TCP connection from start to finish.
pub(crate) async fn handle_connection<B: Broker>(
    tcp: TcpStream,
    shared: &Shared<B>,
) -> Result<(), ServerError> {
    tcp.set_nodelay(true)?;
    let accepted = timeout(HANDSHAKE_TIMEOUT, shared.acceptor.accept(tcp))
        .await
        .map_err(|_| ServerError::Timeout)?;
    let mut stream = accepted.map_err(|error| tls_failure(&error, shared))?;
    let head = timeout(HANDSHAKE_TIMEOUT, read_head(&mut stream))
        .await
        .map_err(|_| ServerError::Timeout)?
        .map_err(|error| match error {
            ServerError::Io(io) => tls_failure(&io, shared),
            other => other,
        })?;
    match route(&head, shared.port, &shared.settings.origins) {
        Route::WebSocket { key, origin } => upgrade(stream, &key, origin, shared).await,
        Route::StatusPage => {
            shared.broker.notify(ServerEvent::StatusPageServed);
            write_response(&mut stream, STATUS_OK, Some(&status_page(shared.port))).await
        }
        Route::Forbidden(reason) => {
            shared.broker.notify(ServerEvent::Rejected {
                origin: head.header("origin").map(str::to_owned),
                reason,
            });
            write_response(&mut stream, STATUS_FORBIDDEN, None).await
        }
        Route::NotFound => write_response(&mut stream, STATUS_NOT_FOUND, None).await,
    }
}

/// Completes the upgrade and runs the signer session.
async fn upgrade<B: Broker>(
    mut stream: TlsStream<TcpStream>,
    key: &str,
    origin: String,
    shared: &Shared<B>,
) -> Result<(), ServerError> {
    stream
        .write_all(switching_protocols(key).as_bytes())
        .await?;
    stream.flush().await?;
    let config = WebSocketConfig::default()
        .max_message_size(Some(MAX_REQUEST_BYTES))
        .max_frame_size(Some(MAX_REQUEST_BYTES));
    let socket = WebSocketStream::from_raw_socket(stream, Role::Server, Some(config)).await;
    shared.broker.notify(ServerEvent::Connected {
        origin: origin.clone(),
    });
    let context = SessionContext {
        port: shared.port,
        greeting_version: &shared.settings.greeting_version,
        origin,
        broker: shared.broker.as_ref(),
        stop: shared.stop.clone(),
    };
    run_session(socket, &context).await
}

/// Reports TLS failures. A browser that distrusts the certificate usually
/// finishes the handshake and then sends an alert, so read errors carrying a
/// rustls alert count too.
fn tls_failure<B: Broker>(error: &std::io::Error, shared: &Shared<B>) -> ServerError {
    let alert = error
        .get_ref()
        .and_then(|inner| inner.downcast_ref::<rustls::Error>())
        .is_some_and(|inner| matches!(inner, rustls::Error::AlertReceived(_)));
    if alert || error.kind() == std::io::ErrorKind::InvalidData {
        shared.broker.notify(ServerEvent::TlsFailed {
            reason: error.to_string(),
        });
    }
    ServerError::Tls(error.to_string())
}
