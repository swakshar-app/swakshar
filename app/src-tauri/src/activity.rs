//! A local history of requests, as JSON lines. Never holds PINs, signatures
//! or unmasked PANs.

use std::fs::{self, OpenOptions};
use std::io::Write as _;
use std::path::PathBuf;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use crate::state::lock;

/// History file name.
const ACTIVITY_FILE: &str = "activity.jsonl";
/// Entries kept after trimming.
const KEEP_ENTRIES: usize = 500;
/// Trim once the file grows past this many entries.
const TRIM_AT: usize = 1_000;

/// One finished request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ActivityEntry {
    /// When it ended, Unix seconds.
    pub(crate) at: i64,
    /// Page origin.
    pub(crate) origin: String,
    /// `registration` or `document`.
    pub(crate) kind: String,
    /// Masked PAN, when the request had one.
    pub(crate) pan_masked: Option<String>,
    /// Certificate holder, when one was used.
    pub(crate) holder: Option<String>,
    /// Masked token serial, when one was used.
    pub(crate) token: Option<String>,
    /// `signed`, `canceled`, `timed-out`, `abandoned`, `refused`.
    pub(crate) outcome: String,
}

/// Append-only history file with a lock for writers.
pub(crate) struct ActivityLog {
    /// File path.
    path: PathBuf,
    /// Serialises writers.
    guard: Mutex<()>,
}

impl ActivityLog {
    /// History stored in `dir`.
    pub(crate) fn new(dir: PathBuf) -> Self {
        Self {
            path: dir.join(ACTIVITY_FILE),
            guard: Mutex::new(()),
        }
    }

    /// Appends an entry; failures are logged, never fatal.
    pub(crate) fn append(&self, entry: &ActivityEntry) {
        let _guard = lock(&self.guard);
        let written = serde_json::to_string(entry)
            .map_err(std::io::Error::other)
            .and_then(|line| {
                let mut file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&self.path)?;
                writeln!(file, "{line}")
            });
        if let Err(error) = written {
            log::warn!("could not record activity: {error}");
        }
        let all = self.read_all();
        if all.len() > TRIM_AT {
            self.write_all(
                all.get(all.len().saturating_sub(KEEP_ENTRIES)..)
                    .unwrap_or_default(),
            );
        }
    }

    /// Newest first, at most `limit` entries.
    pub(crate) fn recent(&self, limit: usize) -> Vec<ActivityEntry> {
        let _guard = lock(&self.guard);
        let mut entries = self.read_all();
        entries.reverse();
        entries.truncate(limit);
        entries
    }

    /// Deletes the history.
    pub(crate) fn clear(&self) {
        let _guard = lock(&self.guard);
        self.write_all(&[]);
    }

    /// Every readable entry, oldest first.
    fn read_all(&self) -> Vec<ActivityEntry> {
        fs::read_to_string(&self.path)
            .unwrap_or_default()
            .lines()
            .filter_map(|line| serde_json::from_str(line).ok())
            .collect()
    }

    /// Replaces the file with `entries`.
    fn write_all(&self, entries: &[ActivityEntry]) {
        let text: String = entries
            .iter()
            .filter_map(|entry| serde_json::to_string(entry).ok())
            .map(|line| line + "\n")
            .collect();
        if let Err(error) = fs::write(&self.path, text) {
            log::warn!("could not rewrite activity: {error}");
        }
    }
}

#[cfg(test)]
#[path = "activity_tests.rs"]
mod tests;
