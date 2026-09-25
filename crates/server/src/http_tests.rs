//! Routing tests over hand-written request heads.

use swakshar_protocol::OriginPolicy;

use super::{Route, parse_head, route, switching_protocols};

/// Port used in every test.
const PORT: u16 = 1585;

/// Parses and routes a raw head.
fn routed(raw: &str) -> Route {
    route(
        &parse_head(raw.as_bytes()).unwrap(),
        PORT,
        &OriginPolicy::default(),
    )
}

/// The portal's upgrade request is accepted.
#[test]
fn upgrades_gst_origin() {
    let raw = "GET / HTTP/1.1\r\nHost: 127.0.0.1:1585\r\nUpgrade: websocket\r\nConnection: keep-alive, Upgrade\r\nSec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\nSec-WebSocket-Version: 13\r\nOrigin: https://services.gst.gov.in";
    assert_eq!(
        routed(raw),
        Route::WebSocket {
            key: "dGhlIHNhbXBsZSBub25jZQ==".to_owned(),
            origin: "https://services.gst.gov.in".to_owned()
        }
    );
}

/// Any other origin, or a foreign host, is refused.
#[test]
fn refuses_foreign_origins_and_hosts() {
    let evil = "GET / HTTP/1.1\r\nHost: 127.0.0.1:1585\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: a2V5\r\nSec-WebSocket-Version: 13\r\nOrigin: https://evil.example";
    assert_eq!(routed(evil), Route::Forbidden("origin"));
    let rebinding = "GET / HTTP/1.1\r\nHost: attacker.example:1585\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nOrigin: https://services.gst.gov.in";
    assert_eq!(routed(rebinding), Route::Forbidden("host"));
}

/// Navigations see the status page; cross-site fetches do not.
#[test]
fn serves_status_page_to_navigations_only() {
    let navigate = "GET / HTTP/1.1\r\nHost: localhost:1585\r\nSec-Fetch-Mode: navigate\r\nSec-Fetch-Dest: document";
    assert_eq!(routed(navigate), Route::StatusPage);
    let probe = "GET / HTTP/1.1\r\nHost: 127.0.0.1:1585\r\nSec-Fetch-Mode: no-cors\r\nSec-Fetch-Dest: empty";
    assert_eq!(routed(probe), Route::NotFound);
    assert_eq!(
        routed("POST / HTTP/1.1\r\nHost: 127.0.0.1:1585"),
        Route::NotFound
    );
}

/// The accept key matches RFC 6455's worked example.
#[test]
fn derives_accept_key() {
    assert!(
        switching_protocols("dGhlIHNhbXBsZSBub25jZQ==").contains("s3pPLMBiTxaQ9kYGzzhZRbK+xOo=")
    );
}
