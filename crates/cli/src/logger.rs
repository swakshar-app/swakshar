//! Minimal stderr logger for the `log` facade. `SWAKSHAR_LOG` sets the level.

use log::{LevelFilter, Log, Metadata, Record};

/// Environment variable that selects the log level.
const LEVEL_VARIABLE: &str = "SWAKSHAR_LOG";

/// Writes each record as one line on stderr.
struct StderrLogger;

/// The process-wide logger instance.
static LOGGER: StderrLogger = StderrLogger;

impl Log for StderrLogger {
    /// Honours the global maximum level.
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        metadata.level() <= log::max_level()
    }

    /// Prints one record.
    fn log(&self, record: &Record<'_>) {
        if self.enabled(record.metadata()) {
            eprintln!("{:<5} {}", record.level(), record.args());
        }
    }

    /// Nothing is buffered.
    fn flush(&self) {}
}

/// Installs the logger once; later calls are ignored.
pub(crate) fn init() {
    let level = match std::env::var(LEVEL_VARIABLE).as_deref() {
        Ok("trace") => LevelFilter::Trace,
        Ok("debug") => LevelFilter::Debug,
        Ok("warn") => LevelFilter::Warn,
        Ok("error") => LevelFilter::Error,
        _ => LevelFilter::Info,
    };
    if log::set_logger(&LOGGER).is_ok() {
        log::set_max_level(level);
    }
}
