//! Swakshar desktop app: a menu bar signer for the GST portal.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod activity;
mod app;
mod broker;
mod commands;
mod error;
mod pending;
mod server_task;
mod settings;
mod state;
mod tray;
mod views;
mod windows;

/// Starts the app and reports a fatal start-up error.
fn main() {
    if let Err(error) = app::run() {
        eprintln!("Swakshar could not start: {error}");
        std::process::exit(1);
    }
}
