// Copyright 2019-2021 PureStake Inc.
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

use super::*;

use moonbeam_rpc_debug::DebugHandler;
use moonbeam_rpc_debug::{Debug, DebugRequester, DebugServer};
use moonbeam_rpc_trace::{
	CacheRequester as TraceFilterCacheRequester, CacheTask, Trace, TraceServer,
};
use tokio::sync::Semaphore;

#[derive(Clone)]
pub struct RpcRequesters {
	pub debug: Option<DebugRequester>,
	pub trace: Option<TraceFilterCacheRequester>,
}

pub fn extend_with_tracing<C, BE>(
	client: Arc<C>,
	requesters: RpcRequesters,
	trace_filter_max_count: u32,
	io: &mut jsonrpc_core::IoHandler<sc_rpc::Metadata>,
) where
	BE: Backend<Block> + 'static,
	BE::State: StateBackend<BlakeTwo256>,
	BE::Blockchain: BlockchainBackend<Block>,
	C: ProvideRuntimeApi<Block> + StorageProvider<Block, BE> + AuxStore,
	C: BlockchainEvents<Block>,
	C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError> + 'static,
	C: Send + Sync + 'static,
	C::Api: RuntimeApiCollection<StateBackend = BE::State>,
{
	if let Some(trace_filter_requester) = requesters.trace {
		io.extend_with(TraceServer::to_delegate(Trace::new(
			client,
			trace_filter_requester,
			trace_filter_max_count,
		)));
	}

	if let Some(debug_requester) = requesters.debug {
		io.extend_with(DebugServer::to_delegate(Debug::new(debug_requester)));
	}
}

// Spawn the tasks that are required to run a Moonbeam tracing node.
pub fn spawn_tracing_tasks<B, C, BE>(
	rpc_config: &cli_opt::RpcConfig,
	params: SpawnTasksParams<B, C, BE>,
) -> RpcRequesters
where
	C: ProvideRuntimeApi<B> + BlockOf,
	C: HeaderBackend<B> + HeaderMetadata<B, Error = BlockChainError> + 'static,
	C: BlockchainEvents<B>,
	C: Send + Sync + 'static,
	C::Api: EthereumRuntimeRPCApi<B> + moonbeam_rpc_primitives_debug::DebugRuntimeApi<B>,
	C::Api: BlockBuilder<B>,
	B: BlockT<Hash = H256> + Send + Sync + 'static,
	B::Header: HeaderT<Number = u32>,
	BE: Backend<B> + 'static,
	BE::State: StateBackend<BlakeTwo256>,
{
	let permit_pool = Arc::new(Semaphore::new(rpc_config.ethapi_max_permits as usize));

	let (trace_filter_task, trace_filter_requester) =
		if rpc_config.ethapi.contains(&EthApiCmd::Trace) {
			let (trace_filter_task, trace_filter_requester) = CacheTask::create(
				Arc::clone(&params.client),
				Arc::clone(&params.substrate_backend),
				Duration::from_secs(rpc_config.ethapi_trace_cache_duration),
				Arc::clone(&permit_pool),
			);
			(Some(trace_filter_task), Some(trace_filter_requester))
		} else {
			(None, None)
		};

	let (debug_task, debug_requester) = if rpc_config.ethapi.contains(&EthApiCmd::Debug) {
		let (debug_task, debug_requester) = DebugHandler::task(
			Arc::clone(&params.client),
			Arc::clone(&params.substrate_backend),
			Arc::clone(&params.frontier_backend),
			Arc::clone(&permit_pool),
		);
		(Some(debug_task), Some(debug_requester))
	} else {
		(None, None)
	};

	// `trace_filter` cache task. Essential.
	// Proxies rpc requests to it's handler.
	if let Some(trace_filter_task) = trace_filter_task {
		params
			.task_manager
			.spawn_essential_handle()
			.spawn("trace-filter-cache", Some("EthTracing"), trace_filter_task);
	}

	// `debug` task if enabled. Essential.
	// Proxies rpc requests to it's handler.
	if let Some(debug_task) = debug_task {
		params
			.task_manager
			.spawn_essential_handle()
			.spawn("ethapi-debug", Some("EthTracing"), debug_task);
	}

	RpcRequesters {
		debug: debug_requester,
		trace: trace_filter_requester,
	}
}
