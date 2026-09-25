//! Terminal prompts: choosing a certificate, confirming, entering the PIN.

use std::io::{BufRead as _, Write as _};

use swakshar_protocol::format_date_utc;
use swakshar_token::{AuthPin, Candidate, Inventory, PanMatch};

use crate::mask_serial;

/// Lets the user pick a certificate. The only PAN match is taken without
/// asking; otherwise the user chooses by number. `None` means cancel.
pub(crate) fn choose(inventory: &Inventory, found: &[Candidate]) -> Option<Candidate> {
    if found.is_empty() {
        println!("No suitable certificate is on the connected tokens.");
        return None;
    }
    for (index, candidate) in found.iter().enumerate() {
        println!("  [{}] {}", index + 1, describe(inventory, candidate));
    }
    let matches = found
        .iter()
        .filter(|candidate| candidate.pan_match == PanMatch::Match)
        .count();
    if found.len() == 1 || matches == 1 {
        let first = found.first().copied()?;
        println!("Using [1].");
        return Some(first);
    }
    let answer = ask("Which certificate? Enter its number, or press Enter to cancel: ")?;
    let index: usize = answer.parse().ok()?;
    found.get(index.checked_sub(1)?).copied()
}

/// Asks a yes/no question; anything but `y` or `yes` means no.
pub(crate) fn confirm(question: &str) -> bool {
    ask(&format!("{question} [y/N] "))
        .is_some_and(|answer| matches!(answer.to_ascii_lowercase().as_str(), "y" | "yes"))
}

/// Reads the PIN without echo. Empty input means cancel.
pub(crate) fn pin() -> Option<AuthPin> {
    rpassword::prompt_password("Token PIN: ")
        .ok()
        .filter(|pin| !pin.is_empty())
        .map(AuthPin::from)
}

/// One line describing a candidate.
fn describe(inventory: &Inventory, candidate: &Candidate) -> String {
    let token = inventory.tokens.get(candidate.token);
    let summary = token
        .and_then(|token| token.certificates.get(candidate.certificate))
        .map(|certificate| &certificate.summary);
    let (Some(token), Some(summary)) = (token, summary) else {
        return "certificate no longer available".to_owned();
    };
    let pan = match candidate.pan_match {
        PanMatch::Match => ", matches the PAN",
        PanMatch::Mismatch => ", belongs to a DIFFERENT PAN (GST will reject it)",
        PanMatch::Unknown => "",
    };
    format!(
        "{} ({}, valid until {}) on token {}{pan}",
        summary.subject_cn,
        summary.class_label(),
        format_date_utc(summary.not_after),
        mask_serial(&token.serial)
    )
}

/// Prints a prompt and reads one trimmed line; `None` on empty or EOF.
fn ask(prompt: &str) -> Option<String> {
    print!("{prompt}");
    std::io::stdout().flush().ok()?;
    let mut line = String::new();
    std::io::stdin().lock().read_line(&mut line).ok()?;
    let answer = line.trim().to_owned();
    (!answer.is_empty()).then_some(answer)
}
