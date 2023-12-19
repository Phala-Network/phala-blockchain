//! Logger for sidevm programs.

pub use log::LevelFilter;
use log::{Log, Metadata, Record};
use sidevm_env::ocall_funcs_guest as ocall;

/// A logger working inside a Sidevm.
pub struct Logger {
    max_level: LevelFilter,
}

impl Logger {
    /// Create a new logger with the given maximum level.
    pub const fn with_max_level(max_level: LevelFilter) -> Self {
        Self { max_level }
    }

    /// Install the logger as the global logger.
    pub fn init(&'static self) {
        log::set_max_level(self.max_level);
        log::set_logger(self).unwrap();
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.max_level
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let message = format!("{}", record.args());

            let _ = ocall::log(record.level(), &message);
        }
    }

    fn flush(&self) {}
}
