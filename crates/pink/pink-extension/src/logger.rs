//! Logger for Pink contracts.

use core::fmt::Arguments;
pub use log::Level;

pub fn log(level: Level, args: Arguments<'_>) {
    let message = alloc::format!("{}", args);
    let level = match level {
        Level::Error => 1,
        Level::Warn => 2,
        Level::Info => 3,
        Level::Debug => 4,
        Level::Trace => 5,
    };
    crate::ext().log(level, &message);
}

/// The `log!` macro allows you to log messages with specific logging levels in pink contract.
///
/// It is a flexible macro that uses a provided log level (trace, debug, info, warn, error),
/// followed by a format string and an optional list of arguments to generate the final log message.
#[macro_export]
macro_rules! log {
    ($level: expr, $($arg:tt)+) => {{ $crate::logger::log($level, ::core::format_args!($($arg)+)) }}
}

/// Same as `info!` but at Error level.
#[macro_export(local_inner_macros)]
macro_rules! error {
    ($($arg:tt)+) => {{ log!($crate::logger::Level::Error, $($arg)+) }}
}

/// Same as `info!` but at Warn level.
#[macro_export(local_inner_macros)]
macro_rules! warn {
    ($($arg:tt)+) => {{ log!($crate::logger::Level::Warn, $($arg)+) }}
}

/// Macro `info!` logs messages at the Info level in pink contract.
///
/// This macro is used to log information that would be helpful to understand the general flow
/// of the system's execution. It is similar to `log::info`, but it is specifically designed
/// to work within the pink contract environment.
///
/// # Examples
///
/// Basic usage:
///
/// ```ignore
/// use pink_extension as pink;
/// pink::info!("This is an information message.");
/// let answer = 42;
/// pink::info!("The answer is {}.", answer);
/// ```
///
/// The above example would log "This is an information message." and
/// "The answer is 42." at the Info level.
#[macro_export(local_inner_macros)]
macro_rules! info {
    ($($arg:tt)+) => {{ log!($crate::logger::Level::Info, $($arg)+) }}
}

/// Same as `info!` but at Debug level.
#[macro_export(local_inner_macros)]
macro_rules! debug {
    ($($arg:tt)+) => {{ log!($crate::logger::Level::Debug, $($arg)+) }}
}

/// Same as `info!` but at Trace level.
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
