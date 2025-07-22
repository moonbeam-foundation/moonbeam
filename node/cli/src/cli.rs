// Copyright 2019-2025 PureStake Inc.
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
use moonbeam_cli_opt::{account_key::GenerateAccountKey, EthApi, FrontierBackendType, Sealing};
use moonbeam_service::chain_spec;
use sc_cli::{Error as CliError, SubstrateCli};
use std::path::PathBuf;
use std::time::Duration;
use url::Url;

#[cfg(feature = "lazy-loading")]
fn parse_block_hash(s: &str) -> Result<sp_core::H256, String> {
	use std::str::FromStr;
	sp_core::H256::from_str(s).map_err(|err| err.to_string())
}

fn validate_url(arg: &str) -> Result<Url, String> {
	let url = Url::parse(arg).map_err(|e| e.to_string())?;

	let scheme = url.scheme();
	if scheme == "http" || scheme == "https" {
		Ok(url)
	} else {
		Err(format!("'{}' URL scheme not supported.", url.scheme()))
	}
}

/// Sub-commands supported by the collator.
#[derive(Debug, clap::Subcommand)]
pub enum Subcommand {
	/// Export the genesis state of the parachain.
	#[clap(name = "export-genesis-state")]
	ExportGenesisHead(cumulus_client_cli::ExportGenesisHeadCommand),

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
	TryRuntime,

	/// Key management cli utilities
	#[clap(subcommand)]
	Key(KeyCmd),

	/// Precompile the WASM runtime into native code
	PrecompileWasm(sc_cli::PrecompileWasmCmd),
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

	/// No-op
	/// Deprecated in: https://github.com/moonbeam-foundation/moonbeam/pull/3204
	#[clap(long)]
	pub experimental_block_import_strategy: bool,

	/// Enable the legacy block import strategy
	#[clap(long)]
	pub legacy_block_import_strategy: bool,

	/// Specifies the URL used to fetch chain data via RPC.
	///
	/// The URL should point to the RPC endpoint of the chain being forked.
	/// Ensure that the RPC has sufficient rate limits to handle the expected load.
	#[cfg(feature = "lazy-loading")]
	#[clap(long)]
	#[arg(
		long,
		value_parser = validate_url,
		alias = "fork-chain-from-rpc"
	)]
	pub lazy_loading_remote_rpc: Option<Url>,

	/// Optional parameter to specify the block hash for lazy loading.
	///
	/// This parameter allows the user to specify a block hash from which to start loading data.
	///
	/// If not provided, the latest block will be used.
	#[cfg(feature = "lazy-loading")]
	#[arg(
		long,
		value_name = "BLOCK",
		value_parser = parse_block_hash,
		alias = "block"
	)]
	pub lazy_loading_block: Option<sp_core::H256>,

	/// Optional parameter to specify state overrides during lazy loading.
	///
	/// This parameter allows the user to provide a path to a file containing state overrides.
	/// The file can contain any custom state modifications that should be applied.
	#[cfg(feature = "lazy-loading")]
	#[clap(
		long,
		value_name = "PATH",
		value_parser,
		alias = "fork-state-overrides"
	)]
	pub lazy_loading_state_overrides: Option<PathBuf>,

	/// Optional parameter to specify a runtime override when starting the lazy loading.
	///
	/// If not provided, it will fetch the runtime from the block being forked.
	#[cfg(feature = "lazy-loading")]
	#[clap(long, value_name = "PATH", value_parser, alias = "runtime-override")]
	pub lazy_loading_runtime_override: Option<PathBuf>,

	/// The delay (in milliseconds) between RPC requests when using lazy loading.
	///
	/// This parameter controls the amount of time (in milliseconds) to wait between consecutive
	/// RPC requests. This can help manage request rate and avoid overwhelming the server.
	///
	/// The default value is 100 milliseconds.
	#[cfg(feature = "lazy-loading")]
	#[clap(long, default_value = "100")]
	pub lazy_loading_delay_between_requests: u32,

	/// The maximum number of retries for an RPC request when using lazy loading.
	///
	/// The default value is 10 retries.
	#[cfg(feature = "lazy-loading")]
	#[clap(long, default_value = "10")]
	pub lazy_loading_max_retries_per_request: u32,

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
	#[clap(long, value_delimiter = ',')]
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

	/// Sets the frontier backend type (KeyValue or Sql)
	#[arg(long, value_enum, ignore_case = true, default_value_t = FrontierBackendType::default())]
	pub frontier_backend_type: FrontierBackendType,

	// Sets the SQL backend's pool size.
	#[arg(long, default_value = "100")]
	pub frontier_sql_backend_pool_size: u32,

	/// Sets the SQL backend's query timeout in number of VM ops.
	#[arg(long, default_value = "10000000")]
	pub frontier_sql_backend_num_ops_timeout: u32,

	/// Sets the SQL backend's auxiliary thread limit.
	#[arg(long, default_value = "4")]
	pub frontier_sql_backend_thread_count: u32,

	/// Sets the SQL backend's cache size in bytes.
	/// Default value is 200MB.
	#[arg(long, default_value = "209715200")]
	pub frontier_sql_backend_cache_size: u64,

	/// Size in bytes of data a raw tracing request is allowed to use.
	/// Bound the size of memory, stack and storage data.
	#[clap(long, default_value = "20000000")]
	pub tracing_raw_max_memory_usage: usize,

	/// Maximum number of logs in a query.
	#[clap(long, default_value = "10000")]
	pub max_past_logs: u32,

	/// Maximum block range to query logs from.
	#[clap(long, default_value = "1024")]
	pub max_block_range: u32,

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

	/// Removes moonbeam prefix from Prometheus metrics
	#[clap(long)]
	pub no_prometheus_prefix: bool,

	/// Maximum duration in milliseconds to produce a block
	#[clap(long, default_value = "2000", value_parser=block_authoring_duration_parser)]
	pub block_authoring_duration: Duration,

	/// Enable full proof-of-validation mode for Nimbus (deprecated, use --max-pov-percentage instead)
	#[clap(long, hide = true)]
	pub nimbus_full_pov: bool,

	/// Maximum percentage of POV size to use (0-100)
	#[arg(
		long,
		conflicts_with = "nimbus_full_pov",
		default_value = "50",
		default_value_if("nimbus_full_pov", "true", "100")
	)]
	pub max_pov_percentage: u8,
}

fn block_authoring_duration_parser(s: &str) -> Result<Duration, String> {
	Ok(Duration::from_millis(clap_num::number_range(
		s, 250, 2_000,
	)?))
}

impl RunCmd {
	pub fn new_rpc_config(&self) -> moonbeam_cli_opt::RpcConfig {
		moonbeam_cli_opt::RpcConfig {
			ethapi: self.ethapi.clone(),
			ethapi_max_permits: self.ethapi_max_permits,
			ethapi_trace_max_count: self.ethapi_trace_max_count,
			ethapi_trace_cache_duration: self.ethapi_trace_cache_duration,
			eth_log_block_cache: self.eth_log_block_cache,
			eth_statuses_cache: self.eth_statuses_cache,
			fee_history_limit: self.fee_history_limit,
			max_past_logs: self.max_past_logs,
			max_block_range: self.max_block_range,
			relay_chain_rpc_urls: self.base.relay_chain_rpc_urls.clone(),
			tracing_raw_max_memory_usage: self.tracing_raw_max_memory_usage,
			frontier_backend_config: match self.frontier_backend_type {
				FrontierBackendType::KeyValue => moonbeam_cli_opt::FrontierBackendConfig::KeyValue,
				FrontierBackendType::Sql => moonbeam_cli_opt::FrontierBackendConfig::Sql {
					pool_size: self.frontier_sql_backend_pool_size,
					num_ops_timeout: self.frontier_sql_backend_num_ops_timeout,
					thread_count: self.frontier_sql_backend_thread_count,
					cache_size: self.frontier_sql_backend_cache_size,
				},
			},
			no_prometheus_prefix: self.no_prometheus_prefix,
		}
	}
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
	pub base_path: PathBuf,
}

impl RelayChainCli {
	/// Parse the relay chain CLI parameters using the para chain `Configuration`.
	pub fn new<'a>(
		para_config: &sc_service::Configuration,
		relay_chain_args: impl Iterator<Item = &'a String>,
	) -> Self {
		let extension = chain_spec::Extensions::try_get(&*para_config.chain_spec);
		let chain_id = extension.map(|e| e.relay_chain.clone());
		let base_path = para_config.base_path.path().join("polkadot");
		Self {
			base_path,
			chain_id,
			base: polkadot_cli::RunCmd::parse_from(relay_chain_args),
		}
	}
}
