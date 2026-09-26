//! State shared by commands, the broker and the tray.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use swakshar_token::TokenService;

use crate::activity::ActivityLog;
use crate::commands::overview::TrustView;
use crate::pending::Pending;
use crate::server_task::ServerControl;
use crate::settings::Settings;

/// What the app has learnt from connections, for the Help view.
#[derive(Debug, Clone, Default)]
pub(crate) struct Diagnostics {
    /// Last allowed page that connected, and when.
    pub(crate) last_connection: Option<(String, i64)>,
    /// Last TLS failure, and when.
    pub(crate) last_tls_failure: Option<(String, i64)>,
    /// A browser has loaded the status page.
    pub(crate) status_page_seen: bool,
}

/// Everything shared across the app.
pub(crate) struct AppState {
    /// Per-user data directory.
    pub(crate) data_dir: PathBuf,
    /// Token thread.
    pub(crate) token: TokenService,
    /// Current settings.
    pub(crate) settings: Mutex<Settings>,
    /// Loopback server state and task.
    pub(crate) server: Mutex<ServerControl>,
    /// The request waiting for approval, if any.
    pub(crate) pending: Mutex<Option<Pending>>,
    /// Connection diagnostics.
    pub(crate) diagnostics: Mutex<Diagnostics>,
    /// Request history.
    pub(crate) activity: ActivityLog,
    /// Set while a request holds the single approval slot.
    pub(crate) busy: AtomicBool,
    /// Last trust check and when it ran; asking macOS spawns a process.
    pub(crate) trust_cache: Mutex<Option<(Instant, TrustView)>>,
    /// Next request id.
    next_id: AtomicU64,
}

impl AppState {
    /// Fresh state around a started token thread and loaded settings.
    pub(crate) fn new(data_dir: PathBuf, token: TokenService, settings: Settings) -> Self {
        Self {
            activity: ActivityLog::new(data_dir.clone()),
            data_dir,
            token,
            settings: Mutex::new(settings),
            server: Mutex::new(ServerControl::default()),
            pending: Mutex::new(None),
            diagnostics: Mutex::new(Diagnostics::default()),
            busy: AtomicBool::new(false),
            trust_cache: Mutex::new(None),
            next_id: AtomicU64::new(1),
        }
    }

    /// A copy of the current settings.
    pub(crate) fn settings(&self) -> Settings {
        lock(&self.settings).clone()
    }

    /// A new request id.
    pub(crate) fn next_id(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }
}

/// Locks a mutex, recovering the data if a panicking thread poisoned it.
pub(crate) fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
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
    let tail: Vec<char> = serial.chars().rev().take(4).collect();
    format!("****{}", tail.iter().rev().collect::<String>())
}
