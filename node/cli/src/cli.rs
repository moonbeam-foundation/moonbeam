// Copyright 2019-2022 PureStake Inc.
// This file is part of Moonbeam.

// Moonbeam is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Moonbeam is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Moonbeam.  If not, see <http://www.gnu.org/licenses/>.

//! Moonbeam CLI Library. Built with clap
//!
//! This module defines the Moonbeam node's Command Line Interface (CLI)
//! It is built using clap and inherits behavior from Substrate's sc_cli crate.

use clap::Parser;
use moonbeam_cli_opt::{account_key::GenerateAccountKey, EthApi, Sealing};
use moonbeam_service::chain_spec;
use sc_cli::{Error as CliError, SubstrateCli};
use std::path::PathBuf;

/// Sub-commands supported by the collator.
#[derive(Debug, clap::Subcommand)]
pub enum Subcommand {
	/// Export the genesis state of the parachain.
	#[clap(name = "export-genesis-state")]
	ExportGenesisState(ExportGenesisStateCommand),

	/// Export the genesis wasm of the parachain.
	#[clap(name = "export-genesis-wasm")]
	ExportGenesisWasm(ExportGenesisWasmCommand),

	/// Build a chain specification.
	BuildSpec(BuildSpecCommand),

	/// Validate blocks.
	CheckBlock(sc_cli::CheckBlockCmd),

	/// Export blocks.
	ExportBlocks(sc_cli::ExportBlocksCmd),

	/// Export the state of a given block into a chain spec.
	ExportState(sc_cli::ExportStateCmd),

	/// Import blocks.
	ImportBlocks(sc_cli::ImportBlocksCmd),

	/// Remove the whole chain.
	PurgeChain(cumulus_client_cli::PurgeChainCmd),

	/// Revert the chain to a previous state.
	Revert(sc_cli::RevertCmd),

	/// Sub-commands concerned with benchmarking.
	/// The pallet benchmarking moved to the `pallet` sub-command.
	#[clap(subcommand)]
	Benchmark(frame_benchmarking_cli::BenchmarkCmd),

	/// Try some command against runtime state.
	#[cfg(feature = "try-runtime")]
	TryRuntime(try_runtime_cli::TryRuntimeCmd),

	/// Try some command against runtime state. Note: `try-runtime` feature must be enabled.
	#[cfg(not(feature = "try-runtime"))]
	TryRuntime,

	/// Key management cli utilities
	#[clap(subcommand)]
	Key(KeyCmd),
}

#[derive(Debug, Parser)]
pub struct BuildSpecCommand {
	#[clap(flatten)]
	pub base: sc_cli::BuildSpecCmd,

	/// Number of accounts to be funded in the genesis
	/// Warning: This flag implies a development spec and overrides any explicitly supplied spec
	#[clap(long, conflicts_with = "chain")]
	pub accounts: Option<u32>,

	/// Mnemonic from which we can derive funded accounts in the genesis
	/// Warning: This flag implies a development spec and overrides any explicitly supplied spec
	#[clap(long, conflicts_with = "chain")]
	pub mnemonic: Option<String>,
}

/// Command for exporting the genesis state of the parachain
#[derive(Debug, Parser)]
pub struct ExportGenesisStateCommand {
	/// Output file name or stdout if unspecified.
	#[clap(value_parser)]
	pub output: Option<PathBuf>,

	/// Id of the parachain this state is for.
	#[clap(long)]
	pub parachain_id: Option<u32>,

	/// Write output in binary. Default is to write in hex.
	#[clap(short, long)]
	pub raw: bool,

	/// The name of the chain for that the genesis state should be exported.
	#[clap(long)]
	pub chain: Option<String>,
}

/// Command for exporting the genesis wasm file.
#[derive(Debug, Parser)]
pub struct ExportGenesisWasmCommand {
	/// Output file name or stdout if unspecified.
	#[clap(value_parser)]
	pub output: Option<PathBuf>,

	/// Write output in binary. Default is to write in hex.
	#[clap(short, long)]
	pub raw: bool,

	/// The name of the chain for that the genesis wasm file should be exported.
	#[clap(long)]
	pub chain: Option<String>,
}

#[derive(Debug, Parser)]
#[group(skip)]
pub struct RunCmd {
	#[clap(flatten)]
	pub base: cumulus_client_cli::RunCmd,

	/// Enable the development service to run without a backing relay chain
	#[clap(long)]
	pub dev_service: bool,

	/// When blocks should be sealed in the dev service.
	///
	/// Options are "instant", "manual", or timer interval in milliseconds
	#[clap(long, default_value = "instant")]
	pub sealing: Sealing,

	/// Public authoring identity to be inserted in the author inherent
	/// This is not currently used, but we may want a way to use it in the dev service.
	// #[clap(long)]
	// pub author_id: Option<NimbusId>,

	/// Enable EVM tracing module on a non-authority node.
	#[clap(
		long,
		conflicts_with = "collator",
		conflicts_with = "validator",
		value_delimiter = ','
	)]
	pub ethapi: Vec<EthApi>,

	/// Number of concurrent tracing tasks. Meant to be shared by both "debug" and "trace" modules.
	#[clap(long, default_value = "10")]
	pub ethapi_max_permits: u32,

	/// Maximum number of trace entries a single request of `trace_filter` is allowed to return.
	/// A request asking for more or an unbounded one going over this limit will both return an
	/// error.
	#[clap(long, default_value = "500")]
	pub ethapi_trace_max_count: u32,

	/// Duration (in seconds) after which the cache of `trace_filter` for a given block will be
	/// discarded.
	#[clap(long, default_value = "300")]
	pub ethapi_trace_cache_duration: u64,

	/// Size in bytes of the LRU cache for block data.
	#[clap(long, default_value = "300000000")]
	pub eth_log_block_cache: usize,

	/// Size in bytes of the LRU cache for transactions statuses data.
	#[clap(long, default_value = "300000000")]
	pub eth_statuses_cache: usize,

	/// Size in bytes of data a raw tracing request is allowed to use.
	/// Bound the size of memory, stack and storage data.
	#[clap(long, default_value = "20000000")]
	pub tracing_raw_max_memory_usage: usize,

	/// Maximum number of logs in a query.
	#[clap(long, default_value = "10000")]
	pub max_past_logs: u32,

	/// Force using Moonbase native runtime.
	#[clap(long = "force-moonbase")]
	pub force_moonbase: bool,

	/// Force using Moonriver native runtime.
	#[clap(long = "force-moonriver")]
	pub force_moonriver: bool,

	/// Id of the parachain this collator collates for.
	#[clap(long)]
	pub parachain_id: Option<u32>,

	/// Maximum fee history cache size.
	#[clap(long, default_value = "2048")]
	pub fee_history_limit: u64,

	/// Disable automatic hardware benchmarks.
	///
	/// By default these benchmarks are automatically ran at startup and measure
	/// the CPU speed, the memory bandwidth and the disk speed.
	///
	/// The results are then printed out in the logs, and also sent as part of
	/// telemetry, if telemetry is enabled.
	#[clap(long)]
	pub no_hardware_benchmarks: bool,
}

impl std::ops::Deref for RunCmd {
	type Target = cumulus_client_cli::RunCmd;

	fn deref(&self) -> &Self::Target {
		&self.base
	}
}

#[derive(Debug, clap::Subcommand)]
pub enum KeyCmd {
	#[clap(flatten)]
	BaseCli(sc_cli::KeySubcommand),
	/// Generate an Ethereum account.
	GenerateAccountKey(GenerateAccountKey),
}

impl KeyCmd {
	/// run the key subcommands
	pub fn run<C: SubstrateCli>(&self, cli: &C) -> Result<(), CliError> {
		match self {
			KeyCmd::BaseCli(cmd) => cmd.run(cli),
			KeyCmd::GenerateAccountKey(cmd) => {
				cmd.run();
				Ok(())
			}
		}
	}
}

#[derive(Debug, Parser)]
#[clap(
	propagate_version = true,
	args_conflicts_with_subcommands = true,
	subcommand_negates_reqs = true
)]
pub struct Cli {
	#[clap(subcommand)]
	pub subcommand: Option<Subcommand>,

	#[clap(flatten)]
	pub run: RunCmd,

	/// Relaychain arguments
	#[clap(raw = true)]
	pub relaychain_args: Vec<String>,
}

#[derive(Debug)]
pub struct RelayChainCli {
	/// The actual relay chain cli object.
	pub base: polkadot_cli::RunCmd,

	/// Optional chain id that should be passed to the relay chain.
	pub chain_id: Option<String>,

	/// The base path that should be used by the relay chain.
	pub base_path: Option<PathBuf>,
}

impl RelayChainCli {
	/// Parse the relay chain CLI parameters using the para chain `Configuration`.
	pub fn new<'a>(
		para_config: &sc_service::Configuration,
		relay_chain_args: impl Iterator<Item = &'a String>,
	) -> Self {
		let extension = chain_spec::Extensions::try_get(&*para_config.chain_spec);
		let chain_id = extension.map(|e| e.relay_chain.clone());
		let base_path = para_config
			.base_path
			.as_ref()
			.map(|x| x.path().join("polkadot"));
		Self {
			base_path,
			chain_id,
			base: polkadot_cli::RunCmd::parse_from(relay_chain_args),
		}
	}
}
