//! Substrate Node Template CLI library.

#![warn(missing_docs)]
#![warn(unused_extern_crates)]

mod chain_spec;
#[macro_use]
mod service;
mod cli;

pub use substrate_cli::{VersionInfo, IntoExit, error};

fn main() -> Result<(), cli::error::Error> {
	let version = VersionInfo {
		name: "Experimental Substrate Node",
		commit: env!("VERGEN_SHA_SHORT"),
		version: env!("CARGO_PKG_VERSION"),
		executable_name: "experimental-node",
		author: "Jasl",
		description: "Experimental Substrate Node to evaluate PKI verification on Runtime.",
		support_url: "support.anonymous.an",
	};

	cli::run(std::env::args(), cli::Exit, version)
}
