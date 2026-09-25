//! Assembly of the `signtype=1` `SignedData`.

use std::time::Duration;

use cms::cert::{CertificateChoices, IssuerAndSerialNumber};
use cms::content_info::{CmsVersion, ContentInfo};
use cms::signed_data::{
    CertificateSet, EncapsulatedContentInfo, SignedAttributes, SignedData, SignerIdentifier,
    SignerInfo, SignerInfos,
};
use der::asn1::{OctetString, SetOfVec, UtcTime};
use der::{Any, Encode, Tag};
use sha1::{Digest as _, Sha1};
use spki::AlgorithmIdentifierOwned;
use x509_cert::Certificate;
use x509_cert::attr::{Attribute, AttributeValue};

use crate::error::{BuildError, CmsError};
use crate::oids::{
    ID_CONTENT_TYPE, ID_DATA, ID_MESSAGE_DIGEST, ID_SHA1, ID_SIGNED_DATA, ID_SIGNING_TIME,
    RSA_ENCRYPTION,
};

/// Everything a `signtype=1` signature covers, apart from the key itself.
#[derive(Debug, Clone, Copy)]
pub struct SignedDataInput<'a> {
    /// The request's `tobesigned` text as UTF-8 bytes, attached as `eContent`.
    pub content: &'a [u8],
    /// The signer certificate; the only certificate embedded.
    pub certificate: &'a Certificate,
    /// Signing time as a duration since the Unix epoch, written as UTCTime.
    pub signing_time: Duration,
}

/// Builds the DER `ContentInfo` for a `signtype=1` request.
///
/// `sign` receives the DER encoding of the signed attributes as a `SET OF`
/// and must return an RSASSA-PKCS1-v1_5 signature with SHA-1 over it, for
/// example from `CKM_SHA1_RSA_PKCS` on a token.
///
/// # Errors
///
/// Returns [`BuildError::Signer`] with the callback's error, or
/// [`BuildError::Cms`] when encoding fails.
pub fn build_signed_data<E, F>(
    input: &SignedDataInput<'_>,
    sign: F,
) -> Result<Vec<u8>, BuildError<E>>
where
    F: FnOnce(&[u8]) -> Result<Vec<u8>, E>,
{
    let signed_attrs = signed_attributes(input)?;
    let signature = sign(&signed_attrs.to_der()?).map_err(BuildError::Signer)?;
    let tbs = &input.certificate.tbs_certificate;
    let signer_info = SignerInfo {
        version: CmsVersion::V1,
        sid: SignerIdentifier::IssuerAndSerialNumber(IssuerAndSerialNumber {
            issuer: tbs.issuer.clone(),
            serial_number: tbs.serial_number.clone(),
        }),
        digest_alg: algorithm(ID_SHA1),
        signed_attrs: Some(signed_attrs),
        signature_algorithm: algorithm(RSA_ENCRYPTION),
        signature: OctetString::new(signature)?,
        unsigned_attrs: None,
    };
    let signed_data = SignedData {
        version: CmsVersion::V1,
        digest_algorithms: SetOfVec::try_from(vec![algorithm(ID_SHA1)])?,
        encap_content_info: EncapsulatedContentInfo {
            econtent_type: ID_DATA,
            econtent: Some(Any::new(Tag::OctetString, input.content.to_vec())?),
        },
        certificates: Some(CertificateSet(SetOfVec::try_from(vec![
            CertificateChoices::Certificate(input.certificate.clone()),
        ])?)),
        crls: None,
        signer_infos: SignerInfos(SetOfVec::try_from(vec![signer_info])?),
    };
    let content_info = ContentInfo {
        content_type: ID_SIGNED_DATA,
        content: Any::encode_from(&signed_data)?,
    };
    Ok(content_info.to_der()?)
}

/// The three signed attributes the reference signer emits, in DER order.
///
/// # Errors
///
/// Returns [`CmsError`] when the signing time is outside the UTCTime range or
/// encoding fails.
pub(crate) fn signed_attributes(input: &SignedDataInput<'_>) -> Result<SignedAttributes, CmsError> {
    let digest = Sha1::digest(input.content);
    let signing_time =
        UtcTime::from_unix_duration(input.signing_time).map_err(|_| CmsError::SigningTime)?;
    let attributes = vec![
        attribute(
            ID_CONTENT_TYPE,
            AttributeValue::new(Tag::ObjectIdentifier, ID_DATA.as_bytes())?,
        )?,
        attribute(ID_SIGNING_TIME, Any::encode_from(&signing_time)?)?,
        attribute(
            ID_MESSAGE_DIGEST,
            AttributeValue::new(Tag::OctetString, digest.as_slice())?,
        )?,
    ];
    Ok(SetOfVec::try_from(attributes)?)
}

/// One attribute holding a single value.
fn attribute(
    oid: const_oid::ObjectIdentifier,
    value: AttributeValue,
) -> Result<Attribute, CmsError> {
    Ok(Attribute {
        oid,
        values: SetOfVec::try_from(vec![value])?,
    })
}

/// An algorithm identifier with explicit NULL parameters, as BouncyCastle
/// writes them for SHA-1 and rsaEncryption.
fn algorithm(oid: const_oid::ObjectIdentifier) -> AlgorithmIdentifierOwned {
    AlgorithmIdentifierOwned {
        oid,
        parameters: Some(Any::null()),
    }
}
