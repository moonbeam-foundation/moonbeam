//! Substrate Node Template CLI library.
#![warn(missing_docs)]

mod chain_spec;
#[macro_use]
mod service;
mod cli;
mod command;

pub use sc_cli::{VersionInfo, error};

fn main() -> Result<(), error::Error> {
	let version = VersionInfo {
		name: "Moonbeam",
		commit: env!("VERGEN_SHA_SHORT"),
		version: env!("CARGO_PKG_VERSION"),
		executable_name: "moonbeam",
		author: "PureStake",
		description: "Moonbeam core",
		support_url: "purestake.io",
		copyright_start_year: 2020,
	};

	command::run(version)
}
