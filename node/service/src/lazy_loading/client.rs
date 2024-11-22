// Copyright 2024 Moonbeam foundation
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

use crate::lazy_loading;
use crate::lazy_loading::{TLazyLoadingBackend, TLazyLoadingCallExecutor};
use cumulus_primitives_core::BlockT;
use sc_chain_spec::BuildGenesisBlock;
use sc_client_api::execution_extensions::ExecutionExtensions;
use sc_client_api::{BadBlocks, ForkBlocks};
use sc_executor::RuntimeVersionOf;
use sc_service::client::Client;
use sc_service::{ClientConfig, LocalCallExecutor};
use sc_telemetry::TelemetryHandle;
use sp_core::traits::{CodeExecutor, SpawnNamed};
use std::sync::Arc;

pub fn new_client<E, Block, RA, G>(
	backend: Arc<TLazyLoadingBackend<Block>>,
	executor: E,
	genesis_block_builder: G,
	fork_blocks: ForkBlocks<Block>,
	bad_blocks: BadBlocks<Block>,
	execution_extensions: ExecutionExtensions<Block>,
	spawn_handle: Box<dyn SpawnNamed>,
	prometheus_registry: Option<substrate_prometheus_endpoint::Registry>,
	telemetry: Option<TelemetryHandle>,
	config: ClientConfig<Block>,
) -> Result<
	Client<TLazyLoadingBackend<Block>, TLazyLoadingCallExecutor<Block, E>, Block, RA>,
	sp_blockchain::Error,
>
where
	Block: BlockT + sp_runtime::DeserializeOwned,
	Block::Hash: From<sp_core::H256>,
	E: CodeExecutor + RuntimeVersionOf,
	TLazyLoadingBackend<Block>: sc_client_api::Backend<Block> + 'static,
	G: BuildGenesisBlock<
		Block,
		BlockImportOperation = <TLazyLoadingBackend<Block> as sc_client_api::backend::Backend<
			Block,
		>>::BlockImportOperation,
	>,
{
	let executor =
		lazy_loading::call_executor::LazyLoadingCallExecutor::new(LocalCallExecutor::new(
			backend.clone(),
			executor,
			config.clone(),
			execution_extensions,
		)?)?;

	Client::new(
		backend,
		executor,
		spawn_handle,
		genesis_block_builder,
		fork_blocks,
		bad_blocks,
		prometheus_registry,
		telemetry,
		config,
	)
}
