//! Certificate fields the app shows and matches on, parsed from DER.

use const_oid::ObjectIdentifier;
use der::asn1::UintRef;
use der::{Any, Decode, Tag, Tagged as _};
use sha2::{Digest as _, Sha256};
use x509_cert::Certificate;
use x509_cert::certificate::TbsCertificate;
use x509_cert::ext::pkix::{CertificatePolicies, KeyUsage};
use x509_cert::name::Name;

use crate::error::TokenError;

/// Subject common name.
const OID_COMMON_NAME: ObjectIdentifier = const_oid::db::rfc4519::CN;
/// Subject serial number; CCA puts SHA-256 of the PAN here.
const OID_SERIAL_NUMBER: ObjectIdentifier = const_oid::db::rfc4519::SERIAL_NUMBER;
/// Key usage extension.
const OID_KEY_USAGE: ObjectIdentifier = const_oid::db::rfc5280::ID_CE_KEY_USAGE;
/// Certificate policies extension.
const OID_CERTIFICATE_POLICIES: ObjectIdentifier =
    const_oid::db::rfc5280::ID_CE_CERTIFICATE_POLICIES;
/// RSA public keys.
const OID_RSA_ENCRYPTION: ObjectIdentifier = const_oid::db::rfc5912::RSA_ENCRYPTION;
/// CCA India class policies (CCA-IOG): `2.16.356.100.2.<class>`.
const CCA_CLASS_POLICIES: [(u8, ObjectIdentifier); 2] = [
    (2, ObjectIdentifier::new_unwrap("2.16.356.100.2.2")),
    (3, ObjectIdentifier::new_unwrap("2.16.356.100.2.3")),
];
/// Length of a hex SHA-256.
const SHA256_HEX_LEN: usize = 64;

/// What the app needs to know about a certificate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CertSummary {
    /// Subject common name (the holder).
    pub subject_cn: String,
    /// Issuer common name (the CA).
    pub issuer_cn: String,
    /// Serial number in decimal.
    pub serial_decimal: String,
    /// `notBefore`, Unix seconds.
    pub not_before: i64,
    /// `notAfter`, Unix seconds.
    pub not_after: i64,
    /// Subject `serialNumber` when it is a SHA-256 hex (the PAN hash), lowercase.
    pub pan_hash: Option<String>,
    /// CCA classes the certificate is valid for, for example `[2, 3]`.
    pub classes: Vec<u8>,
    /// Key usage allows signing (digitalSignature or nonRepudiation).
    pub signing: bool,
    /// The public key is RSA.
    pub rsa: bool,
    /// SHA-256 of the DER, lowercase hex.
    pub fingerprint: String,
}

impl CertSummary {
    /// Highest class as a label: `Class 3`, `Class 2` or `Unclassified`.
    pub fn class_label(&self) -> &'static str {
        match self.classes.iter().max() {
            Some(3) => "Class 3",
            Some(2) => "Class 2",
            _ => "Unclassified",
        }
    }

    /// True when `now` (Unix seconds) is inside the validity period.
    pub fn valid_at(&self, now: i64) -> bool {
        self.not_before <= now && now <= self.not_after
    }
}

/// Parses a DER certificate into a [`CertSummary`].
///
/// # Errors
///
/// Returns [`TokenError::Certificate`] when the DER or an extension is malformed.
pub fn summarize(der: &[u8]) -> Result<CertSummary, TokenError> {
    let certificate = Certificate::from_der(der)?;
    let tbs = &certificate.tbs_certificate;
    let (signing, classes) = extensions(tbs)?;
    Ok(CertSummary {
        subject_cn: name_attribute(&tbs.subject, OID_COMMON_NAME).unwrap_or_default(),
        issuer_cn: name_attribute(&tbs.issuer, OID_COMMON_NAME).unwrap_or_default(),
        serial_decimal: to_decimal(tbs.serial_number.as_bytes()),
        not_before: unix_seconds(&tbs.validity.not_before),
        not_after: unix_seconds(&tbs.validity.not_after),
        pan_hash: name_attribute(&tbs.subject, OID_SERIAL_NUMBER)
            .map(|value| value.trim().to_ascii_lowercase())
            .filter(|value| {
                value.len() == SHA256_HEX_LEN && value.chars().all(|ch| ch.is_ascii_hexdigit())
            }),
        classes,
        signing,
        rsa: tbs.subject_public_key_info.algorithm.oid == OID_RSA_ENCRYPTION,
        fingerprint: hex(&Sha256::digest(der)),
    })
}

/// The RSA modulus of a certificate's key, without leading zeros.
pub(crate) fn rsa_modulus(certificate: &Certificate) -> Option<Vec<u8>> {
    let bits = certificate
        .tbs_certificate
        .subject_public_key_info
        .subject_public_key
        .raw_bytes();
    let key = RsaPublicKey::from_der(bits).ok()?;
    Some(key.modulus.as_bytes().to_vec())
}

/// PKCS#1 `RSAPublicKey`.
#[derive(der::Sequence)]
struct RsaPublicKey<'a> {
    /// Modulus `n`.
    modulus: UintRef<'a>,
    /// Public exponent `e`.
    public_exponent: UintRef<'a>,
}

/// Signing capability and CCA classes from the extensions. A certificate
/// without a key usage extension is not restricted.
fn extensions(tbs: &TbsCertificate) -> Result<(bool, Vec<u8>), TokenError> {
    let mut signing = true;
    let mut classes = Vec::new();
    for extension in tbs.extensions.iter().flatten() {
        let value = extension.extn_value.as_bytes();
        if extension.extn_id == OID_KEY_USAGE {
            let usage = KeyUsage::from_der(value)?;
            signing = usage.digital_signature() || usage.non_repudiation();
        } else if extension.extn_id == OID_CERTIFICATE_POLICIES {
            let policies = CertificatePolicies::from_der(value)?;
            classes = CCA_CLASS_POLICIES
                .iter()
                .filter(|(_, oid)| {
                    policies
                        .0
                        .iter()
                        .any(|policy| policy.policy_identifier == *oid)
                })
                .map(|(class, _)| *class)
                .collect();
        }
    }
    Ok((signing, classes))
}

/// First value of `oid` in a distinguished name, as text.
fn name_attribute(name: &Name, oid: ObjectIdentifier) -> Option<String> {
    name.0
        .iter()
        .flat_map(|rdn| rdn.0.iter())
        .find(|attribute| attribute.oid == oid)
        .and_then(|attribute| any_to_string(&attribute.value))
}

/// Decodes the directory string types certificates use.
fn any_to_string(value: &Any) -> Option<String> {
    let bytes = value.value();
    match value.tag() {
        Tag::Utf8String
        | Tag::PrintableString
        | Tag::Ia5String
        | Tag::VisibleString
        | Tag::TeletexString => Some(String::from_utf8_lossy(bytes).into_owned()),
        Tag::BmpString => {
            let (pairs, _) = bytes.as_chunks::<2>();
            let units: Vec<u16> = pairs.iter().map(|pair| u16::from_be_bytes(*pair)).collect();
            String::from_utf16(&units).ok()
        }
        _ => None,
    }
}

/// Converts a big-endian unsigned integer into decimal text.
fn to_decimal(bytes: &[u8]) -> String {
    let mut number: Vec<u8> = bytes
        .iter()
        .copied()
        .skip_while(|byte| *byte == 0)
        .collect();
    if number.is_empty() {
        return "0".to_owned();
    }
    let mut digits = Vec::new();
    while !number.is_empty() {
        let mut remainder = 0_u32;
        let mut quotient = Vec::with_capacity(number.len());
        for byte in &number {
            let accumulator = remainder * 256 + u32::from(*byte);
            let digit = accumulator / 10;
            remainder = accumulator % 10;
            if !(quotient.is_empty() && digit == 0) {
                quotient.push(u8::try_from(digit).unwrap_or(u8::MAX));
            }
        }
        digits.push(char::from(b'0' + u8::try_from(remainder).unwrap_or(0)));
        number = quotient;
    }
    digits.iter().rev().collect()
}

/// Unix seconds of an X.509 time, saturating.
fn unix_seconds(time: &x509_cert::time::Time) -> i64 {
    i64::try_from(time.to_unix_duration().as_secs()).unwrap_or(i64::MAX)
}

/// Lowercase hex.
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
#[path = "certinfo_tests.rs"]
mod tests;
