//! The WebSocket conversation: greeting, one request, one reply.

use std::time::Duration;

use futures_util::stream::SplitStream;
use futures_util::{SinkExt as _, StreamExt as _};
use swakshar_protocol::{REPLY_FAILED, greeting, parse_request};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::time::timeout;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::Message;

use crate::broker::{Broker, PortalRequest};
use crate::error::ServerError;

/// Longest a connection may sit idle between frames.
const IDLE_TIMEOUT: Duration = Duration::from_secs(10 * 60);

/// Everything a session needs, borrowed from the server.
pub(crate) struct SessionContext<'a, B> {
    /// Port, echoed in the greeting.
    pub(crate) port: u16,
    /// Greeting `version`.
    pub(crate) greeting_version: &'a str,
    /// Checked page origin.
    pub(crate) origin: String,
    /// Who decides requests.
    pub(crate) broker: &'a B,
}

/// Sends the greeting, then answers each text frame until the page leaves.
pub(crate) async fn run_session<S, B>(
    socket: WebSocketStream<S>,
    context: &SessionContext<'_, B>,
) -> Result<(), ServerError>
where
    S: AsyncRead + AsyncWrite + Unpin + Send,
    B: Broker,
{
    let (mut sink, mut stream) = socket.split();
    sink.send(Message::text(greeting(
        context.port,
        context.greeting_version,
    )))
    .await?;
    loop {
        let Ok(next) = timeout(IDLE_TIMEOUT, stream.next()).await else {
            log::info!("closing an idle signer connection");
            return Ok(());
        };
        match next.transpose()? {
            Some(Message::Text(text)) => {
                let Some(reply) = answer(text.as_str(), context, &mut stream).await else {
                    return Ok(());
                };
                sink.send(Message::text(reply)).await?;
            }
            Some(Message::Close(_)) | None => return Ok(()),
            Some(_) => {}
        }
    }
}

/// Decides one frame. Returns `None` when the page disconnected first, which
/// drops the broker's future and with it the pending approval.
async fn answer<S, B>(
    text: &str,
    context: &SessionContext<'_, B>,
    stream: &mut SplitStream<WebSocketStream<S>>,
) -> Option<String>
where
    S: AsyncRead + AsyncWrite + Unpin + Send,
    B: Broker,
{
    let request = match parse_request(text) {
        Ok(request) => request,
        Err(error) => {
            log::warn!("refusing a frame: {error}");
            return Some(REPLY_FAILED.to_owned());
        }
    };
    let portal = PortalRequest {
        origin: context.origin.clone(),
        port: context.port,
        request,
    };
    tokio::select! {
        reply = context.broker.handle(portal) => Some(reply),
        () = closed(stream) => {
            log::info!("the page left before the request was answered");
            None
        }
    }
}

/// Resolves when the page closes the connection or it fails.
async fn closed<S>(stream: &mut SplitStream<WebSocketStream<S>>)
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    while let Some(frame) = stream.next().await {
        match frame {
            Ok(Message::Close(_)) | Err(_) => return,
            Ok(_) => log::debug!("ignoring a frame that arrived while a request was pending"),
        }
    }
}

#[cfg(test)]
#[path = "session_tests.rs"]
mod tests;
