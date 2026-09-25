//! Minting the per-install CA and the loopback leaf certificate.

use rcgen::{
    BasicConstraints, CertificateParams, CidrSubnet, DnType, ExtendedKeyUsagePurpose,
    GeneralSubtree, IsCa, Issuer, KeyPair, KeyUsagePurpose, NameConstraints,
};
use time::{Duration, OffsetDateTime};

use crate::error::TlsError;

/// Leaf validity. Apple rejects TLS server certificates valid for over 825 days.
const VALIDITY_DAYS: i64 = 800;
/// Backdating that absorbs small clock differences.
const BACKDATE_SECONDS: i64 = 86_400;
/// The loopback address the portal connects to.
const LOOPBACK_V4: [u8; 4] = [127, 0, 0, 1];
/// Loopback host name.
const LOCALHOST: &str = "localhost";

/// Freshly minted PEM files. The CA private key is not among them.
pub(crate) struct Generated {
    /// CA certificate, to be trusted by the OS.
    pub(crate) ca_pem: String,
    /// Leaf certificate served on loopback.
    pub(crate) cert_pem: String,
    /// Leaf private key (PKCS#8).
    pub(crate) key_pem: String,
}

/// Mints a name-constrained CA and a loopback leaf. The CA key lives only in
/// this function and is dropped when it returns.
pub(crate) fn generate(now: i64) -> Result<Generated, TlsError> {
    let not_before = OffsetDateTime::from_unix_timestamp(now - BACKDATE_SECONDS)?;
    let not_after = not_before + Duration::days(VALIDITY_DAYS);
    let ca_key = KeyPair::generate()?;
    let ca_params = ca_params(not_before, not_after)?;
    let ca_cert = ca_params.self_signed(&ca_key)?;
    let issuer = Issuer::new(ca_params, ca_key);
    let leaf_key = KeyPair::generate()?;
    let leaf = leaf_params(not_before, not_after)?.signed_by(&leaf_key, &issuer)?;
    Ok(Generated {
        ca_pem: ca_cert.pem(),
        cert_pem: leaf.pem(),
        key_pem: leaf_key.serialize_pem(),
    })
}

/// A CA that may only sign one level of certificates, and only for loopback names.
fn ca_params(
    not_before: OffsetDateTime,
    not_after: OffsetDateTime,
) -> Result<CertificateParams, TlsError> {
    let mut params = CertificateParams::new(Vec::<String>::new())?;
    params.distinguished_name.push(
        DnType::CommonName,
        format!("Swakshar Local CA {}", not_before.date()),
    );
    params.distinguished_name.push(
        DnType::OrganizationName,
        "Swakshar, valid on this computer only",
    );
    params.is_ca = IsCa::Ca(BasicConstraints::Constrained(0));
    params.key_usages = vec![KeyUsagePurpose::KeyCertSign, KeyUsagePurpose::CrlSign];
    params.name_constraints = Some(NameConstraints {
        permitted_subtrees: vec![
            GeneralSubtree::IpAddress(CidrSubnet::from_v4_prefix(LOOPBACK_V4, 32)),
            GeneralSubtree::DnsName(LOCALHOST.to_owned()),
        ],
        excluded_subtrees: Vec::new(),
    });
    params.not_before = not_before;
    params.not_after = not_after;
    Ok(params)
}

/// The server certificate for `127.0.0.1` and `localhost`.
fn leaf_params(
    not_before: OffsetDateTime,
    not_after: OffsetDateTime,
) -> Result<CertificateParams, TlsError> {
    let mut params = CertificateParams::new(vec!["127.0.0.1".to_owned(), LOCALHOST.to_owned()])?;
    params
        .distinguished_name
        .push(DnType::CommonName, "127.0.0.1");
    params.key_usages = vec![KeyUsagePurpose::DigitalSignature];
    params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];
    params.use_authority_key_identifier_extension = true;
    params.not_before = not_before;
    params.not_after = not_after;
    Ok(params)
}
