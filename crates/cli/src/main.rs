//! `swakshar`: the headless signer and its diagnostics.
//!
//! The desktop app is the product; this binary exists for power users, for
//! checking a machine end to end, and for comparing signatures with other
//! implementations byte for byte (`selftest --signing-time`).

mod args;
mod doctor;
mod error;
mod logger;
mod probe;
mod prompt;
mod selftest;
mod serve;
mod signing;

use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

use args::{Command, parse};
use error::CliError;

/// Help text.
const USAGE: &str = "\
Usage: swakshar <command> [options]

Commands:
  doctor      Show drivers, tokens, certificates and the local certificate
  setup       Create the local certificate (add --trust to trust it now)
  serve       Run the portal signer here, approving requests in this terminal
  selftest    Sign a test text directly on the token and verify the result
  probe       Act like the GST portal against a running signer
  version     Print the version

Options:
  --module <path>            Extra PKCS#11 driver (repeatable)
  --port <port>              Preferred signer port (1585, 2095, 2568, 2868, 4587)
  --greeting-version <v>     Greeting version the portal expects (default 2.8)
  --allow-origin <origin>    Extra page origin allowed to connect (repeatable)
  --trust                    setup: add the local CA to the login keychain
  --content <text>           selftest, probe: text to sign
  --pan <PAN>                probe: send this PAN like the portal does
  --signing-time <unix>      selftest: fixed signing time for byte comparisons
  --token <serial>           selftest: use the token with this serial
  --out <file>               selftest: write the DER signature to a file
";

/// Parses arguments, runs one command, and maps errors to an exit code.
fn main() -> ExitCode {
    logger::init();
    let (command, options) = match parse(std::env::args().skip(1)) {
        Ok(parsed) => parsed,
        Err(message) => {
            eprintln!("{message}\n\n{USAGE}");
            return ExitCode::from(2);
        }
    };
    let outcome = match command {
        Command::Help => {
            println!("{USAGE}");
            Ok(())
        }
        Command::Version => {
            println!("swakshar {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Command::Doctor => doctor::doctor(&options),
        Command::Setup { trust } => doctor::setup(trust),
        Command::Serve => runtime().and_then(|runtime| runtime.block_on(serve::serve(options))),
        Command::Selftest(selftest) => selftest::selftest(&options, &selftest),
        Command::Probe(probe) => {
            runtime().and_then(|runtime| runtime.block_on(probe::probe(&options, &probe)))
        }
    };
    match outcome {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("swakshar: {error}");
            ExitCode::FAILURE
        }
    }
}

/// A multi-threaded Tokio runtime for the network commands.
fn runtime() -> Result<tokio::runtime::Runtime, CliError> {
    Ok(tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?)
}

/// Current time in Unix seconds.
pub(crate) fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| {
            i64::try_from(elapsed.as_secs()).unwrap_or(i64::MAX)
        })
}

/// Shows only the last four characters of a token serial.
pub(crate) fn mask_serial(serial: &str) -> String {
    let tail: String = serial
        .chars()
        .rev()
        .take(4)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    format!("****{tail}")
}
