//! # Woothee
//!
//! Woothee is a user-agent strings parser.
//!
//! ## Usage
//!
//! ```toml
//! [dependencies]
//! woothee = "*"
//! ```
//!
//! ```rust
//! use woothee::parser::Parser;
//! let parser = Parser::new();
//! let result = parser.parse("Mozilla/4.0 (compatible; MSIE 8.0; Windows NT 6.1; Trident/4.0)");
//! println!("{:?}", result);
//! ```
//!

pub mod dataset;
pub mod parser;
pub mod woothee;
