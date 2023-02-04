//! Logger for Pink contracts.

use core::fmt::Arguments;
pub use log::Level;
use log::{Log, Metadata, Record};

/// A logger working inside a Pink contract.
struct Logger;

impl Log for Logger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
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
            crate::ext().log(level, &message);
        }
    }

    fn flush(&self) {}
}

pub fn log(level: Level, args: Arguments<'_>) {
    let record = Record::builder().level(level).args(args).build();
    Logger.log(&record);
}

/// Same as log::log!
#[macro_export]
macro_rules! log {
    ($level: expr, $($arg:tt)+) => {{ $crate::logger::log($level, ::core::format_args!($($arg)+)) }}
}

/// Same as log::error!
#[macro_export(local_inner_macros)]
macro_rules! error {
    ($($arg:tt)+) => {{ log!($crate::logger::Level::Error, $($arg)+) }}
}

/// Same as log::warn!
#[macro_export(local_inner_macros)]
macro_rules! warn {
    ($($arg:tt)+) => {{ log!($crate::logger::Level::Warn, $($arg)+) }}
}

/// Same as log::info!
#[macro_export(local_inner_macros)]
macro_rules! info {
    ($($arg:tt)+) => {{ log!($crate::logger::Level::Info, $($arg)+) }}
}

/// Same as log::debug!
#[macro_export(local_inner_macros)]
macro_rules! debug {
    ($($arg:tt)+) => {{ log!($crate::logger::Level::Debug, $($arg)+) }}
}

/// Same as log::trace!
#[macro_export(local_inner_macros)]
macro_rules! trace {
    ($($arg:tt)+) => {{ log!($crate::logger::Level::Trace, $($arg)+) }}
}

#[macro_export]
macro_rules! panic {
    ($($arg:tt)+) => {{ $crate::error!($($arg)+); panic!($($arg)+) }}
}

/// An extension for Result<T, E> to log error conveniently.
pub trait ResultExt {
    /// Log the the error message with `pink::error!` with a tip `msg` in front if the Result is Err.
    fn log_err(self, msg: &str) -> Self
    where
        Self: Sized,
    {
        self.log_err_with_level(Level::Error, msg)
    }

    /// Log the the error message with `level` and a tip `msg` in front if the Result is Err.
    fn log_err_with_level(self, level: Level, msg: &str) -> Self
    where
        Self: Sized;
}

impl<T, E: core::fmt::Debug> ResultExt for Result<T, E> {
    fn log_err_with_level(self, level: Level, msg: &str) -> Self {
        if let Err(err) = &self {
            log!(level, "{msg}: {err:?}");
        }
        self
    }
}
