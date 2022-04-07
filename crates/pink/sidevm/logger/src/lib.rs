use log::{Level, Metadata};

/// A logger working inside a SideVM.
pub struct Logger {
    max_level: Level,
}

impl Logger {
    /// Create a new logger with the given maximum level.
    pub fn with_max_level(max_level: Level) -> Self {
        Self { max_level }
    }

    /// Install the logger as the global logger.
    pub fn init(self) {
        log::set_boxed_logger(Box::new(self)).unwrap();
        log::set_max_level(self.max_level);
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.max_level
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let message = format!("{}", record.args());
        }
    }

    fn flush(&self) {}
}
