//! Object identifiers used by the portal's CMS layout.

use const_oid::ObjectIdentifier;

/// `id-data`, the content type of the signed text.
pub const ID_DATA: ObjectIdentifier = const_oid::db::rfc5911::ID_DATA;

/// `id-signedData`, the outer content type.
pub const ID_SIGNED_DATA: ObjectIdentifier = const_oid::db::rfc5911::ID_SIGNED_DATA;

/// `contentType` signed attribute.
pub const ID_CONTENT_TYPE: ObjectIdentifier = const_oid::db::rfc5911::ID_CONTENT_TYPE;

/// `messageDigest` signed attribute.
pub const ID_MESSAGE_DIGEST: ObjectIdentifier = const_oid::db::rfc5911::ID_MESSAGE_DIGEST;

/// `signingTime` signed attribute.
pub const ID_SIGNING_TIME: ObjectIdentifier = const_oid::db::rfc5911::ID_SIGNING_TIME;

/// SHA-1 digest algorithm.
pub const ID_SHA1: ObjectIdentifier = const_oid::db::rfc5912::ID_SHA_1;

/// `rsaEncryption`, the signature algorithm the reference signer declares.
pub const RSA_ENCRYPTION: ObjectIdentifier = const_oid::db::rfc5912::RSA_ENCRYPTION;
