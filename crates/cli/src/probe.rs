//! `probe`: behave exactly like the GST portal page against a running signer.
//!
//! It tries the portal's ports in order, validates the greeting the way the
//! portal's JavaScript does, sends the portal's request frame, slices the
//! reply the same way, and verifies the CMS. It sends a GST `Origin` on
//! purpose: it stands in for the portal page.

use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;

use futures_util::{SinkExt as _, StreamExt as _};
use rustls::{ClientConfig, RootCertStore};
use rustls_pki_types::{CertificateDer, ServerName};
use swakshar_cms::{from_portal_base64, inspect_signed_data};
use swakshar_protocol::{
    DEFAULT_GREETING_VERSION, SIGNER_ID, SIGNER_PORTS, portal_extract_signature,
};
use swakshar_tls::{TlsError, data_dir, load_identity, tls_dir};
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;
use tokio_tungstenite::client_async;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::handshake::client::generate_key;
use tokio_tungstenite::tungstenite::http::Request;

use crate::args::{Options, ProbeArgs};
use crate::error::CliError;

/// Origin the probe presents, as the portal page would.
const PROBE_ORIGIN: &str = "https://services.gst.gov.in";

/// What happened on one port.
enum Outcome {
    /// Signed and verified; the holder's name.
    Signed(String),
    /// The greeting failed the portal's check.
    WrongGreeting(String),
    /// The signer answered with a failure frame.
    Refused(String),
}

/// Probes the configured port, or all portal ports in order.
pub(crate) async fn probe(options: &Options, args: &ProbeArgs) -> Result<(), CliError> {
    let data_dir = data_dir().ok_or(TlsError::NoDataDir)?;
    let identity = load_identity(&tls_dir(&data_dir))?;
    let connector = connector(&identity.ca_der)?;
    let version = options
        .greeting_version
        .as_deref()
        .unwrap_or(DEFAULT_GREETING_VERSION);
    let ports = options
        .port
        .map_or_else(|| SIGNER_PORTS.to_vec(), |port| vec![port]);
    for port in ports {
        println!("Trying port {port}...");
        match try_port(&connector, port, version, args).await {
            Ok(Outcome::Signed(holder)) => {
                println!("PASS: signed by {holder}; the portal would accept this reply's layout.");
                return Ok(());
            }
            Ok(Outcome::WrongGreeting(greeting)) => {
                println!("  the portal would reject this greeting: {greeting:?}")
            }
            Ok(Outcome::Refused(reply)) => {
                println!("The signer answered {reply:?}.");
                return Ok(());
            }
            Err(error) => println!("  {error}"),
        }
    }
    Err(CliError::Message(
        "no signer answered the way the portal expects".to_owned(),
    ))
}

/// One port: connect, check the greeting, request, verify.
async fn try_port(
    connector: &TlsConnector,
    port: u16,
    version: &str,
    args: &ProbeArgs,
) -> Result<Outcome, CliError> {
    let tcp = TcpStream::connect((Ipv4Addr::LOCALHOST, port)).await?;
    let tls = connector
        .connect(ServerName::from(IpAddr::V4(Ipv4Addr::LOCALHOST)), tcp)
        .await?;
    let request = Request::builder()
        .uri(format!("wss://127.0.0.1:{port}/"))
        .header("Host", format!("127.0.0.1:{port}"))
        .header("Origin", PROBE_ORIGIN)
        .header("Connection", "Upgrade")
        .header("Upgrade", "websocket")
        .header("Sec-WebSocket-Version", "13")
        .header("Sec-WebSocket-Key", generate_key())
        .body(())
        .map_err(|error| CliError::Message(error.to_string()))?;
    let (mut socket, _) = client_async(request, tls).await?;
    let greeting = next_text(&mut socket).await?;
    if !greeting_accepted(&greeting, version) {
        return Ok(Outcome::WrongGreeting(greeting));
    }
    socket.send(Message::text(portal_frame(args))).await?;
    let reply = next_text(&mut socket).await?;
    let _ = socket.close(None).await;
    let Some(signature) = portal_extract_signature(&reply) else {
        return Ok(Outcome::Refused(reply));
    };
    let inspection = inspect_signed_data(&from_portal_base64(signature)?)?;
    if inspection.content != args.content.as_bytes() {
        return Err(CliError::Message(
            "the signed content differs from the request".to_owned(),
        ));
    }
    Ok(Outcome::Signed(holder_name(&inspection.certificate)))
}

/// The portal's greeting check: after splitting lines on `=`, both
/// `version` and `ID` must match.
fn greeting_accepted(greeting: &str, version: &str) -> bool {
    let matched = greeting
        .split('\n')
        .filter(|line| {
            let mut parts = line.split('=');
            match (parts.next(), parts.next()) {
                (Some(key), Some(value)) => {
                    (key.trim() == "version" && value.trim() == version)
                        || (key.trim() == "ID" && value.trim() == SIGNER_ID)
                }
                _ => false,
            }
        })
        .count();
    matched == 2
}

/// The request frame, field for field as the portal builds it.
fn portal_frame(args: &ProbeArgs) -> String {
    format!(
        "action=sign\ntobesigned={}\npanNo={}\nsigntype=1\nexpirycheck=true\nissuername=\ncertclass=2|3\ncerttype=DSC\ncertdetails=",
        args.content,
        args.pan.as_deref().unwrap_or_default()
    )
}

/// Next text frame, skipping control frames.
async fn next_text<S>(
    socket: &mut tokio_tungstenite::WebSocketStream<S>,
) -> Result<String, CliError>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    while let Some(frame) = socket.next().await {
        if let Message::Text(text) = frame? {
            return Ok(text.as_str().to_owned());
        }
    }
    Err(CliError::Message(
        "the signer closed the connection".to_owned(),
    ))
}

/// A TLS client that trusts only this install's local CA.
fn connector(ca_der: &[u8]) -> Result<TlsConnector, CliError> {
    let mut roots = RootCertStore::empty();
    roots.add(CertificateDer::from(ca_der.to_vec()))?;
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let mut config = ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()?
        .with_root_certificates(roots)
        .with_no_client_auth();
    config.alpn_protocols = vec![b"http/1.1".to_vec()];
    Ok(TlsConnector::from(Arc::new(config)))
}

/// Subject common name of the signer certificate, for display.
fn holder_name(certificate: &swakshar_cms::Certificate) -> String {
    use der::Encode as _;
    certificate
        .to_der()
        .ok()
        .and_then(|der| swakshar_token::summarize(&der).ok())
        .map_or_else(
            || "an unnamed certificate".to_owned(),
            |summary| summary.subject_cn,
        )
}

#[cfg(test)]
mod tests {
    use super::greeting_accepted;

    /// Mirrors the portal: version and ID must both match.
    #[test]
    fn checks_greeting_like_the_portal() {
        let greeting = "status = success\nport = 1585\nversion = 2.8\nID = gstnInfy";
        assert!(greeting_accepted(greeting, "2.8"));
        assert!(!greeting_accepted(greeting, "2.9"));
        assert!(!greeting_accepted("status = success\nversion = 2.8", "2.8"));
    }
}
