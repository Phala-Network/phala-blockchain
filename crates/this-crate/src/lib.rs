//! This crate provides utility macros to work with crate version information.
//!
//! The two main macros are `version_str!` and `version_tuple!`. These macros return the
//! crate version in string format and as a tuple of three `u16` integers, respectively.
//!
//! # Examples
//!
//! ```rust
//! use this_crate::{version_str, version_tuple};
//! let version = version_str!();
//! println!("The version is: {}", version);
//!
//! let (major, minor, patch) = version_tuple!();
//! println!("The version is: {}.{}.{}", major, minor, patch);
//! ```
//!
//! # no_std
//! This crate is `no_std` compatible, so it can be used in environments without
//! the Rust standard library.
//! 
#![no_std]

/// A tuple representing the version of the crate as (major, minor, patch).
pub type VersionTuple = (u16, u16, u16);


/// `version_str!` is a macro that returns the version string of the crate.
///
/// # Examples
///
/// ```
/// use this_crate::version_str;
/// let version = version_str!();
/// println!("The version is: {}", version);
/// ```
#[macro_export]
macro_rules! version_str {
    () => {
        env!("CARGO_PKG_VERSION")
    };
}

/// `version_tuple!` is a macro that returns the version of the crate as a tuple
/// of three `u16` integers: (major, minor, patch).
///
/// # Examples
///
/// ```
/// use this_crate::version_tuple;
/// let (major, minor, patch) = version_tuple!();
/// println!("The version is: {}.{}.{}", major, minor, patch);
/// ```
#[macro_export]
macro_rules! version_tuple {
    () => {{
        let major = env!("CARGO_PKG_VERSION_MAJOR").parse::<u16>().unwrap_or(0);
        let minor = env!("CARGO_PKG_VERSION_MINOR").parse::<u16>().unwrap_or(0);
        let patch = env!("CARGO_PKG_VERSION_PATCH").parse::<u16>().unwrap_or(0);
        (major, minor, patch)
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(version_str!(), "0.1.0");
        assert_eq!(version_tuple!(), (0, 1, 0));
    }
}
