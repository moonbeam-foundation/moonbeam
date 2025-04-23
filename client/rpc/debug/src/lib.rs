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
use futures::StreamExt;
use jsonrpsee::core::{async_trait, RpcResult};
pub use moonbeam_rpc_core_debug::{DebugServer, TraceCallParams, TraceParams};

use tokio::{
	self,
	sync::{oneshot, Semaphore},
};

use ethereum_types::H256;
use fc_rpc::{frontier_backend_client, internal_err};
use fc_storage::StorageOverride;
use fp_rpc::EthereumRuntimeRPCApi;
use moonbeam_client_evm_tracing::types::block;
use moonbeam_client_evm_tracing::types::block::BlockTransactionTrace;
use moonbeam_client_evm_tracing::{formatters::ResponseFormatter, types::single};
use moonbeam_rpc_core_types::{RequestBlockId, RequestBlockTag};
use moonbeam_rpc_primitives_debug::{DebugRuntimeApi, TracerInput};
use sc_client_api::backend::{Backend, StateBackend, StorageProvider};
use sc_utils::mpsc::TracingUnboundedSender;
use sp_api::{ApiExt, Core, ProvideRuntimeApi};
use sp_block_builder::BlockBuilder;
use sp_blockchain::{
	Backend as BlockchainBackend, Error as BlockChainError, HeaderBackend, HeaderMetadata,
};
use sp_runtime::{
	generic::BlockId,
	traits::{BlakeTwo256, Block as BlockT, Header as HeaderT, UniqueSaturatedInto},
};
use std::collections::BTreeMap;
use std::{future::Future, marker::PhantomData, sync::Arc};

pub enum RequesterInput {
	Call((RequestBlockId, TraceCallParams)),
	Transaction(H256),
	Block(RequestBlockId),
}

pub enum Response {
	Single(single::TransactionTrace),
	Block(Vec<block::BlockTransactionTrace>),
}

pub type Responder = oneshot::Sender<RpcResult<Response>>;
pub type DebugRequester =
	TracingUnboundedSender<((RequesterInput, Option<TraceParams>), Responder)>;

pub struct Debug {
	pub requester: DebugRequester,
}

impl Debug {
	pub fn new(requester: DebugRequester) -> Self {
		Self { requester }
	}
}

#[async_trait]
impl DebugServer for Debug {
	/// Handler for `debug_traceTransaction` request. Communicates with the service-defined task
	/// using channels.
	async fn trace_transaction(
		&self,
		transaction_hash: H256,
		params: Option<TraceParams>,
	) -> RpcResult<single::TransactionTrace> {
		let requester = self.requester.clone();

		let (tx, rx) = oneshot::channel();
		// Send a message from the rpc handler to the service level task.
		requester
			.unbounded_send(((RequesterInput::Transaction(transaction_hash), params), tx))
			.map_err(|err| {
				internal_err(format!(
					"failed to send request to debug service : {:?}",
					err
				))
			})?;

		// Receive a message from the service level task and send the rpc response.
		rx.await
			.map_err(|err| internal_err(format!("debug service dropped the channel : {:?}", err)))?
			.map(|res| match res {
				Response::Single(res) => res,
				_ => unreachable!(),
			})
	}

	async fn trace_block(
		&self,
		id: RequestBlockId,
		params: Option<TraceParams>,
	) -> RpcResult<Vec<BlockTransactionTrace>> {
		let requester = self.requester.clone();

		let (tx, rx) = oneshot::channel();
		// Send a message from the rpc handler to the service level task.
		requester
			.unbounded_send(((RequesterInput::Block(id), params), tx))
			.map_err(|err| {
				internal_err(format!(
					"failed to send request to debug service : {:?}",
					err
				))
			})?;

		// Receive a message from the service level task and send the rpc response.
		rx.await
			.map_err(|err| internal_err(format!("debug service dropped the channel : {:?}", err)))?
			.map(|res| match res {
				Response::Block(res) => res,
				_ => unreachable!(),
			})
	}

	/// Handler for `debug_traceCall` request. Communicates with the service-defined task
	/// using channels.
	async fn trace_call(
		&self,
		call_params: TraceCallParams,
		id: RequestBlockId,
		params: Option<TraceParams>,
	) -> RpcResult<single::TransactionTrace> {
		let requester = self.requester.clone();

		let (tx, rx) = oneshot::channel();
		// Send a message from the rpc handler to the service level task.
		requester
			.unbounded_send(((RequesterInput::Call((id, call_params)), params), tx))
			.map_err(|err| {
				internal_err(format!(
					"failed to send request to debug service : {:?}",
					err
				))
			})?;

		// Receive a message from the service level task and send the rpc response.
		rx.await
			.map_err(|err| internal_err(format!("debug service dropped the channel : {:?}", err)))?
			.map(|res| match res {
				Response::Single(res) => res,
				_ => unreachable!(),
			})
	}
}

pub struct DebugHandler<B: BlockT, C, BE>(PhantomData<(B, C, BE)>);

impl<B, C, BE> DebugHandler<B, C, BE>
where
	BE: Backend<B> + 'static,
	BE::State: StateBackend<BlakeTwo256>,
	C: ProvideRuntimeApi<B>,
	C: StorageProvider<B, BE>,
	C: HeaderMetadata<B, Error = BlockChainError> + HeaderBackend<B>,
	C: Send + Sync + 'static,
	B: BlockT<Hash = H256> + Send + Sync + 'static,
	C::Api: BlockBuilder<B>,
	C::Api: DebugRuntimeApi<B>,
	C::Api: EthereumRuntimeRPCApi<B>,
	C::Api: ApiExt<B>,
{
	/// Task spawned at service level that listens for messages on the rpc channel and spawns
	/// blocking tasks using a permit pool.
	pub fn task(
		client: Arc<C>,
		backend: Arc<BE>,
		frontier_backend: Arc<dyn fc_api::Backend<B> + Send + Sync>,
		permit_pool: Arc<Semaphore>,
		overrides: Arc<dyn StorageOverride<B>>,
		raw_max_memory_usage: usize,
	) -> (impl Future<Output = ()>, DebugRequester) {
		let (tx, mut rx): (DebugRequester, _) =
			sc_utils::mpsc::tracing_unbounded("debug-requester", 100_000);

		let fut = async move {
			loop {
				match rx.next().await {
					Some((
						(RequesterInput::Transaction(transaction_hash), params),
						response_tx,
					)) => {
						let client = client.clone();
						let backend = backend.clone();
						let frontier_backend = frontier_backend.clone();
						let permit_pool = permit_pool.clone();
						let overrides = overrides.clone();

						tokio::task::spawn(async move {
							let _ = response_tx.send(
								async {
									let _permit = permit_pool.acquire().await;
									tokio::task::spawn_blocking(move || {
										Self::handle_transaction_request(
											client.clone(),
											backend.clone(),
											frontier_backend.clone(),
											transaction_hash,
											params,
											overrides.clone(),
											raw_max_memory_usage,
										)
									})
									.await
									.map_err(|e| {
										internal_err(format!(
											"Internal error on spawned task : {:?}",
											e
										))
									})?
								}
								.await,
							);
						});
					}
					Some((
						(RequesterInput::Call((request_block_id, call_params)), params),
						response_tx,
					)) => {
						let client = client.clone();
						let frontier_backend = frontier_backend.clone();
						let permit_pool = permit_pool.clone();

						tokio::task::spawn(async move {
							let _ = response_tx.send(
								async {
									let _permit = permit_pool.acquire().await;
									tokio::task::spawn_blocking(move || {
										Self::handle_call_request(
											client.clone(),
											frontier_backend.clone(),
											request_block_id,
											call_params,
											params,
											raw_max_memory_usage,
										)
									})
									.await
									.map_err(|e| {
										internal_err(format!(
											"Internal error on spawned task : {:?}",
											e
										))
									})?
								}
								.await,
							);
						});
					}
					Some(((RequesterInput::Block(request_block_id), params), response_tx)) => {
						let client = client.clone();
						let backend = backend.clone();
						let frontier_backend = frontier_backend.clone();
						let permit_pool = permit_pool.clone();
						let overrides = overrides.clone();

						tokio::task::spawn(async move {
							let _ = response_tx.send(
								async {
									let _permit = permit_pool.acquire().await;

									tokio::task::spawn_blocking(move || {
										Self::handle_block_request(
											client.clone(),
											backend.clone(),
											frontier_backend.clone(),
											request_block_id,
											params,
											overrides.clone(),
										)
									})
									.await
									.map_err(|e| {
										internal_err(format!(
											"Internal error on spawned task : {:?}",
											e
										))
									})?
								}
								.await,
							);
						});
					}
					_ => {}
				}
			}
		};
		(fut, tx)
	}

	fn handle_params(
		params: Option<TraceParams>,
	) -> RpcResult<(
		TracerInput,
		single::TraceType,
		Option<single::TraceCallConfig>,
	)> {
		// Set trace input and type
		match params {
			Some(TraceParams {
				tracer: Some(tracer),
				tracer_config,
				..
			}) => {
				const BLOCKSCOUT_JS_CODE_HASH: [u8; 16] =
					hex_literal::hex!("94d9f08796f91eb13a2e82a6066882f7");
				const BLOCKSCOUT_JS_CODE_HASH_V2: [u8; 16] =
					hex_literal::hex!("89db13694675692951673a1e6e18ff02");
				let hash = sp_io::hashing::twox_128(&tracer.as_bytes());
				let tracer =
					if hash == BLOCKSCOUT_JS_CODE_HASH || hash == BLOCKSCOUT_JS_CODE_HASH_V2 {
						Some(TracerInput::Blockscout)
					} else if tracer == "callTracer" {
						Some(TracerInput::CallTracer)
					} else {
						None
					};
				if let Some(tracer) = tracer {
					Ok((tracer, single::TraceType::CallList, tracer_config))
				} else {
					return Err(internal_err(format!(
						"javascript based tracing is not available (hash :{:?})",
						hash
					)));
				}
			}
			Some(params) => Ok((
				TracerInput::None,
				single::TraceType::Raw {
					disable_storage: params.disable_storage.unwrap_or(false),
					disable_memory: params.disable_memory.unwrap_or(false),
					disable_stack: params.disable_stack.unwrap_or(false),
				},
				params.tracer_config,
			)),
			_ => Ok((
				TracerInput::None,
				single::TraceType::Raw {
					disable_storage: false,
					disable_memory: false,
					disable_stack: false,
				},
				None,
			)),
		}
	}

	fn handle_block_request(
		client: Arc<C>,
		backend: Arc<BE>,
		frontier_backend: Arc<dyn fc_api::Backend<B> + Send + Sync>,
		request_block_id: RequestBlockId,
		params: Option<TraceParams>,
		overrides: Arc<dyn StorageOverride<B>>,
	) -> RpcResult<Response> {
		let (tracer_input, trace_type, tracer_config) = Self::handle_params(params)?;

		let reference_id: BlockId<B> = match request_block_id {
			RequestBlockId::Number(n) => Ok(BlockId::Number(n.unique_saturated_into())),
			RequestBlockId::Tag(RequestBlockTag::Latest) => {
				Ok(BlockId::Number(client.info().best_number))
			}
			RequestBlockId::Tag(RequestBlockTag::Earliest) => {
				Ok(BlockId::Number(0u32.unique_saturated_into()))
			}
			RequestBlockId::Tag(RequestBlockTag::Pending) => {
				Err(internal_err("'pending' blocks are not supported"))
			}
			RequestBlockId::Hash(eth_hash) => {
				match futures::executor::block_on(frontier_backend_client::load_hash::<B, C>(
					client.as_ref(),
					frontier_backend.as_ref(),
					eth_hash,
				)) {
					Ok(Some(hash)) => Ok(BlockId::Hash(hash)),
					Ok(_) => Err(internal_err("Block hash not found".to_string())),
					Err(e) => Err(e),
				}
			}
		}?;

		// Get ApiRef. This handle allows to keep changes between txs in an internal buffer.
		let mut api = client.runtime_api();

		// Enable proof recording
		api.record_proof();
		api.proof_recorder().map(|recorder| {
			let ext = sp_trie::proof_size_extension::ProofSizeExt::new(recorder);
			api.register_extension(ext);
		});

		// Get Blockchain backend
		let blockchain = backend.blockchain();
		// Get the header I want to work with.
		let Ok(hash) = client.expect_block_hash_from_id(&reference_id) else {
			return Err(internal_err("Block header not found"));
		};
		let header = match client.header(hash) {
			Ok(Some(h)) => h,
			_ => return Err(internal_err("Block header not found")),
		};

		// Get parent blockid.
		let parent_block_hash = *header.parent_hash();

		let statuses = overrides
			.current_transaction_statuses(hash)
			.unwrap_or_default();

		// Known ethereum transaction hashes.
		let eth_transactions_by_index: BTreeMap<u32, H256> = statuses
			.iter()
			.map(|t| (t.transaction_index, t.transaction_hash))
			.collect();

		let eth_tx_hashes: Vec<_> = eth_transactions_by_index.values().cloned().collect();

		// If there are no ethereum transactions in the block return empty trace right away.
		if eth_tx_hashes.is_empty() {
			return Ok(Response::Block(vec![]));
		}

		// Get block extrinsics.
		let exts = blockchain
			.body(hash)
			.map_err(|e| internal_err(format!("Fail to read blockchain db: {:?}", e)))?
			.unwrap_or_default();

		// Get DebugRuntimeApi version
		let trace_api_version = if let Ok(Some(api_version)) =
			api.api_version::<dyn DebugRuntimeApi<B>>(parent_block_hash)
		{
			api_version
		} else {
			return Err(internal_err(
				"Runtime api version call failed (trace)".to_string(),
			));
		};

		// Trace the block.
		let f = || -> RpcResult<_> {
			let result = if trace_api_version >= 5 {
				// The block is initialized inside "trace_block"
				api.trace_block(parent_block_hash, exts, eth_tx_hashes, &header)
			} else {
				// Get core runtime api version
				let core_api_version = if let Ok(Some(api_version)) =
					api.api_version::<dyn Core<B>>(parent_block_hash)
				{
					api_version
				} else {
					return Err(internal_err(
						"Runtime api version call failed (core)".to_string(),
					));
				};

				// Initialize block: calls the "on_initialize" hook on every pallet
				// in AllPalletsWithSystem
				// This was fine before pallet-message-queue because the XCM messages
				// were processed by the "setValidationData" inherent call and not on an
				// "on_initialize" hook, which runs before enabling XCM tracing
				if core_api_version >= 5 {
					api.initialize_block(parent_block_hash, &header)
						.map_err(|e| internal_err(format!("Runtime api access error: {:?}", e)))?;
				} else {
					#[allow(deprecated)]
					api.initialize_block_before_version_5(parent_block_hash, &header)
						.map_err(|e| internal_err(format!("Runtime api access error: {:?}", e)))?;
				}

				#[allow(deprecated)]
				api.trace_block_before_version_5(parent_block_hash, exts, eth_tx_hashes)
			};

			result
				.map_err(|e| {
					internal_err(format!(
						"Blockchain error when replaying block {} : {:?}",
						reference_id, e
					))
				})?
				.map_err(|e| {
					internal_err(format!(
						"Internal runtime error when replaying block {} : {:?}",
						reference_id, e
					))
				})?;

			Ok(moonbeam_rpc_primitives_debug::Response::Block)
		};

		// Offset to account for old buggy transactions that are in trace not in the ethereum block
		let mut tx_position_offset = 0;

		return match trace_type {
			single::TraceType::CallList => {
				let mut proxy = moonbeam_client_evm_tracing::listeners::CallList::default();
				proxy.with_log = tracer_config.map_or(false, |cfg| cfg.with_log);
				proxy.using(f)?;
				proxy.finish_transaction();
				let response = match tracer_input {
					TracerInput::CallTracer => {
						let result =
							moonbeam_client_evm_tracing::formatters::CallTracer::format(proxy)
								.ok_or("Trace result is empty.")
								.map_err(|e| internal_err(format!("{:?}", e)))?
								.into_iter()
								.filter_map(|mut trace| {
									if let Some(transaction_hash) = eth_transactions_by_index
										.get(&(trace.tx_position - tx_position_offset))
									{
										trace.tx_hash = *transaction_hash;
										Some(trace)
									} else {
										// If the transaction is not in the ethereum block
										// it should not appear in the block trace
										tx_position_offset += 1;
										None
									}
								})
								.collect::<Vec<BlockTransactionTrace>>();

						let n_txs = eth_transactions_by_index.len();
						let n_traces = result.len();
						if n_txs != n_traces {
							log::warn!(
								"The traces in block {:?} don't match with the number of ethereum transactions. (txs: {}, traces: {})",
								request_block_id,
								n_txs,
								n_traces
							);
						}

						Ok(result)
					}
					_ => Err(internal_err(
						"Bug: failed to resolve the tracer format.".to_string(),
					)),
				}?;

				Ok(Response::Block(response))
			}
			_ => Err(internal_err(
				"debug_traceBlock functions currently only support callList mode (enabled
				by providing `{{'tracer': 'callTracer'}}` in the request)."
					.to_string(),
			)),
		};
	}

	/// Replays a transaction in the Runtime at a given block height.
	///
	/// In order to successfully reproduce the result of the original transaction we need a correct
	/// state to replay over.
	///
	/// Substrate allows to apply extrinsics in the Runtime and thus creating an overlayed state.
	/// These overlayed changes will live in-memory for the lifetime of the ApiRef.
	fn handle_transaction_request(
		client: Arc<C>,
		backend: Arc<BE>,
		frontier_backend: Arc<dyn fc_api::Backend<B> + Send + Sync>,
		transaction_hash: H256,
		params: Option<TraceParams>,
		overrides: Arc<dyn StorageOverride<B>>,
		raw_max_memory_usage: usize,
	) -> RpcResult<Response> {
		let (tracer_input, trace_type, tracer_config) = Self::handle_params(params)?;

		let (hash, index) =
			match futures::executor::block_on(frontier_backend_client::load_transactions::<B, C>(
				client.as_ref(),
				frontier_backend.as_ref(),
				transaction_hash,
				false,
			)) {
				Ok(Some((hash, index))) => (hash, index as usize),
				Ok(None) => return Err(internal_err("Transaction hash not found".to_string())),
				Err(e) => return Err(e),
			};

		let reference_id =
			match futures::executor::block_on(frontier_backend_client::load_hash::<B, C>(
				client.as_ref(),
				frontier_backend.as_ref(),
				hash,
			)) {
				Ok(Some(hash)) => BlockId::Hash(hash),
				Ok(_) => return Err(internal_err("Block hash not found".to_string())),
				Err(e) => return Err(e),
			};
		// Get ApiRef. This handle allow to keep changes between txs in an internal buffer.
		let mut api = client.runtime_api();

		// Enable proof recording
		api.record_proof();
		api.proof_recorder().map(|recorder| {
			let ext = sp_trie::proof_size_extension::ProofSizeExt::new(recorder);
			api.register_extension(ext);
		});

		// Get Blockchain backend
		let blockchain = backend.blockchain();
		// Get the header I want to work with.
		let Ok(reference_hash) = client.expect_block_hash_from_id(&reference_id) else {
			return Err(internal_err("Block header not found"));
		};
		let header = match client.header(reference_hash) {
			Ok(Some(h)) => h,
			_ => return Err(internal_err("Block header not found")),
		};
		// Get parent blockid.
		let parent_block_hash = *header.parent_hash();

		// Get block extrinsics.
		let exts = blockchain
			.body(reference_hash)
			.map_err(|e| internal_err(format!("Fail to read blockchain db: {:?}", e)))?
			.unwrap_or_default();

		// Get DebugRuntimeApi version
		let trace_api_version = if let Ok(Some(api_version)) =
			api.api_version::<dyn DebugRuntimeApi<B>>(parent_block_hash)
		{
			api_version
		} else {
			return Err(internal_err(
				"Runtime api version call failed (trace)".to_string(),
			));
		};

		let reference_block = overrides.current_block(reference_hash);

		// Get the actual ethereum transaction.
		if let Some(block) = reference_block {
			let transactions = block.transactions;
			if let Some(transaction) = transactions.get(index) {
				let f = || -> RpcResult<_> {
					let result = if trace_api_version >= 5 {
						// The block is initialized inside "trace_transaction"
						api.trace_transaction(parent_block_hash, exts, &transaction, &header)
					} else {
						// Get core runtime api version
						let core_api_version = if let Ok(Some(api_version)) =
							api.api_version::<dyn Core<B>>(parent_block_hash)
						{
							api_version
						} else {
							return Err(internal_err(
								"Runtime api version call failed (core)".to_string(),
							));
						};

						// Initialize block: calls the "on_initialize" hook on every pallet
						// in AllPalletsWithSystem
						// This was fine before pallet-message-queue because the XCM messages
						// were processed by the "setValidationData" inherent call and not on an
						// "on_initialize" hook, which runs before enabling XCM tracing
						if core_api_version >= 5 {
							api.initialize_block(parent_block_hash, &header)
								.map_err(|e| {
									internal_err(format!("Runtime api access error: {:?}", e))
								})?;
						} else {
							#[allow(deprecated)]
							api.initialize_block_before_version_5(parent_block_hash, &header)
								.map_err(|e| {
									internal_err(format!("Runtime api access error: {:?}", e))
								})?;
						}

						if trace_api_version == 4 {
							// Pre pallet-message-queue
							#[allow(deprecated)]
							api.trace_transaction_before_version_5(
								parent_block_hash,
								exts,
								&transaction,
							)
						} else {
							// Pre-london update, legacy transactions.
							match transaction {
								ethereum::TransactionV2::Legacy(tx) =>
								{
									#[allow(deprecated)]
									api.trace_transaction_before_version_4(
										parent_block_hash,
										exts,
										&tx,
									)
								}
								_ => {
									return Err(internal_err(
										"Bug: pre-london runtime expects legacy transactions"
											.to_string(),
									))
								}
							}
						}
					};

					result
						.map_err(|e| {
							internal_err(format!(
								"Runtime api access error (version {:?}): {:?}",
								trace_api_version, e
							))
						})?
						.map_err(|e| internal_err(format!("DispatchError: {:?}", e)))?;

					Ok(moonbeam_rpc_primitives_debug::Response::Single)
				};

				return match trace_type {
					single::TraceType::Raw {
						disable_storage,
						disable_memory,
						disable_stack,
					} => {
						let mut proxy = moonbeam_client_evm_tracing::listeners::Raw::new(
							disable_storage,
							disable_memory,
							disable_stack,
							raw_max_memory_usage,
						);
						proxy.using(f)?;
						Ok(Response::Single(
							moonbeam_client_evm_tracing::formatters::Raw::format(proxy).ok_or(
								internal_err(
									"replayed transaction generated too much data. \
								try disabling memory or storage?",
								),
							)?,
						))
					}
					single::TraceType::CallList => {
						let mut proxy = moonbeam_client_evm_tracing::listeners::CallList::default();
						proxy.with_log = tracer_config.map_or(false, |cfg| cfg.with_log);
						proxy.using(f)?;
						proxy.finish_transaction();
						let response = match tracer_input {
							TracerInput::Blockscout => {
								moonbeam_client_evm_tracing::formatters::Blockscout::format(proxy)
									.ok_or("Trace result is empty.")
									.map_err(|e| internal_err(format!("{:?}", e)))
							}
							TracerInput::CallTracer => {
								let mut res =
									moonbeam_client_evm_tracing::formatters::CallTracer::format(
										proxy,
									)
									.ok_or("Trace result is empty.")
									.map_err(|e| internal_err(format!("{:?}", e)))?;
								Ok(res.pop().expect("Trace result is empty.").result)
							}
							_ => Err(internal_err(
								"Bug: failed to resolve the tracer format.".to_string(),
							)),
						}?;
						Ok(Response::Single(response))
					}
					not_supported => Err(internal_err(format!(
						"Bug: `handle_transaction_request` does not support {:?}.",
						not_supported
					))),
				};
			}
		}
		Err(internal_err("Runtime block call failed".to_string()))
	}

	fn handle_call_request(
		client: Arc<C>,
		frontier_backend: Arc<dyn fc_api::Backend<B> + Send + Sync>,
		request_block_id: RequestBlockId,
		call_params: TraceCallParams,
		trace_params: Option<TraceParams>,
		raw_max_memory_usage: usize,
	) -> RpcResult<Response> {
		let (tracer_input, trace_type, tracer_config) = Self::handle_params(trace_params)?;

		let reference_id: BlockId<B> = match request_block_id {
			RequestBlockId::Number(n) => Ok(BlockId::Number(n.unique_saturated_into())),
			RequestBlockId::Tag(RequestBlockTag::Latest) => {
				Ok(BlockId::Number(client.info().best_number))
			}
			RequestBlockId::Tag(RequestBlockTag::Earliest) => {
				Ok(BlockId::Number(0u32.unique_saturated_into()))
			}
			RequestBlockId::Tag(RequestBlockTag::Pending) => {
				Err(internal_err("'pending' blocks are not supported"))
			}
			RequestBlockId::Hash(eth_hash) => {
				match futures::executor::block_on(frontier_backend_client::load_hash::<B, C>(
					client.as_ref(),
					frontier_backend.as_ref(),
					eth_hash,
				)) {
					Ok(Some(hash)) => Ok(BlockId::Hash(hash)),
					Ok(_) => Err(internal_err("Block hash not found".to_string())),
					Err(e) => Err(e),
				}
			}
		}?;

		// Get ApiRef. This handle allow to keep changes between txs in an internal buffer.
		let mut api = client.runtime_api();

		// Enable proof recording
		api.record_proof();
		api.proof_recorder().map(|recorder| {
			let ext = sp_trie::proof_size_extension::ProofSizeExt::new(recorder);
			api.register_extension(ext);
		});

		// Get the header I want to work with.
		let Ok(hash) = client.expect_block_hash_from_id(&reference_id) else {
			return Err(internal_err("Block header not found"));
		};
		let header = match client.header(hash) {
			Ok(Some(h)) => h,
			_ => return Err(internal_err("Block header not found")),
		};
		// Get parent blockid.
		let parent_block_hash = *header.parent_hash();

		// Get DebugRuntimeApi version
		let trace_api_version = if let Ok(Some(api_version)) =
			api.api_version::<dyn DebugRuntimeApi<B>>(parent_block_hash)
		{
			api_version
		} else {
			return Err(internal_err(
				"Runtime api version call failed (trace)".to_string(),
			));
		};

		if trace_api_version <= 5 {
			return Err(internal_err(
				"debug_traceCall not supported with old runtimes".to_string(),
			));
		}

		let TraceCallParams {
			from,
			to,
			gas_price,
			max_fee_per_gas,
			max_priority_fee_per_gas,
			gas,
			value,
			data,
			nonce,
			access_list,
			..
		} = call_params;

		let (max_fee_per_gas, max_priority_fee_per_gas) =
			match (gas_price, max_fee_per_gas, max_priority_fee_per_gas) {
				(gas_price, None, None) => {
					// Legacy request, all default to gas price.
					// A zero-set gas price is None.
					let gas_price = if gas_price.unwrap_or_default().is_zero() {
						None
					} else {
						gas_price
					};
					(gas_price, gas_price)
				}
				(_, max_fee, max_priority) => {
					// eip-1559
					// A zero-set max fee is None.
					let max_fee = if max_fee.unwrap_or_default().is_zero() {
						None
					} else {
						max_fee
					};
					// Ensure `max_priority_fee_per_gas` is less or equal to `max_fee_per_gas`.
					if let Some(max_priority) = max_priority {
						let max_fee = max_fee.unwrap_or_default();
						if max_priority > max_fee {
							return Err(internal_err(
							"Invalid input: `max_priority_fee_per_gas` greater than `max_fee_per_gas`",
						));
						}
					}
					(max_fee, max_priority)
				}
			};

		let gas_limit = match gas {
			Some(amount) => amount,
			None => {
				if let Some(block) = api
					.current_block(parent_block_hash)
					.map_err(|err| internal_err(format!("runtime error: {:?}", err)))?
				{
					block.header.gas_limit
				} else {
					return Err(internal_err(
						"block unavailable, cannot query gas limit".to_string(),
					));
				}
			}
		};
		let data = data.map(|d| d.0).unwrap_or_default();

		let access_list = access_list.unwrap_or_default();

		let f = || -> RpcResult<_> {
			let _result = api
				.trace_call(
					parent_block_hash,
					&header,
					from.unwrap_or_default(),
					to,
					data,
					value.unwrap_or_default(),
					gas_limit,
					max_fee_per_gas,
					max_priority_fee_per_gas,
					nonce,
					Some(
						access_list
							.into_iter()
							.map(|item| (item.address, item.storage_keys))
							.collect(),
					),
				)
				.map_err(|e| internal_err(format!("Runtime api access error: {:?}", e)))?
				.map_err(|e| internal_err(format!("DispatchError: {:?}", e)))?;

			Ok(moonbeam_rpc_primitives_debug::Response::Single)
		};

		return match trace_type {
			single::TraceType::Raw {
				disable_storage,
				disable_memory,
				disable_stack,
			} => {
				let mut proxy = moonbeam_client_evm_tracing::listeners::Raw::new(
					disable_storage,
					disable_memory,
					disable_stack,
					raw_max_memory_usage,
				);
				proxy.using(f)?;
				Ok(Response::Single(
					moonbeam_client_evm_tracing::formatters::Raw::format(proxy).ok_or(
						internal_err(
							"replayed transaction generated too much data. \
						try disabling memory or storage?",
						),
					)?,
				))
			}
			single::TraceType::CallList => {
				let mut proxy = moonbeam_client_evm_tracing::listeners::CallList::default();
				proxy.with_log = tracer_config.map_or(false, |cfg| cfg.with_log);
				proxy.using(f)?;
				proxy.finish_transaction();
				let response = match tracer_input {
					TracerInput::Blockscout => {
						moonbeam_client_evm_tracing::formatters::Blockscout::format(proxy)
							.ok_or("Trace result is empty.")
							.map_err(|e| internal_err(format!("{:?}", e)))
					}
					TracerInput::CallTracer => {
						let mut res =
							moonbeam_client_evm_tracing::formatters::CallTracer::format(proxy)
								.ok_or("Trace result is empty.")
								.map_err(|e| internal_err(format!("{:?}", e)))?;
						Ok(res.pop().expect("Trace result is empty.").result)
					}
					_ => Err(internal_err(
						"Bug: failed to resolve the tracer format.".to_string(),
					)),
				}?;
				Ok(Response::Single(response))
			}
			not_supported => Err(internal_err(format!(
				"Bug: `handle_call_request` does not support {:?}.",
				not_supported
			))),
		};
	}
}
