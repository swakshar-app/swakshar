//! Swakshar desktop app: a menu bar signer for the GST portal.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod activity;
mod app;
mod attached;
mod broker;
mod cache;
mod commands;
mod dock;
mod error;
mod notify;
mod panics;
mod pending;
mod quit;
mod relaunch;
mod server_task;
mod settings;
mod state;
mod tray;
mod update_install;
mod update_rules;
mod updates;
mod views;
mod windows;

/// Starts the app and reports a fatal start-up error.
fn main() {
    panics::log_panics();
    if let Err(error) = app::run() {
        eprintln!("Swakshar could not start: {error}");
        std::process::exit(1);
    }
}
