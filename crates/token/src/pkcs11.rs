//! Low-level PKCS#11 calls. Only the token thread calls these.

use std::path::Path;

use cryptoki::context::{CInitializeArgs, CInitializeFlags, Pkcs11};
use cryptoki::error::{Error as CryptokiError, RvError};
use cryptoki::mechanism::{Mechanism, MechanismType};
use cryptoki::object::{Attribute, AttributeType, CertificateType, ObjectClass, ObjectHandle};
use cryptoki::session::{Session, UserType};
use cryptoki::slot::{Slot, TokenInfo};
use cryptoki::types::AuthPin;
use swakshar_cms::{sha1_digest_info, verify_rsa_sha1};
use x509_cert::Certificate;

use crate::certinfo::{rsa_modulus, summarize};
use crate::error::TokenError;
use crate::types::{PinState, SignMechanism, TokenCertificate, TokenEntry};

/// Loads and initialises a module.
pub(crate) fn load(path: &Path) -> Result<Pkcs11, TokenError> {
    let pkcs11 = Pkcs11::new(path).map_err(|error| TokenError::ModuleLoad(error.to_string()))?;
    match pkcs11.initialize(CInitializeArgs::new(CInitializeFlags::OS_LOCKING_OK)) {
        Ok(()) | Err(CryptokiError::Pkcs11(RvError::CryptokiAlreadyInitialized, _)) => Ok(pkcs11),
        Err(error) => Err(TokenError::ModuleLoad(error.to_string())),
    }
}

/// Every token the module exposes, with its certificates. A token that fails
/// to read is logged and skipped so one bad slot cannot hide the others.
pub(crate) fn read_tokens(pkcs11: &Pkcs11, module: &Path) -> Result<Vec<TokenEntry>, TokenError> {
    let tokens = pkcs11
        .get_slots_with_token()?
        .into_iter()
        .filter_map(|slot| match read_token(pkcs11, slot, module) {
            Ok(token) => Some(token),
            Err(error) => {
                log::warn!("skipping a slot in {}: {error}", module.display());
                None
            }
        })
        .collect();
    Ok(tokens)
}

/// Reads one token's details and certificates, without logging in.
fn read_token(pkcs11: &Pkcs11, slot: Slot, module: &Path) -> Result<TokenEntry, TokenError> {
    let info = pkcs11.get_token_info(slot)?;
    let session = pkcs11.open_ro_session(slot)?;
    Ok(TokenEntry {
        module: module.to_path_buf(),
        label: info.label().trim().to_owned(),
        serial: info.serial_number().trim().to_owned(),
        manufacturer: info.manufacturer_id().trim().to_owned(),
        model: info.model().trim().to_owned(),
        pin: pin_state(&info),
        certificates: read_certificates(&session)?,
    })
}

/// PIN flags from the token info.
pub(crate) fn pin_state(info: &TokenInfo) -> PinState {
    PinState {
        count_low: info.user_pin_count_low(),
        final_try: info.user_pin_final_try(),
        locked: info.user_pin_locked(),
        protected_path: info.protected_authentication_path(),
    }
}

/// Every X.509 certificate object on the token that parses.
fn read_certificates(session: &Session) -> Result<Vec<TokenCertificate>, TokenError> {
    let template = [
        Attribute::Class(ObjectClass::CERTIFICATE),
        Attribute::CertificateType(CertificateType::X_509),
    ];
    let mut certificates = Vec::new();
    for handle in session.find_objects(&template)? {
        let wanted = [
            AttributeType::Value,
            AttributeType::Id,
            AttributeType::Label,
        ];
        let (mut der, mut id, mut label) = (None, Vec::new(), String::new());
        for attribute in session.get_attributes(handle, &wanted)? {
            match attribute {
                Attribute::Value(value) => der = Some(value),
                Attribute::Id(value) => id = value,
                Attribute::Label(value) => {
                    label = String::from_utf8_lossy(&value).trim().to_owned()
                }
                _ => {}
            }
        }
        let Some(der) = der else { continue };
        match summarize(&der) {
            Ok(summary) => certificates.push(TokenCertificate {
                id,
                label,
                der,
                summary,
            }),
            Err(error) => log::warn!("skipping an unreadable certificate: {error}"),
        }
    }
    Ok(certificates)
}

/// The slot holding the token with `serial`.
pub(crate) fn find_slot(pkcs11: &Pkcs11, serial: &str) -> Result<Slot, TokenError> {
    for slot in pkcs11.get_slots_with_token()? {
        if pkcs11.get_token_info(slot)?.serial_number().trim() == serial {
            return Ok(slot);
        }
    }
    Err(TokenError::TokenNotFound)
}

/// DER of the certificate object with `id`.
pub(crate) fn find_certificate(session: &Session, id: &[u8]) -> Result<Vec<u8>, TokenError> {
    let template = [
        Attribute::Class(ObjectClass::CERTIFICATE),
        Attribute::Id(id.to_vec()),
    ];
    for handle in session.find_objects(&template)? {
        for attribute in session.get_attributes(handle, &[AttributeType::Value])? {
            if let Attribute::Value(der) = attribute {
                return Ok(der);
            }
        }
    }
    Err(TokenError::CertificateNotFound)
}

/// Logs the user in. Tokens with a PIN pad get no PIN; a wrong PIN reports
/// the token's PIN flags afterwards so the UI can warn before a lockout.
pub(crate) fn login(
    pkcs11: &Pkcs11,
    slot: Slot,
    session: &Session,
    pin: Option<&AuthPin>,
) -> Result<(), TokenError> {
    let info = pkcs11.get_token_info(slot)?;
    if info.user_pin_locked() {
        return Err(TokenError::PinLocked);
    }
    let pin = if info.protected_authentication_path() {
        None
    } else {
        Some(pin.ok_or(TokenError::PinRequired)?)
    };
    match session.login(UserType::User, pin) {
        Ok(()) | Err(CryptokiError::Pkcs11(RvError::UserAlreadyLoggedIn, _)) => Ok(()),
        Err(error) => match TokenError::from(error) {
            TokenError::PinIncorrect(_) => Err(TokenError::PinIncorrect(
                pkcs11
                    .get_token_info(slot)
                    .map(|info| pin_state(&info))
                    .unwrap_or_default(),
            )),
            other => Err(other),
        },
    }
}

/// The private key for a certificate: by shared `CKA_ID` first, then by
/// matching RSA modulus (some tokens label keys differently from certs).
pub(crate) fn find_private_key(
    session: &Session,
    cert_id: &[u8],
    certificate: &Certificate,
) -> Result<ObjectHandle, TokenError> {
    if !cert_id.is_empty() {
        let template = [
            Attribute::Class(ObjectClass::PRIVATE_KEY),
            Attribute::Id(cert_id.to_vec()),
        ];
        if let Some(key) = session.find_objects(&template)?.first() {
            return Ok(*key);
        }
    }
    let modulus = rsa_modulus(certificate).ok_or(TokenError::KeyNotFound)?;
    for key in session.find_objects(&[Attribute::Class(ObjectClass::PRIVATE_KEY)])? {
        let matches = session
            .get_attributes(key, &[AttributeType::Modulus])?
            .iter()
            .any(|attribute| matches!(attribute, Attribute::Modulus(value) if strip_zeros(value) == strip_zeros(&modulus)));
        if matches {
            return Ok(key);
        }
    }
    Err(TokenError::KeyNotFound)
}

/// Signs the DER signed attributes. Prefers `CKM_SHA1_RSA_PKCS`, the path
/// the portal already accepted; falls back to `CKM_RSA_PKCS` over a proper
/// `DigestInfo`. Every signature is verified before it is used.
pub(crate) fn sign_verified(
    session: &Session,
    key: ObjectHandle,
    mechanisms: &[MechanismType],
    certificate: &Certificate,
    data: &[u8],
) -> Result<(Vec<u8>, SignMechanism), TokenError> {
    if mechanisms.contains(&MechanismType::SHA1_RSA_PKCS) {
        let signature = session.sign(&Mechanism::Sha1RsaPkcs, key, data)?;
        if verify_rsa_sha1(certificate, data, &signature) {
            return Ok((signature, SignMechanism::Sha1RsaPkcs));
        }
        log::warn!("CKM_SHA1_RSA_PKCS output did not verify; retrying with CKM_RSA_PKCS");
    }
    let signature = session.sign(&Mechanism::RsaPkcs, key, &sha1_digest_info(data))?;
    if verify_rsa_sha1(certificate, data, &signature) {
        Ok((signature, SignMechanism::RsaPkcsDigestInfo))
    } else {
        Err(TokenError::SignatureMismatch)
    }
}

/// Drops leading zero bytes of a big-endian integer.
fn strip_zeros(bytes: &[u8]) -> &[u8] {
    let start = bytes
        .iter()
        .position(|byte| *byte != 0)
        .unwrap_or(bytes.len());
    bytes.get(start..).unwrap_or_default()
}
