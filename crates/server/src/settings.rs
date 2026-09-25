//! Server settings and the rustls configuration.

use std::sync::Arc;

use rustls::ServerConfig;
use rustls_pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use swakshar_protocol::{DEFAULT_GREETING_VERSION, OriginPolicy};

use crate::error::ServerError;

/// The only ALPN protocol offered; browsers run WebSockets over HTTP/1.1.
const ALPN_HTTP_1_1: &[u8] = b"http/1.1";

/// What the server needs to know besides its socket and certificate.
#[derive(Debug, Clone)]
pub struct ServerSettings {
    /// Greeting `version`; must equal the portal's `LATEST_EM_VERSION`.
    pub greeting_version: String,
    /// Which page origins may connect.
    pub origins: OriginPolicy,
}

impl Default for ServerSettings {
    /// The August 2026 greeting version and GST origins only.
    fn default() -> Self {
        Self {
            greeting_version: DEFAULT_GREETING_VERSION.to_owned(),
            origins: OriginPolicy::default(),
        }
    }
}

/// Builds a TLS 1.2/1.3 server config from the leaf certificate and its
/// PKCS#8 key, using the ring provider.
///
/// # Errors
///
/// Returns [`ServerError::Tls`] when the key does not match the certificate.
pub fn tls_config(cert_der: &[u8], key_der: &[u8]) -> Result<Arc<ServerConfig>, ServerError> {
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let mut config = ServerConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()?
        .with_no_client_auth()
        .with_single_cert(
            vec![CertificateDer::from(cert_der.to_vec())],
            PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(key_der.to_vec())),
        )?;
    config.alpn_protocols = vec![ALPN_HTTP_1_1.to_vec()];
    Ok(Arc::new(config))
}
