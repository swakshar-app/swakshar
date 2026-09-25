//! Where token drivers usually live, per operating system.
//!
//! Paths are facts gathered by installing each vendor's driver. Users can
//! add any other module path in settings.

use std::path::{Path, PathBuf};

use crate::macho::{ArchSupport, arch_support};

/// Driver locations on macOS, as (family, path).
const MACOS_MODULES: &[(&str, &str)] = &[
    (
        "ePass2003 / HYP2003",
        "/usr/local/lib/libcastle_v2.1.0.0.dylib",
    ),
    ("ePass2003", "/usr/local/lib/libcastle.1.0.0.dylib"),
    ("SafeNet eToken", "/usr/local/lib/libeTPkcs11.dylib"),
    (
        "Watchdata ProxKey",
        "/usr/local/lib/wdProxKeyUsbKeyTool/libwdpkcs_Proxkey.dylib",
    ),
    (
        "Watchdata",
        "/usr/lib/WatchData/eMudhra_3.4.3/lib/libwdpkcs_eMudhra_343.dylib",
    ),
    ("TrustKey", "/usr/local/lib/TrustKeyP11_ND_v3.4.dylib"),
    ("mToken CryptoID", "/usr/local/lib/libcryptoid_pkcs11.dylib"),
    ("OpenSC", "/Library/OpenSC/lib/opensc-pkcs11.so"),
    ("OpenSC (Homebrew)", "/opt/homebrew/lib/opensc-pkcs11.so"),
];

/// Driver locations on Linux, as (family, path).
const LINUX_MODULES: &[(&str, &str)] = &[
    ("ePass2003", "/usr/lib/libcastle.so.1.0.0"),
    ("SafeNet eToken", "/usr/lib/libeTPkcs11.so"),
    ("OpenSC", "/usr/lib/x86_64-linux-gnu/opensc-pkcs11.so"),
    ("OpenSC", "/usr/lib64/opensc-pkcs11.so"),
];

/// Driver locations on Windows, as (family, path).
const WINDOWS_MODULES: &[(&str, &str)] = &[
    ("ePass2003", r"C:\Windows\System32\eps2003csp11.dll"),
    ("SafeNet eToken", r"C:\Windows\System32\eTPKCS11.dll"),
    (
        "OpenSC",
        r"C:\Program Files\OpenSC Project\OpenSC\pkcs11\opensc-pkcs11.dll",
    ),
];

/// A driver module that may expose tokens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleCandidate {
    /// Absolute path of the PKCS#11 module.
    pub path: PathBuf,
    /// Token family, for display.
    pub family: String,
    /// Whether the file exists.
    pub exists: bool,
    /// Whether this process can load it.
    pub arch: ArchSupport,
    /// Added by the user rather than discovered.
    pub user_added: bool,
}

/// Known driver paths for this OS followed by user-added paths, deduplicated.
pub fn candidate_modules(extra: &[PathBuf]) -> Vec<ModuleCandidate> {
    let known = known_modules()
        .iter()
        .map(|(family, path)| candidate(Path::new(path), family, false));
    let added = extra
        .iter()
        .map(|path| candidate(path, "Added by you", true));
    let mut seen = Vec::<PathBuf>::new();
    known
        .chain(added)
        .filter(|module| {
            let fresh = !seen.contains(&module.path);
            seen.push(module.path.clone());
            fresh
        })
        .collect()
}

/// The known list for the current OS.
fn known_modules() -> &'static [(&'static str, &'static str)] {
    if cfg!(target_os = "macos") {
        MACOS_MODULES
    } else if cfg!(target_os = "windows") {
        WINDOWS_MODULES
    } else {
        LINUX_MODULES
    }
}

/// Probes one path.
fn candidate(path: &Path, family: &str, user_added: bool) -> ModuleCandidate {
    let exists = path.is_file();
    ModuleCandidate {
        path: path.to_path_buf(),
        family: family.to_owned(),
        exists,
        arch: if exists && cfg!(target_os = "macos") {
            arch_support(path)
        } else {
            ArchSupport::Unknown
        },
        user_added,
    }
}
