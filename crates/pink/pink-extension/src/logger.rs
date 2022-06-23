//! Logger for Pink contracts.

use core::sync::atomic::{AtomicBool, Ordering};
pub use log::{self, Level};
use log::{Log, Metadata, Record};

/// A logger working inside a Pink contract.
pub struct Logger {
    max_level: Level,
}

impl Logger {
    /// Create a new logger with the given maximum level.
    pub const fn with_max_level(max_level: Level) -> Self {
        Self { max_level }
    }

    /// Install the logger as the global logger.
    pub fn init(&'static self) {
        log::set_max_level(self.max_level.to_level_filter());
        log::set_logger(self).unwrap();
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.max_level
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let message = alloc::format!("{}", record.args());
            let level = match record.level() {
                Level::Error => 1,
                Level::Warn => 2,
                Level::Info => 3,
                Level::Debug => 4,
                Level::Trace => 5,
            };
            let _ = crate::ext().log(level, &message);
        }
    }

    fn flush(&self) {}
}

pub fn once(x: fn()) {
    static INIT_LOCK: AtomicBool = AtomicBool::new(false);
    if !INIT_LOCK.swap(true, Ordering::Relaxed) {
        x();
    }
}

/// Register a logger as global logger for log.
///
/// We can not use `log::set_boxed_logger` or `log::set_logger` in ink due to it's
/// non-contiguous execution model.
#[macro_export]
macro_rules! register_logger {
    ($logger: expr) => {
        #[no_mangle]
        fn pink_logger_try_init() {
            $crate::logger::once(|| $logger.init());
        }
    };
}

#[macro_export]
macro_rules! try_init {
    () => {
        extern "Rust" {
            fn pink_logger_try_init();
        }
        unsafe { pink_logger_try_init() };
    };
}

/// Same as log::error!
#[macro_export(local_inner_macros)]
macro_rules! error {
    ($($arg:tt)+) => {{ try_init!(); $crate::logger::log::error!($($arg)+) }}
}

/// Same as log::warn!
#[macro_export(local_inner_macros)]
macro_rules! warn {
    ($($arg:tt)+) => {{ try_init!(); $crate::logger::log::warn!($($arg)+) }}
}

/// Same as log::info!
#[macro_export(local_inner_macros)]
macro_rules! info {
    ($($arg:tt)+) => {{ try_init!(); $crate::logger::log::info!($($arg)+) }}
}

/// Same as log::debug!
#[macro_export(local_inner_macros)]
macro_rules! debug {
    ($($arg:tt)+) => {{ try_init!(); $crate::logger::log::debug!($($arg)+) }}
}

/// Same as log::trace!
#[macro_export(local_inner_macros)]
macro_rules! trace {
    ($($arg:tt)+) => {{ try_init!(); $crate::logger::log::trace!($($arg)+) }}
}
