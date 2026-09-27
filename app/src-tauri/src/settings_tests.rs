//! Settings: defaults for a fresh install, compatibility with files written
//! by older versions, and validation.

use super::Settings;

/// A fresh install: signing off, updates and notifications on.
#[test]
fn fresh_install_defaults() {
    let settings = Settings::default();
    assert!(!settings.signing_enabled);
    assert!(settings.update_checks);
    assert!(settings.notifications);
    assert!(!settings.onboarding_complete);
    assert!(settings.validate().is_ok());
}

/// A file from before the newer switches keeps its values and gets the
/// defaults for the rest.
#[test]
fn older_files_gain_the_new_defaults() {
    let settings: Settings =
        serde_json::from_str(r#"{"onboardingComplete":true,"greetingVersion":"2.8"}"#).unwrap();
    assert!(settings.onboarding_complete);
    assert!(!settings.signing_enabled);
    assert!(settings.update_checks);
    assert!(settings.notifications);
}

/// Saved and loaded settings are identical.
#[test]
fn round_trips_through_disk() {
    let dir = std::env::temp_dir().join(format!("swakshar-settings-{}", std::process::id()));
    let settings = Settings {
        signing_enabled: true,
        notifications: false,
        extra_origins: vec!["https://example.gov.in".to_owned()],
        ..Settings::default()
    };
    settings.save(&dir).unwrap();
    assert_eq!(Settings::load(&dir), settings);
    std::fs::remove_dir_all(&dir).unwrap();
}

/// Unreadable files fall back to the defaults instead of failing.
#[test]
fn unreadable_files_fall_back_to_defaults() {
    let dir = std::env::temp_dir().join(format!("swakshar-settings-bad-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("settings.json"), "{not json").unwrap();
    assert_eq!(Settings::load(&dir), Settings::default());
    std::fs::remove_dir_all(&dir).unwrap();
}

/// Validation names the first bad value.
#[test]
fn validation_rejects_bad_values() {
    let port = Settings {
        preferred_port: Some(8080),
        ..Settings::default()
    };
    assert!(port.validate().unwrap_err().contains("8080"));
    let origin = Settings {
        extra_origins: vec!["http://example.gov.in".to_owned()],
        ..Settings::default()
    };
    assert!(origin.validate().unwrap_err().contains("https://"));
    let module = Settings {
        modules: vec!["relative/driver.dylib".to_owned()],
        ..Settings::default()
    };
    assert!(module.validate().unwrap_err().contains("full path"));
    let version = Settings {
        greeting_version: "2.8; rm".to_owned(),
        ..Settings::default()
    };
    assert!(version.validate().is_err());
}
