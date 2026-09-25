//! Identity tests: minting, reloading, file modes and the CA's constraints.

use std::path::PathBuf;

use der::Decode;
use x509_cert::Certificate;

use super::ensure_identity;

/// 2026-09-25T00:00:00Z.
const NOW: i64 = 1_790_294_400;
/// `nameConstraints`.
const OID_NAME_CONSTRAINTS: &str = "2.5.29.30";
/// `basicConstraints`.
const OID_BASIC_CONSTRAINTS: &str = "2.5.29.19";

/// A fresh directory under the system temp dir.
fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("swakshar-tls-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

/// Mints once, then reloads the same identity.
#[test]
fn mints_then_reuses() {
    let dir = scratch("reuse");
    let (first, minted) = ensure_identity(&dir, NOW).unwrap();
    assert!(minted);
    let (second, minted_again) = ensure_identity(&dir, NOW).unwrap();
    assert!(!minted_again);
    assert_eq!(first.cert_der, second.cert_der);
    assert_eq!(first.ca_sha1.len(), 40);
    std::fs::remove_dir_all(&dir).unwrap();
}

/// Renews when the leaf is inside the 30-day window.
#[test]
fn renews_near_expiry() {
    let dir = scratch("renew");
    let (identity, _) = ensure_identity(&dir, NOW).unwrap();
    let (renewed, minted) = ensure_identity(&dir, identity.not_after - 86_400).unwrap();
    assert!(minted);
    assert_ne!(identity.cert_der, renewed.cert_der);
    std::fs::remove_dir_all(&dir).unwrap();
}

/// The CA carries basic and name constraints; the key file is owner-only.
#[test]
fn constrains_the_ca() {
    let dir = scratch("constraints");
    let (identity, _) = ensure_identity(&dir, NOW).unwrap();
    let ca = Certificate::from_der(&identity.ca_der).unwrap();
    let oids: Vec<String> = ca
        .tbs_certificate
        .extensions
        .iter()
        .flatten()
        .map(|extension| extension.extn_id.to_string())
        .collect();
    assert!(oids.contains(&OID_NAME_CONSTRAINTS.to_owned()));
    assert!(oids.contains(&OID_BASIC_CONSTRAINTS.to_owned()));
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        let mode = std::fs::metadata(dir.join("server-key.pem"))
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o600);
    }
    std::fs::remove_dir_all(&dir).unwrap();
}
