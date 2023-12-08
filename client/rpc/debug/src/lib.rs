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
use futures::StreamExt;
use jsonrpsee::core::{async_trait, RpcResult};
pub use moonbeam_rpc_core_debug::{DebugServer, TraceParams};

use tokio::{
	self,
	sync::{oneshot, Semaphore},
};

use ethereum_types::H256;
use fc_rpc::{frontier_backend_client, internal_err, OverrideHandle};
use fp_rpc::EthereumRuntimeRPCApi;
use moonbeam_client_evm_tracing::{formatters::ResponseFormatter, types::single};
use moonbeam_rpc_core_types::{RequestBlockId, RequestBlockTag};
use moonbeam_rpc_primitives_debug::{DebugRuntimeApi, TracerInput};
use sc_client_api::backend::{Backend, StateBackend, StorageProvider};
use sc_utils::mpsc::TracingUnboundedSender;
use sp_api::{ApiExt, BlockId, Core, HeaderT, ProvideRuntimeApi};
use sp_block_builder::BlockBuilder;
use sp_blockchain::{
	Backend as BlockchainBackend, Error as BlockChainError, HeaderBackend, HeaderMetadata,
};
use sp_runtime::traits::{BlakeTwo256, Block as BlockT, UniqueSaturatedInto};
use std::{future::Future, marker::PhantomData, sync::Arc};

pub enum RequesterInput {
	Transaction(H256),
	Block(RequestBlockId),
}

pub enum Response {
	Single(single::TransactionTrace),
	Block(Vec<single::TransactionTrace>),
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
	) -> RpcResult<Vec<single::TransactionTrace>> {
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
		overrides: Arc<OverrideHandle<B>>,
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

	fn handle_params(params: Option<TraceParams>) -> RpcResult<(TracerInput, single::TraceType)> {
		// Set trace input and type
		match params {
			Some(TraceParams {
				tracer: Some(tracer),
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
					Ok((tracer, single::TraceType::CallList))
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
			)),
			_ => Ok((
				TracerInput::None,
				single::TraceType::Raw {
					disable_storage: false,
					disable_memory: false,
					disable_stack: false,
				},
			)),
		}
	}

	fn handle_block_request(
		client: Arc<C>,
		backend: Arc<BE>,
		frontier_backend: Arc<dyn fc_api::Backend<B> + Send + Sync>,
		request_block_id: RequestBlockId,
		params: Option<TraceParams>,
		overrides: Arc<OverrideHandle<B>>,
	) -> RpcResult<Response> {
		let (tracer_input, trace_type) = Self::handle_params(params)?;

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
		let api = client.runtime_api();
		// Get Blockchain backend
		let blockchain = backend.blockchain();
		// Get the header I want to work with.
		let Ok(hash) = client.expect_block_hash_from_id(&reference_id) else {
			return Err(internal_err("Block header not found"))
		};
		let header = match client.header(hash) {
			Ok(Some(h)) => h,
			_ => return Err(internal_err("Block header not found")),
		};

		// Get parent blockid.
		let parent_block_hash = *header.parent_hash();

		let schema = fc_storage::onchain_storage_schema::<B, C, BE>(client.as_ref(), hash);

		// Using storage overrides we align with `:ethereum_schema` which will result in proper
		// SCALE decoding in case of migration.
		let statuses = match overrides.schemas.get(&schema) {
			Some(schema) => schema
				.current_transaction_statuses(hash)
				.unwrap_or_default(),
			_ => {
				return Err(internal_err(format!(
					"No storage override at {:?}",
					reference_id
				)))
			}
		};

		// Known ethereum transaction hashes.
		let eth_tx_hashes: Vec<_> = statuses.iter().map(|t| t.transaction_hash).collect();

		// If there are no ethereum transactions in the block return empty trace right away.
		if eth_tx_hashes.is_empty() {
			return Ok(Response::Block(vec![]));
		}

		// Get block extrinsics.
		let exts = blockchain
			.body(hash)
			.map_err(|e| internal_err(format!("Fail to read blockchain db: {:?}", e)))?
			.unwrap_or_default();

		// Trace the block.
		let f = || -> RpcResult<_> {
			api.initialize_block(parent_block_hash, &header)
				.map_err(|e| internal_err(format!("Runtime api access error: {:?}", e)))?;

			let _result = api
				.trace_block(parent_block_hash, exts, eth_tx_hashes)
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

		return match trace_type {
			single::TraceType::CallList => {
				let mut proxy = moonbeam_client_evm_tracing::listeners::CallList::default();
				proxy.using(f)?;
				proxy.finish_transaction();
				let response = match tracer_input {
					TracerInput::CallTracer => {
						moonbeam_client_evm_tracing::formatters::CallTracer::format(proxy)
							.ok_or("Trace result is empty.")
							.map_err(|e| internal_err(format!("{:?}", e)))
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
	/// In order to succesfully reproduce the result of the original transaction we need a correct
	/// state to replay over.
	///
	/// Substrate allows to apply extrinsics in the Runtime and thus creating an overlayed state.
	/// This overlayed changes will live in-memory for the lifetime of the ApiRef.
	fn handle_transaction_request(
		client: Arc<C>,
		backend: Arc<BE>,
		frontier_backend: Arc<dyn fc_api::Backend<B> + Send + Sync>,
		transaction_hash: H256,
		params: Option<TraceParams>,
		overrides: Arc<OverrideHandle<B>>,
		raw_max_memory_usage: usize,
	) -> RpcResult<Response> {
		let (tracer_input, trace_type) = Self::handle_params(params)?;

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
		let api = client.runtime_api();
		// Get Blockchain backend
		let blockchain = backend.blockchain();
		// Get the header I want to work with.
		let Ok(reference_hash) = client.expect_block_hash_from_id(&reference_id) else {
			return Err(internal_err("Block header not found"))
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

		let schema =
			fc_storage::onchain_storage_schema::<B, C, BE>(client.as_ref(), reference_hash);

		// Get the block that contains the requested transaction. Using storage overrides we align
		// with `:ethereum_schema` which will result in proper SCALE decoding in case of migration.
		let reference_block = match overrides.schemas.get(&schema) {
			Some(schema) => schema.current_block(reference_hash),
			_ => {
				return Err(internal_err(format!(
					"No storage override at {:?}",
					reference_hash
				)))
			}
		};

		// Get the actual ethereum transaction.
		if let Some(block) = reference_block {
			let transactions = block.transactions;
			if let Some(transaction) = transactions.get(index) {
				let f = || -> RpcResult<_> {
					api.initialize_block(parent_block_hash, &header)
						.map_err(|e| internal_err(format!("Runtime api access error: {:?}", e)))?;

					if trace_api_version >= 4 {
						let _result = api
							.trace_transaction(parent_block_hash, exts, &transaction)
							.map_err(|e| {
								internal_err(format!(
									"Runtime api access error (version {:?}): {:?}",
									trace_api_version, e
								))
							})?
							.map_err(|e| internal_err(format!("DispatchError: {:?}", e)))?;
					} else {
						// Pre-london update, legacy transactions.
						let _result = match transaction {
							ethereum::TransactionV2::Legacy(tx) =>
							{
								#[allow(deprecated)]
								api.trace_transaction_before_version_4(parent_block_hash, exts, &tx)
									.map_err(|e| {
										internal_err(format!(
											"Runtime api access error (legacy): {:?}",
											e
										))
									})?
									.map_err(|e| internal_err(format!("DispatchError: {:?}", e)))?
							}
							_ => {
								return Err(internal_err(
									"Bug: pre-london runtime expects legacy transactions"
										.to_string(),
								))
							}
						};
					}

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
								Ok(res.pop().expect("Trace result is empty."))
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
}
