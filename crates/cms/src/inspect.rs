//! Parsing and checking a CMS the way the portal will, before it leaves the app.

use cms::content_info::ContentInfo;
use cms::signed_data::{SignedData, SignerInfo};
use der::{Decode, Encode};
use ring::signature::{RSA_PKCS1_1024_8192_SHA1_FOR_LEGACY_USE_ONLY, UnparsedPublicKey};
use sha1::{Digest as _, Sha1};
use x509_cert::Certificate;

use crate::error::CmsError;
use crate::oids::{
    ID_CONTENT_TYPE, ID_DATA, ID_MESSAGE_DIGEST, ID_SHA1, ID_SIGNED_DATA, ID_SIGNING_TIME,
};

/// The parts of a portal CMS, after its layout has been checked.
#[derive(Debug, Clone)]
pub struct Inspection {
    /// The attached content (the request's `tobesigned` bytes).
    pub content: Vec<u8>,
    /// The single embedded certificate.
    pub certificate: Certificate,
    /// DER of the signed attributes as a `SET OF`, the bytes the key signed.
    pub signed_attributes_der: Vec<u8>,
    /// The raw RSA signature.
    pub signature: Vec<u8>,
}

/// Parses a DER CMS and checks that it has the portal's layout: signed data
/// version 1, SHA-1 only, attached `id-data` content, one certificate, one
/// signer, and exactly the three expected signed attributes.
///
/// # Errors
///
/// Returns [`CmsError::Layout`] naming the first difference, or a DER error.
pub fn inspect_layout(der: &[u8]) -> Result<Inspection, CmsError> {
    let content_info = ContentInfo::from_der(der)?;
    if content_info.content_type != ID_SIGNED_DATA {
        return Err(CmsError::Layout("outer content type is not signed-data"));
    }
    let signed_data = SignedData::from_der(&content_info.content.to_der()?)?;
    let digest_ok = signed_data.digest_algorithms.len() == 1
        && signed_data
            .digest_algorithms
            .iter()
            .all(|alg| alg.oid == ID_SHA1);
    if !digest_ok {
        return Err(CmsError::Layout("digest algorithms must be exactly SHA-1"));
    }
    let content = attached_content(&signed_data)?;
    let certificate = single_certificate(&signed_data)?;
    let signer = single_signer(&signed_data)?;
    let attributes = signer
        .signed_attrs
        .as_ref()
        .ok_or(CmsError::Layout("signed attributes are missing"))?;
    let oids: Vec<_> = attributes.iter().map(|attribute| attribute.oid).collect();
    if oids != [ID_CONTENT_TYPE, ID_SIGNING_TIME, ID_MESSAGE_DIGEST] {
        return Err(CmsError::Layout(
            "signed attributes must be contentType, signingTime, messageDigest",
        ));
    }
    Ok(Inspection {
        content,
        certificate,
        signed_attributes_der: attributes.to_der()?,
        signature: signer.signature.as_bytes().to_vec(),
    })
}

/// Full check: layout, `messageDigest` against the content, and the RSA
/// signature against the embedded certificate.
///
/// # Errors
///
/// Returns the first failed check as a [`CmsError`].
pub fn inspect_signed_data(der: &[u8]) -> Result<Inspection, CmsError> {
    let inspection = inspect_layout(der)?;
    if !message_digest_matches(&inspection)? {
        return Err(CmsError::DigestMismatch);
    }
    if !verify_rsa_sha1(
        &inspection.certificate,
        &inspection.signed_attributes_der,
        &inspection.signature,
    ) {
        return Err(CmsError::BadSignature);
    }
    Ok(inspection)
}

/// Verifies an RSASSA-PKCS1-v1_5 SHA-1 signature with the certificate's key.
/// Returns false for non-RSA keys and for any verification failure.
pub fn verify_rsa_sha1(certificate: &Certificate, message: &[u8], signature: &[u8]) -> bool {
    let key = certificate
        .tbs_certificate
        .subject_public_key_info
        .subject_public_key
        .raw_bytes();
    UnparsedPublicKey::new(&RSA_PKCS1_1024_8192_SHA1_FOR_LEGACY_USE_ONLY, key)
        .verify(message, signature)
        .is_ok()
}

/// Checks that `messageDigest` holds the SHA-1 of the attached content.
fn message_digest_matches(inspection: &Inspection) -> Result<bool, CmsError> {
    let attributes =
        cms::signed_data::SignedAttributes::from_der(&inspection.signed_attributes_der)?;
    let expected = Sha1::digest(&inspection.content);
    Ok(attributes
        .iter()
        .filter(|attribute| attribute.oid == ID_MESSAGE_DIGEST)
        .flat_map(|attribute| attribute.values.iter())
        .any(|value| value.value() == expected.as_slice()))
}

/// Returns the attached `id-data` content.
fn attached_content(signed_data: &SignedData) -> Result<Vec<u8>, CmsError> {
    let encap = &signed_data.encap_content_info;
    if encap.econtent_type != ID_DATA {
        return Err(CmsError::Layout("encapsulated content type is not id-data"));
    }
    let econtent = encap
        .econtent
        .as_ref()
        .ok_or(CmsError::Layout("content is detached"))?;
    Ok(econtent.value().to_vec())
}

/// Returns the one embedded certificate.
fn single_certificate(signed_data: &SignedData) -> Result<Certificate, CmsError> {
    let set = signed_data
        .certificates
        .as_ref()
        .ok_or(CmsError::Layout("no certificate embedded"))?;
    let mut certificates = set.0.iter();
    match (certificates.next(), certificates.next()) {
        (Some(cms::cert::CertificateChoices::Certificate(certificate)), None) => {
            Ok(certificate.clone())
        }
        _ => Err(CmsError::Layout(
            "exactly one X.509 certificate must be embedded",
        )),
    }
}

/// Returns the one signer.
fn single_signer(signed_data: &SignedData) -> Result<&SignerInfo, CmsError> {
    let mut signers = signed_data.signer_infos.0.iter();
    match (signers.next(), signers.next()) {
        (Some(signer), None) if signer.digest_alg.oid == ID_SHA1 => Ok(signer),
        (Some(_), None) => Err(CmsError::Layout("signer digest algorithm is not SHA-1")),
        _ => Err(CmsError::Layout("exactly one signer is required")),
    }
}
