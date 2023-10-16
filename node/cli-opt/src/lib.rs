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
use std::str::FromStr;

pub mod account_key;

/// Block authoring scheme to be used by the dev service.
#[derive(Debug, Copy, Clone)]
pub enum Sealing {
	/// Author a block immediately upon receiving a transaction into the transaction pool
	Instant,
	/// Author a block upon receiving an RPC command
	Manual,
	/// Author blocks at a regular interval specified in milliseconds
	Interval(u64),
}

impl FromStr for Sealing {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"instant" => Self::Instant,
			"manual" => Self::Manual,
			s => {
				let millis =
					u64::from_str_radix(s, 10).map_err(|_| "couldn't decode sealing param")?;
				Self::Interval(millis)
			}
		})
	}
}

#[derive(Debug, PartialEq, Clone)]
pub enum EthApi {
	Txpool,
	Debug,
	Trace,
}

impl FromStr for EthApi {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"txpool" => Self::Txpool,
			"debug" => Self::Debug,
			"trace" => Self::Trace,
			_ => {
				return Err(format!(
					"`{}` is not recognized as a supported Ethereum Api",
					s
				))
			}
		})
	}
}

/// Available frontier backend types.
#[derive(Debug, Copy, Clone, Default, clap::ValueEnum)]
pub enum FrontierBackendType {
	/// Either RocksDb or ParityDb as per inherited from the global backend settings.
	#[default]
	KeyValue,
	/// Sql database with custom log indexing.
	Sql,
}

/// Defines the frontier backend configuration.
pub enum FrontierBackendConfig {
	KeyValue,
	Sql {
		pool_size: u32,
		num_ops_timeout: u32,
		thread_count: u32,
		cache_size: u64,
	},
}

impl Default for FrontierBackendConfig {
	fn default() -> FrontierBackendConfig {
		FrontierBackendConfig::KeyValue
	}
}

pub struct RpcConfig {
	pub ethapi: Vec<EthApi>,
	pub ethapi_max_permits: u32,
	pub ethapi_trace_max_count: u32,
	pub ethapi_trace_cache_duration: u64,
	pub eth_log_block_cache: usize,
	pub eth_statuses_cache: usize,
	pub fee_history_limit: u64,
	pub max_past_logs: u32,
	pub relay_chain_rpc_urls: Vec<url::Url>,
	pub tracing_raw_max_memory_usage: usize,
	pub frontier_backend_config: FrontierBackendConfig,
	pub no_prometheus_prefix: bool,
}
