//! Companion controls for the vendor's installed macOS service.

use std::net::{Ipv4Addr, SocketAddr, TcpStream};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::error::{CommandError, CommandResult};

/// Vendor service identifier, including the vendor's spelling.
const SERVICE: &str = "system/com.eMudhra.emBridge.deamon";
/// Installed service definition.
const PLIST: &str = "/Library/LaunchDaemons/com.eMudhra.emBridge.plist";
/// Portal transport port.
const PORT: u16 = 26769;
/// Portal hostname covered by the vendor's certificate.
const HOST: &str = "localhost.emudhra.com";
/// Vendor's public download page.
pub(crate) const DOWNLOAD_URL: &str = "https://embridge.emudhra.com/";

/// Read-only status displayed in the portal section.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BridgeStatus {
    /// The app bundle carries the offline vendor installer.
    pub(crate) installer_available: bool,
    /// Service controls are currently implemented on macOS.
    supported: bool,
    /// Vendor service definition exists.
    installed: bool,
    /// launchd reports the service loaded.
    loaded: bool,
    /// Something is listening on the portal port.
    listening: bool,
    /// The local HTTPS certificate passes system trust validation.
    tls_ready: bool,
    /// The hostname is explicitly mapped to loopback.
    hosts_ready: bool,
}

/// Only these fixed actions can cross the privileged boundary.
#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum BridgeAction {
    /// Enable and load the vendor service.
    Start,
    /// Unload and disable the vendor service.
    Stop,
}

/// Inspects installation, service, loopback transport and trust independently.
pub(crate) fn status() -> BridgeStatus {
    let supported = cfg!(target_os = "macos");
    let installed = supported && Path::new(PLIST).is_file();
    let loaded = installed
        && Command::new("/bin/launchctl")
            .args(["print", SERVICE])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|result| result.success());
    let listening = supported
        && TcpStream::connect_timeout(
            &SocketAddr::from((Ipv4Addr::LOCALHOST, PORT)),
            Duration::from_millis(300),
        )
        .is_ok();
    let tls_ready = listening
        && Command::new("/usr/bin/curl")
            .args([
                "--silent",
                "--max-time",
                "3",
                "--noproxy",
                "*",
                "--resolve",
                "localhost.emudhra.com:26769:127.0.0.1",
                "--output",
                "/dev/null",
                "https://localhost.emudhra.com:26769/",
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|result| result.success());
    let hosts_ready =
        std::fs::read_to_string("/etc/hosts").is_ok_and(|text| hosts_maps_loopback(&text));
    BridgeStatus {
        installer_available: false,
        supported,
        installed,
        loaded,
        listening,
        tls_ready,
        hosts_ready,
    }
}

/// Checks exact host aliases while ignoring comments and non-loopback entries.
fn hosts_maps_loopback(text: &str) -> bool {
    text.lines().any(|line| {
        let mut words = line
            .split('#')
            .next()
            .unwrap_or_default()
            .split_whitespace();
        words.next() == Some("127.0.0.1") && words.any(|word| word == HOST)
    })
}

/// Runs a fixed launchctl operation after macOS obtains administrator approval.
pub(crate) fn control(action: BridgeAction) -> CommandResult<BridgeStatus> {
    let current = status();
    if !current.installed {
        return Err(CommandError::Message(
            "Install emBridge from its official download page first.".to_owned(),
        ));
    }
    let shell = match action {
        BridgeAction::Start if current.listening && !current.loaded => {
            return Err(CommandError::Message(
                "Port 26769 is occupied by another service. Close it before starting emBridge."
                    .to_owned(),
            ));
        }
        BridgeAction::Start => {
            "/bin/launchctl enable system/com.eMudhra.emBridge.deamon && (/bin/launchctl bootstrap system /Library/LaunchDaemons/com.eMudhra.emBridge.plist || /bin/launchctl kickstart system/com.eMudhra.emBridge.deamon)"
        }
        BridgeAction::Stop if !current.loaded => return Ok(current),
        BridgeAction::Stop => {
            "/bin/launchctl bootout system /Library/LaunchDaemons/com.eMudhra.emBridge.plist && /bin/launchctl disable system/com.eMudhra.emBridge.deamon"
        }
    };
    let script = format!("do shell script \"{shell}\" with administrator privileges");
    let result = Command::new("/usr/bin/osascript")
        .args(["-e", &script])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;
    if !result.success() {
        return Err(CommandError::Message("The service action was cancelled or macOS could not complete it. Open emBridge to finish its setup, then refresh.".to_owned()));
    }
    Ok(status())
}

#[cfg(test)]
mod tests {
    use super::hosts_maps_loopback;

    /// Comments, similar hostnames and public addresses cannot pass the check.
    #[test]
    fn exact_loopback_alias_is_required() {
        assert!(hosts_maps_loopback(
            "127.0.0.1 localhost localhost.emudhra.com # local"
        ));
        for text in [
            "# 127.0.0.1 localhost.emudhra.com",
            "127.0.0.1 localhost.emudhra.com.evil",
            "192.0.2.1 localhost.emudhra.com",
        ] {
            assert!(!hosts_maps_loopback(text));
        }
    }
}
