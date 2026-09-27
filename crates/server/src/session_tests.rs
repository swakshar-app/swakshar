//! Session tests over an in-memory pipe, without TLS.

use std::sync::Mutex;

use futures_util::{SinkExt as _, StreamExt as _};
use tokio::io::duplex;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::protocol::Role;

use super::{SessionContext, run_session};
use crate::broker::{Broker, PortalRequest, ServerEvent};
use crate::stop::stop_pair;

/// Records requests and answers with a fixed frame.
#[derive(Default)]
struct Recorder {
    /// Every request received.
    seen: Mutex<Vec<PortalRequest>>,
}

impl Broker for Recorder {
    /// Stores the request and replies with a fixed signature frame.
    async fn handle(&self, request: PortalRequest) -> String {
        self.seen.lock().unwrap().push(request);
        "signature= SIG\nSerialNo= 1".to_owned()
    }

    /// Ignores events.
    fn notify(&self, _event: ServerEvent) {}
}

/// Greeting first, then the request is parsed, brokered and answered.
#[tokio::test]
async fn greets_then_answers() {
    let (server_io, client_io) = duplex(64 * 1024);
    let recorder = Recorder::default();
    let (_stopper, signal) = stop_pair();
    let context = SessionContext {
        port: 1585,
        greeting_version: "2.8",
        origin: "https://services.gst.gov.in".to_owned(),
        broker: &recorder,
        stop: signal,
    };
    let server = WebSocketStream::from_raw_socket(server_io, Role::Server, None).await;
    let mut client = WebSocketStream::from_raw_socket(client_io, Role::Client, None).await;
    let client_side = async {
        let greeting = client.next().await.unwrap().unwrap();
        assert_eq!(
            greeting.to_text().unwrap(),
            "status = success\nport = 1585\nversion = 2.8\nID = gstnInfy"
        );
        client
            .send(Message::text(
                "action=sign\ntobesigned=ABCDE1234F\npanNo=ABCDE1234F\nsigntype=1",
            ))
            .await
            .unwrap();
        let reply = client.next().await.unwrap().unwrap();
        assert_eq!(reply.to_text().unwrap(), "signature= SIG\nSerialNo= 1");
        client.close(None).await.unwrap();
    };
    let (served, ()) = tokio::join!(run_session(server, &context), client_side);
    served.unwrap();
    let seen = recorder.seen.lock().unwrap();
    assert_eq!(seen.len(), 1);
    assert_eq!(seen[0].request.content, "ABCDE1234F");
    assert_eq!(seen[0].origin, "https://services.gst.gov.in");
}

/// A frame that is not a sign request gets `signing failed`.
#[tokio::test]
async fn refuses_other_actions() {
    let (server_io, client_io) = duplex(64 * 1024);
    let recorder = Recorder::default();
    let (_stopper, signal) = stop_pair();
    let context = SessionContext {
        port: 1585,
        greeting_version: "2.8",
        origin: "https://services.gst.gov.in".to_owned(),
        broker: &recorder,
        stop: signal,
    };
    let server = WebSocketStream::from_raw_socket(server_io, Role::Server, None).await;
    let mut client = WebSocketStream::from_raw_socket(client_io, Role::Client, None).await;
    let client_side = async {
        client.next().await.unwrap().unwrap();
        client.send(Message::text("action=verify")).await.unwrap();
        let reply = client.next().await.unwrap().unwrap();
        assert_eq!(reply.to_text().unwrap(), "signing failed");
        client.close(None).await.unwrap();
    };
    let (served, ()) = tokio::join!(run_session(server, &context), client_side);
    served.unwrap();
    assert!(recorder.seen.lock().unwrap().is_empty());
}

/// Never answers, like a user who has not pressed Sign yet.
struct Waiting;

impl Broker for Waiting {
    /// Waits forever.
    async fn handle(&self, _request: PortalRequest) -> String {
        std::future::pending().await
    }

    /// Ignores events.
    fn notify(&self, _event: ServerEvent) {}
}

/// A request still waiting when the signer stops is answered `signing
/// canceled`, then the page gets a close frame instead of a dropped socket.
#[tokio::test]
async fn stopping_cancels_a_waiting_request_and_closes() {
    let (server_io, client_io) = duplex(64 * 1024);
    let (stopper, signal) = stop_pair();
    let context = SessionContext {
        port: 1585,
        greeting_version: "2.8",
        origin: "https://services.gst.gov.in".to_owned(),
        broker: &Waiting,
        stop: signal,
    };
    let server = WebSocketStream::from_raw_socket(server_io, Role::Server, None).await;
    let mut client = WebSocketStream::from_raw_socket(client_io, Role::Client, None).await;
    let client_side = async {
        client.next().await.unwrap().unwrap();
        client
            .send(Message::text(
                "action=sign\ntobesigned=ABCDE1234F\npanNo=ABCDE1234F\nsigntype=1",
            ))
            .await
            .unwrap();
        stopper.stop();
        let reply = client.next().await.unwrap().unwrap();
        assert_eq!(reply.to_text().unwrap(), swakshar_protocol::REPLY_CANCELED);
        let close = client.next().await.unwrap().unwrap();
        assert!(close.is_close(), "expected a close frame, got {close:?}");
    };
    let (served, ()) = tokio::join!(run_session(server, &context), client_side);
    served.unwrap();
}

/// An idle page gets a close frame as soon as the signer stops.
#[tokio::test]
async fn stopping_closes_an_idle_page() {
    let (server_io, client_io) = duplex(64 * 1024);
    let recorder = Recorder::default();
    let (stopper, signal) = stop_pair();
    let context = SessionContext {
        port: 1585,
        greeting_version: "2.8",
        origin: "https://services.gst.gov.in".to_owned(),
        broker: &recorder,
        stop: signal,
    };
    let server = WebSocketStream::from_raw_socket(server_io, Role::Server, None).await;
    let mut client = WebSocketStream::from_raw_socket(client_io, Role::Client, None).await;
    let client_side = async {
        client.next().await.unwrap().unwrap();
        stopper.stop();
        let close = client.next().await.unwrap().unwrap();
        assert!(close.is_close(), "expected a close frame, got {close:?}");
    };
    let (served, ()) = tokio::join!(run_session(server, &context), client_side);
    served.unwrap();
    assert!(recorder.seen.lock().unwrap().is_empty());
}
