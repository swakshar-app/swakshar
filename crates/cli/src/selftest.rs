//! `selftest`: sign a known text directly on the token and verify it.

use swakshar_cms::{inspect_signed_data, to_portal_base64};
use swakshar_token::{Criteria, TokenService, candidates};

use crate::args::{Options, SelftestArgs};
use crate::error::CliError;
use crate::prompt::choose;
use crate::signing::{Signed, sign_interactively};
use crate::unix_now;

/// Signs `args.content`, verifies the CMS, prints it, optionally saves it.
pub(crate) fn selftest(options: &Options, args: &SelftestArgs) -> Result<(), CliError> {
    let service = TokenService::spawn()?;
    let inventory = service.inventory_blocking(options.modules.clone())?;
    let signing_time = args
        .signing_time
        .map_or_else(unix_now, |time| i64::try_from(time).unwrap_or(i64::MAX));
    let criteria = Criteria {
        pan: None,
        expiry_check: false,
        classes: &[],
        issuer_name: None,
        now: signing_time,
    };
    let mut found = candidates(&inventory.tokens, &criteria);
    if let Some(serial) = &args.token {
        found.retain(|candidate| {
            inventory
                .tokens
                .get(candidate.token)
                .is_some_and(|token| &token.serial == serial)
        });
    }
    let candidate = choose(&inventory, &found)
        .ok_or_else(|| CliError::Message("no certificate chosen".to_owned()))?;
    let Signed::Done(output) = sign_interactively(
        &service,
        &inventory,
        &candidate,
        args.content.as_bytes(),
        signing_time,
    )?
    else {
        return Err(CliError::Message("canceled".to_owned()));
    };
    let inspection = inspect_signed_data(&output.cms_der)?;
    println!(
        "Signed {} bytes as {} using {:?}; the signature verifies with the token's certificate.",
        inspection.content.len(),
        output.summary.subject_cn,
        output.mechanism
    );
    println!("{}", to_portal_base64(&output.cms_der));
    if let Some(path) = &args.out {
        std::fs::write(path, &output.cms_der)?;
        println!("DER written to {}", path.display());
    }
    Ok(())
}
