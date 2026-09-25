//! Just enough HTTP/1.1 to route a request before the WebSocket upgrade.

use swakshar_protocol::OriginPolicy;
use tokio::io::{AsyncRead, AsyncReadExt as _, AsyncWrite, AsyncWriteExt as _};
use tokio_tungstenite::tungstenite::handshake::derive_accept_key;

use crate::error::ServerError;

/// Largest request head accepted.
const MAX_HEAD_BYTES: usize = 8 * 1024;
/// End of an HTTP head.
const HEAD_END: &[u8] = b"\r\n\r\n";
/// The only WebSocket version browsers speak.
const WEBSOCKET_VERSION: &str = "13";
/// 200 status line.
pub(crate) const STATUS_OK: &str = "200 OK";
/// 403 status line.
pub(crate) const STATUS_FORBIDDEN: &str = "403 Forbidden";
/// 404 status line.
pub(crate) const STATUS_NOT_FOUND: &str = "404 Not Found";

/// A parsed request line and headers (names lowercased).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RequestHead {
    /// Request method.
    pub(crate) method: String,
    /// Request target.
    pub(crate) path: String,
    /// Header names (lowercase) and values.
    headers: Vec<(String, String)>,
}

impl RequestHead {
    /// First value of a header, by lowercase name.
    pub(crate) fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    }
}

/// What to do with a request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Route {
    /// Upgrade: an allowed page wants the signer.
    WebSocket {
        /// `Sec-WebSocket-Key`.
        key: String,
        /// The allowed origin.
        origin: String,
    },
    /// A browser navigated to `/`: show the status page.
    StatusPage,
    /// Refuse, naming the failed check.
    Forbidden(&'static str),
    /// Anything else.
    NotFound,
}

/// Reads the request head. Refuses bytes after the head: browsers wait for
/// the `101` before sending frames, and nothing else is expected.
pub(crate) async fn read_head<S: AsyncRead + Unpin>(
    stream: &mut S,
) -> Result<RequestHead, ServerError> {
    let mut buffer = Vec::with_capacity(1_024);
    let mut chunk = [0_u8; 1_024];
    loop {
        let read = stream.read(&mut chunk).await?;
        if read == 0 {
            return Err(ServerError::Protocol(
                "connection closed before the request head",
            ));
        }
        buffer.extend_from_slice(chunk.get(..read).unwrap_or_default());
        if let Some(end) = find(&buffer, HEAD_END) {
            if end + HEAD_END.len() != buffer.len() {
                return Err(ServerError::Protocol("data sent before the upgrade"));
            }
            return parse_head(buffer.get(..end).unwrap_or_default());
        }
        if buffer.len() > MAX_HEAD_BYTES {
            return Err(ServerError::Protocol("request head too large"));
        }
    }
}

/// Parses a request line and headers.
pub(crate) fn parse_head(bytes: &[u8]) -> Result<RequestHead, ServerError> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| ServerError::Protocol("request head is not UTF-8"))?;
    let mut lines = text.split("\r\n");
    let mut parts = lines.next().unwrap_or_default().split(' ');
    let (Some(method), Some(path), Some(version)) = (parts.next(), parts.next(), parts.next())
    else {
        return Err(ServerError::Protocol("malformed request line"));
    };
    if !version.starts_with("HTTP/1.") {
        return Err(ServerError::Protocol("unsupported HTTP version"));
    }
    let headers = lines
        .filter_map(|line| line.split_once(':'))
        .map(|(name, value)| (name.trim().to_ascii_lowercase(), value.trim().to_owned()))
        .collect();
    Ok(RequestHead {
        method: method.to_owned(),
        path: path.to_owned(),
        headers,
    })
}

/// Routes a request: `Host` first, then upgrade with an allowed `Origin`,
/// then the status page for top-level navigations only.
pub(crate) fn route(head: &RequestHead, port: u16, origins: &OriginPolicy) -> Route {
    if !OriginPolicy::allows_host(head.header("host").unwrap_or_default(), port) {
        return Route::Forbidden("host");
    }
    if head.method != "GET" {
        return Route::NotFound;
    }
    if !wants_upgrade(head) {
        return if head.path == "/" && is_navigation(head) {
            Route::StatusPage
        } else {
            Route::NotFound
        };
    }
    let origin = head.header("origin").unwrap_or_default();
    if !origins.allows(origin) {
        return Route::Forbidden("origin");
    }
    match (
        head.header("sec-websocket-key"),
        head.header("sec-websocket-version"),
    ) {
        (Some(key), Some(WEBSOCKET_VERSION)) if !key.is_empty() => Route::WebSocket {
            key: key.to_owned(),
            origin: origin.to_owned(),
        },
        _ => Route::Forbidden("websocket handshake"),
    }
}

/// The `101` response completing the upgrade.
pub(crate) fn switching_protocols(key: &str) -> String {
    format!(
        "HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: {}\r\n\r\n",
        derive_accept_key(key.as_bytes())
    )
}

/// Writes a complete response and closes the stream. No CORS headers, ever.
pub(crate) async fn write_response<S: AsyncWrite + Unpin>(
    stream: &mut S,
    status: &str,
    body: Option<&str>,
) -> Result<(), ServerError> {
    let body = body.unwrap_or_default();
    let head = format!(
        "HTTP/1.1 {status}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nContent-Security-Policy: default-src 'none'; style-src 'unsafe-inline'\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\nReferrer-Policy: no-referrer\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream.write_all(head.as_bytes()).await?;
    stream.write_all(body.as_bytes()).await?;
    stream.flush().await?;
    stream.shutdown().await?;
    Ok(())
}

/// True for a WebSocket upgrade request.
fn wants_upgrade(head: &RequestHead) -> bool {
    head.header("upgrade")
        .is_some_and(|value| value.eq_ignore_ascii_case("websocket"))
        && head.header("connection").is_some_and(|value| {
            value
                .split(',')
                .any(|token| token.trim().eq_ignore_ascii_case("upgrade"))
        })
}

/// True for top-level navigations, so other sites cannot probe the page
/// with `fetch`. Browsers without Fetch Metadata count as navigations.
fn is_navigation(head: &RequestHead) -> bool {
    head.header("sec-fetch-mode")
        .is_none_or(|mode| mode.eq_ignore_ascii_case("navigate"))
        && head
            .header("sec-fetch-dest")
            .is_none_or(|dest| dest.eq_ignore_ascii_case("document"))
}

/// Position of `needle` in `haystack`.
fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

#[cfg(test)]
#[path = "http_tests.rs"]
mod tests;
