//! `serve`: the portal signer, with approval in this terminal.

use std::path::PathBuf;
use std::sync::Arc;

use swakshar_protocol::{
    DEFAULT_GREETING_VERSION, OriginPolicy, REPLY_CANCELED, REPLY_FAILED, RequestKind,
    SUPPORTED_SIGNTYPE, SignRequest, mask_pan,
};
use swakshar_server::{
    Broker, PortalRequest, ServerEvent, ServerSettings, bind_signer_port, tls_config,
};
use swakshar_tls::{TlsError, TrustStatus, data_dir, ensure_identity, tls_dir, trust_status};
use swakshar_token::{TokenService, candidates, criteria_for, portal_reply};
use tokio::sync::Mutex;

use crate::args::Options;
use crate::error::CliError;
use crate::prompt::{choose, confirm};
use crate::signing::{Signed, sign_interactively};
use crate::unix_now;

/// Serves the portal until Ctrl-C.
pub(crate) async fn serve(options: Options) -> Result<(), CliError> {
    let data_dir = data_dir().ok_or(TlsError::NoDataDir)?;
    let (identity, _) = ensure_identity(&tls_dir(&data_dir), unix_now())?;
    if trust_status(&identity) == TrustStatus::NotTrusted {
        log::warn!(
            "browsers will refuse the local certificate until you run `swakshar setup --trust`"
        );
    }
    let tls = tls_config(&identity.cert_der, &identity.key_der)?;
    let (listener, port) = bind_signer_port(options.port).await?;
    let settings = ServerSettings {
        greeting_version: options
            .greeting_version
            .clone()
            .unwrap_or_else(|| DEFAULT_GREETING_VERSION.to_owned()),
        origins: OriginPolicy::new(&options.allow_origins),
    };
    let broker = Arc::new(TerminalBroker {
        service: TokenService::spawn()?,
        modules: options.modules,
        turn: Mutex::new(()),
    });
    println!("Listening on wss://127.0.0.1:{port} for the GST portal. Press Ctrl-C to stop.");
    tokio::select! {
        served = swakshar_server::serve(listener, port, tls, settings, broker) => Ok(served?),
        stopped = tokio::signal::ctrl_c() => {
            stopped?;
            println!("Stopped.");
            Ok(())
        }
    }
}

/// Decides requests by asking in the terminal, one at a time.
struct TerminalBroker {
    /// Token thread.
    service: TokenService,
    /// Extra driver paths.
    modules: Vec<PathBuf>,
    /// Serialises prompts.
    turn: Mutex<()>,
}

impl Broker for TerminalBroker {
    /// Runs the blocking prompts on a blocking thread.
    async fn handle(&self, request: PortalRequest) -> String {
        let _turn = self.turn.lock().await;
        let service = self.service.clone();
        let modules = self.modules.clone();
        tokio::task::spawn_blocking(move || decide(&service, modules, &request))
            .await
            .unwrap_or_else(|error| {
                log::error!("the approval prompt failed: {error}");
                REPLY_FAILED.to_owned()
            })
    }

    /// Logs connection events.
    fn notify(&self, event: ServerEvent) {
        match event {
            ServerEvent::Connected { origin } => log::info!("{origin} connected"),
            ServerEvent::Rejected { origin, reason } => {
                log::warn!("refused a connection from {origin:?}: {reason}");
            }
            ServerEvent::TlsFailed { reason } => {
                log::warn!(
                    "a browser refused the local certificate ({reason}); run `swakshar setup --trust`"
                );
            }
            ServerEvent::StatusPageServed => log::info!("a browser loaded the status page"),
        }
    }
}

/// Shows the request, picks a certificate, confirms, signs.
fn decide(service: &TokenService, modules: Vec<PathBuf>, portal: &PortalRequest) -> String {
    let request = &portal.request;
    println!();
    println!(
        "{} asks for a signature: {}",
        portal.origin,
        describe(request)
    );
    if request.signtype != SUPPORTED_SIGNTYPE {
        println!("signtype {} is not supported; refusing.", request.signtype);
        return REPLY_FAILED.to_owned();
    }
    let inventory = match service.inventory_blocking(modules) {
        Ok(inventory) => inventory,
        Err(error) => {
            println!("Could not read the tokens: {error}");
            return REPLY_FAILED.to_owned();
        }
    };
    let now = unix_now();
    let found = candidates(&inventory.tokens, &criteria_for(request, now));
    let Some(candidate) = choose(&inventory, &found) else {
        return REPLY_CANCELED.to_owned();
    };
    if !confirm("Sign this request?") {
        return REPLY_CANCELED.to_owned();
    }
    match sign_interactively(
        service,
        &inventory,
        &candidate,
        request.content.as_bytes(),
        now,
    ) {
        Ok(Signed::Done(output)) => {
            println!("Signed with {}.", output.summary.subject_cn);
            portal_reply(&output, request, now)
        }
        Ok(Signed::Canceled) => REPLY_CANCELED.to_owned(),
        Err(error) => {
            println!("Signing failed: {error}");
            REPLY_FAILED.to_owned()
        }
    }
}

/// One-line description of what is being signed.
fn describe(request: &SignRequest) -> String {
    match request.kind() {
        RequestKind::Registration => {
            format!("DSC registration for PAN {}", mask_pan(&request.content))
        }
        RequestKind::Document => format!("document {}", request.content),
    }
}
