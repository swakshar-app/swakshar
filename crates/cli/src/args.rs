//! A small, dependency-free argument parser.

use std::path::PathBuf;

/// Content signed by `selftest` when none is given.
const DEFAULT_SELFTEST_CONTENT: &str = "swakshar-selftest";
/// Content sent by `probe` when none is given.
const DEFAULT_PROBE_CONTENT: &str = "swakshar-probe";

/// The command to run.
pub(crate) enum Command {
    /// Print usage.
    Help,
    /// Print the version.
    Version,
    /// Diagnose the machine.
    Doctor,
    /// Create, and optionally trust, the local certificate.
    Setup {
        /// Add the CA to the login keychain.
        trust: bool,
    },
    /// Run the signer with terminal approval.
    Serve,
    /// Sign directly on the token.
    Selftest(SelftestArgs),
    /// Emulate the portal against a running signer.
    Probe(ProbeArgs),
}

/// `selftest` options.
pub(crate) struct SelftestArgs {
    /// Text to sign.
    pub(crate) content: String,
    /// Fixed signing time, Unix seconds.
    pub(crate) signing_time: Option<u64>,
    /// Token serial to use.
    pub(crate) token: Option<String>,
    /// File to write the DER signature to.
    pub(crate) out: Option<PathBuf>,
}

/// `probe` options.
pub(crate) struct ProbeArgs {
    /// Text to have signed.
    pub(crate) content: String,
    /// PAN to send, like the portal.
    pub(crate) pan: Option<String>,
}

/// Options shared by several commands.
#[derive(Default)]
pub(crate) struct Options {
    /// Extra PKCS#11 modules.
    pub(crate) modules: Vec<PathBuf>,
    /// Preferred port.
    pub(crate) port: Option<u16>,
    /// Greeting version override.
    pub(crate) greeting_version: Option<String>,
    /// Extra allowed origins.
    pub(crate) allow_origins: Vec<String>,
}

/// Command-specific flags collected before the command is built.
#[derive(Default)]
struct Flags {
    /// `--trust`.
    trust: bool,
    /// `--content`.
    content: Option<String>,
    /// `--signing-time`.
    signing_time: Option<u64>,
    /// `--token`.
    token: Option<String>,
    /// `--out`.
    out: Option<PathBuf>,
    /// `--pan`.
    pan: Option<String>,
}

/// Parses `<command> [options]`.
pub(crate) fn parse(mut args: impl Iterator<Item = String>) -> Result<(Command, Options), String> {
    let name = args.next().unwrap_or_else(|| "help".to_owned());
    let mut options = Options::default();
    let mut flags = Flags::default();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--module" => options.modules.push(PathBuf::from(take(&mut args, &arg)?)),
            "--port" => options.port = Some(number(&take(&mut args, &arg)?, &arg)?),
            "--greeting-version" => options.greeting_version = Some(take(&mut args, &arg)?),
            "--allow-origin" => options.allow_origins.push(take(&mut args, &arg)?),
            "--trust" => flags.trust = true,
            "--content" => flags.content = Some(take(&mut args, &arg)?),
            "--signing-time" => flags.signing_time = Some(number(&take(&mut args, &arg)?, &arg)?),
            "--token" => flags.token = Some(take(&mut args, &arg)?),
            "--out" => flags.out = Some(PathBuf::from(take(&mut args, &arg)?)),
            "--pan" => flags.pan = Some(take(&mut args, &arg)?),
            "-h" | "--help" => return Ok((Command::Help, options)),
            other => return Err(format!("unknown option {other}")),
        }
    }
    let command = match name.as_str() {
        "help" | "-h" | "--help" => Command::Help,
        "version" | "-V" | "--version" => Command::Version,
        "doctor" => Command::Doctor,
        "setup" => Command::Setup { trust: flags.trust },
        "serve" => Command::Serve,
        "selftest" => Command::Selftest(SelftestArgs {
            content: flags
                .content
                .unwrap_or_else(|| DEFAULT_SELFTEST_CONTENT.to_owned()),
            signing_time: flags.signing_time,
            token: flags.token,
            out: flags.out,
        }),
        "probe" => Command::Probe(ProbeArgs {
            content: flags
                .content
                .unwrap_or_else(|| DEFAULT_PROBE_CONTENT.to_owned()),
            pan: flags.pan,
        }),
        other => return Err(format!("unknown command {other}")),
    };
    Ok((command, options))
}

/// The value after a flag.
fn take(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<String, String> {
    args.next().ok_or_else(|| format!("{flag} needs a value"))
}

/// Parses a numeric flag value.
fn number<T: std::str::FromStr>(value: &str, flag: &str) -> Result<T, String> {
    value
        .parse()
        .map_err(|_| format!("{flag} expects a number, got {value}"))
}
