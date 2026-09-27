//! The history file: order, trimming, clearing and damaged lines.

use std::path::PathBuf;

use super::{ActivityEntry, ActivityLog, KEEP_ENTRIES, TRIM_AT};

/// A fresh history in its own temporary folder.
fn log(name: &str) -> (ActivityLog, PathBuf) {
    let dir = std::env::temp_dir().join(format!("swakshar-activity-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    (ActivityLog::new(dir.clone()), dir)
}

/// An entry that ended at `at`.
fn entry(at: i64) -> ActivityEntry {
    ActivityEntry {
        at,
        origin: "https://services.gst.gov.in".to_owned(),
        kind: "document".to_owned(),
        pan_masked: None,
        holder: None,
        token: Some("****0001".to_owned()),
        outcome: "signed".to_owned(),
    }
}

/// Newest first, cut to the limit.
#[test]
fn recent_is_newest_first() {
    let (history, dir) = log("order");
    for at in 1..=3 {
        history.append(&entry(at));
    }
    let times: Vec<i64> = history.recent(2).iter().map(|entry| entry.at).collect();
    assert_eq!(times, [3, 2]);
    std::fs::remove_dir_all(dir).unwrap();
}

/// Past `TRIM_AT` entries the file keeps only the newest `KEEP_ENTRIES`.
#[test]
fn trims_old_entries() {
    let (history, dir) = log("trim");
    let total = i64::try_from(TRIM_AT + 1).unwrap();
    for at in 1..=total {
        history.append(&entry(at));
    }
    let kept = history.recent(usize::MAX);
    assert_eq!(kept.len(), KEEP_ENTRIES);
    assert_eq!(kept.first().map(|entry| entry.at), Some(total));
    std::fs::remove_dir_all(dir).unwrap();
}

/// Clearing empties the history.
#[test]
fn clear_empties() {
    let (history, dir) = log("clear");
    history.append(&entry(1));
    history.clear();
    assert!(history.recent(10).is_empty());
    std::fs::remove_dir_all(dir).unwrap();
}

/// A damaged line is skipped, not fatal.
#[test]
fn skips_damaged_lines() {
    let (history, dir) = log("damaged");
    history.append(&entry(1));
    let file = dir.join("activity.jsonl");
    let mut text = std::fs::read_to_string(&file).unwrap();
    text.push_str("{not json\n");
    std::fs::write(&file, text).unwrap();
    history.append(&entry(2));
    let times: Vec<i64> = history.recent(10).iter().map(|entry| entry.at).collect();
    assert_eq!(times, [2, 1]);
    std::fs::remove_dir_all(dir).unwrap();
}
