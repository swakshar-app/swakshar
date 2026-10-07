//! `doctor` and `setup`: what is installed, what is trusted, what is missing.

use swakshar_protocol::format_date_utc;
use swakshar_tls::{
    TlsError, TrustStatus, data_dir, ensure_identity, install_command, install_trust,
    load_identity, tls_dir, trust_status,
};
use swakshar_token::{ArchSupport, Inventory, TokenService};

use crate::args::Options;
use crate::error::CliError;
use crate::{mask_serial, unix_now};

/// Prints drivers, tokens, certificates and the local certificate's state.
pub(crate) fn doctor(options: &Options) -> Result<(), CliError> {
    let data_dir = data_dir().ok_or(TlsError::NoDataDir)?;
    println!("Data directory: {}", data_dir.display());
    match load_identity(&tls_dir(&data_dir)) {
        Ok(identity) => {
            println!(
                "Local certificate: valid until {}",
                format_date_utc(identity.not_after)
            );
            println!("Trusted by this Mac: {}", describe(trust_status(&identity)));
        }
        Err(error) => {
            println!("Local certificate: not created yet ({error}). Run `swakshar setup`.")
        }
    }
    let inventory = TokenService::spawn()?.inventory_blocking(options.modules.clone())?;
    print_modules(&inventory);
    print_tokens(&inventory, unix_now());
    Ok(())
}

/// Creates the local certificate, and trusts it when `trust` is set.
pub(crate) fn setup(trust: bool) -> Result<(), CliError> {
    let data_dir = data_dir().ok_or(TlsError::NoDataDir)?;
    let (identity, minted) = ensure_identity(&tls_dir(&data_dir), unix_now())?;
    let state = if minted { "Created" } else { "Found" };
    println!(
        "{state} the local certificate, valid until {}.",
        format_date_utc(identity.not_after)
    );
    println!("It only works for 127.0.0.1 and localhost; its CA key was never saved.");
    match trust_status(&identity) {
        TrustStatus::Trusted => println!("This Mac already trusts it."),
        TrustStatus::Unsupported => println!(
            "Trust {} manually on this system.",
            identity.ca_path().display()
        ),
        TrustStatus::NotTrusted if trust => {
            println!("macOS will ask for your password to trust it.");
            install_trust(&identity)?;
            println!("Trusted. Quit and reopen your browser.");
        }
        TrustStatus::NotTrusted => {
            println!("Not trusted yet. Run `swakshar setup --trust`, or run:");
            println!("  {}", install_command(&identity));
        }
    }
    Ok(())
}

/// Human text for a trust state.
fn describe(status: TrustStatus) -> &'static str {
    match status {
        TrustStatus::Trusted => "yes",
        TrustStatus::NotTrusted => "no (run `swakshar setup --trust`)",
        TrustStatus::Unsupported => "not checked on this system",
    }
}

/// One line per driver that exists or was added.
fn print_modules(inventory: &Inventory) {
    println!();
    println!("Token drivers:");
    let mut any = false;
    for status in inventory
        .modules
        .iter()
        .filter(|status| status.candidate.exists || status.candidate.user_added)
    {
        any = true;
        let state = match (
            &status.error,
            status.candidate.exists,
            status.candidate.arch,
        ) {
            (_, false, _) => "missing".to_owned(),
            (Some(error), _, _) => format!("failed: {error}"),
            (None, true, ArchSupport::Incompatible) => "built for another processor".to_owned(),
            (None, true, _) => "loaded".to_owned(),
        };
        println!(
            "  {} ({}): {state}",
            status.candidate.family,
            status.candidate.path.display()
        );
    }
    if !any {
        println!("  none found. Install your token's driver, or pass --module <path>.");
    }
}

/// Tokens and their certificates.
fn print_tokens(inventory: &Inventory, now: i64) {
    println!();
    println!("Tokens:");
    if inventory.tokens.is_empty() {
        println!("  none connected.");
        print_detected(inventory);
    }
    for token in &inventory.tokens {
        let pin = if token.pin.locked {
            ", PIN LOCKED"
        } else if token.pin.final_try {
            ", one PIN try left"
        } else {
            ""
        };
        println!(
            "  {} {} (serial {}){pin}",
            token.manufacturer,
            token.model,
            mask_serial(&token.serial)
        );
        for certificate in &token.certificates {
            let summary = &certificate.summary;
            let validity = if summary.valid_at(now) {
                "valid until"
            } else {
                "expired or not yet valid, until"
            };
            let purpose = if summary.signing {
                "signing"
            } else {
                "not for signing"
            };
            println!(
                "    {}: {}, {purpose}, {validity} {}, issued by {}",
                summary.subject_cn,
                summary.class_label(),
                format_date_utc(summary.not_after),
                summary.issuer_cn
            );
        }
    }
}

/// Tokens seen on the USB bus whose driver is missing or did not load.
fn print_detected(inventory: &Inventory) {
    for token in &inventory.detected {
        let advice = if token.driver_present {
            "its driver is installed but did not load, see Token drivers above".to_owned()
        } else if let Some(url) = token.driver_url {
            format!(
                "install the {} macOS driver from {url}, then run doctor again",
                token.family
            )
        } else {
            format!(
                "install the {} macOS driver from your token vendor or the Certifying \
                 Authority that issued your DSC, then run doctor again",
                token.family
            )
        };
        println!(
            "  {} seen on USB ({}) but no driver read it: {advice}.",
            token.family, token.name
        );
    }
}
