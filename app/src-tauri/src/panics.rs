//! Panics are written to the log before the default handler prints them. A
//! bundled app has no terminal: a panic on the exit path once stopped the
//! relaunch into an update, and its only trace went to a stderr nobody saw.

/// Installs the hook. Call once, before the app starts.
pub(crate) fn log_panics() {
    let default = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        log::error!("{info}");
        default(info);
    }));
}

#[cfg(test)]
#[path = "panics_tests.rs"]
mod tests;
