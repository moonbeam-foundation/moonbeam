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
use futures::{
	compat::Compat,
	future::{BoxFuture, TryFutureExt},
	FutureExt as _,
};
use jsonrpc_core::Result as RpcResult;
use jsonrpc_derive::rpc;

use polkadot_core_primitives::{
	BlockNumber as RelayBlockNumber, InboundDownwardMessage, InboundHrmpMessage,
};
use tokio::sync::oneshot;
//TODO This was in a separate crate in rpc-core for the ethereum-related RPC endpoints.
// But I'm starting with it all in one to keep things simple. Are there drawbacks to doing this?
/// This RPC interface is used to manually submit XCM messages that will be injected into a
/// parachain-enabled runtime. This allows testing XCM logic in a controlled way in integration
/// tests.
#[rpc(server)]
pub trait ManualXcmApi {
	// Inject a downward message - A message that comes from the relay chain.
	#[rpc(name = "xcm_injectDownwardMessage")]
	fn inject_downward_message(
		&self,
		sent_at: RelayBlockNumber,
		message: Vec<u8>,
	) -> BoxFuture<'static, RpcResult<bool>>;
	// For now we return bool which indicates some vague notion of success
	// In the future it may be nice to return which block hash this message was
	// injected into. We may need to learn more about how forks are handled to make
	// sure we don't lose/skip xcm messages in when reorgs happen. I'm not worried about that yet.

	// Inject an HRMP message - A message that comes from a dedicated channel to a sibling
	// parachain.
	#[rpc(name = "xcm_injectHrmpMessage")]
	fn inject_hrmp_message(
		&self,
		channel: u32, //TODO I think there is a better type for this?
		sent_at: RelayBlockNumber,
		message: Vec<u8>,
	) -> BoxFuture<'static, RpcResult<bool>>;
}

pub struct ManualXcm {
	pub downward_message_channel:
		flume::Sender<(InboundDownwardMessage, tokio::sync::oneshot::Sender<bool>)>,
	pub hrmp_message_channel:
		flume::Sender<(InboundHrmpMessage, tokio::sync::oneshot::Sender<bool>)>,
}

// impl Debug {
// 	pub fn new(requester: DebugRequester) -> Self {
// 		Self { requester }
// 	}
// }

impl ManualXcmApi for ManualXcm {
	fn inject_downward_message(
		&self,
		sent_at: RelayBlockNumber,
		msg: Vec<u8>,
	) -> BoxFuture<'static, RpcResult<bool>> {
		let downward_message_channel = self.downward_message_channel.clone();
		async move {
			let (tx, rx) = oneshot::channel();
			let message = InboundDownwardMessage { sent_at, msg };

			// Send the message back to the service where it will be queued up
			// to be injected in to an upcoming block. Also send a channel on which
			// the success message can be sent back.
			downward_message_channel
				.send_async((message, tx))
				.await
				.map_err(|err| internal_err(err))?;

			// Wait for the response that the messages was included successfully
			rx.await.map_err(|err| internal_err(err))
		}
		.boxed()
	}

	fn inject_hrmp_message(
		&self,
		channel: u32, //TODO I think there is a better type for this?
		sent_at: RelayBlockNumber,
		message: Vec<u8>,
	) -> BoxFuture<'static, RpcResult<bool>> {
		// let mut requester = self.requester.clone();

		println!("---> Enter");

		async move { todo!() }.boxed()
	}
}

// This bit cribbed from frontier.
pub fn internal_err<T: ToString>(message: T) -> jsonrpc_core::Error {
	jsonrpc_core::Error {
		code: jsonrpc_core::ErrorCode::InternalError,
		message: message.to_string(),
		data: None,
	}
}

// pub struct DebugHandler<B: BlockT, C, BE>(PhantomData<(B, C, BE)>);

// impl<B, C, BE> DebugHandler<B, C, BE>
// where
// 	BE: Backend<B> + 'static,
// 	C: ProvideRuntimeApi<B>,
// 	C: HeaderMetadata<B, Error = BlockChainError> + HeaderBackend<B>,
// 	C: Send + Sync + 'static,
// 	B: BlockT<Hash = H256> + Send + Sync + 'static,
// 	C::Api: BlockBuilder<B>,
// 	C::Api: DebugRuntimeApi<B>,
// 	C::Api: EthereumRuntimeRPCApi<B>,
// {
// 	/// Task spawned at service level that listens for messages on the rpc channel and spawns
// 	/// blocking tasks using a permit pool.
// 	pub fn task(
// 		client: Arc<C>,
// 		backend: Arc<BE>,
// 		frontier_backend: Arc<fc_db::Backend<B>>,
// 		permit_pool: Arc<Semaphore>,
// 	) -> (impl Future<Output = ()>, DebugRequester) {
// 		let (tx, mut rx): (DebugRequester, _) =
// 			sp_utils::mpsc::tracing_unbounded("debug-requester");

// 		let fut = async move {
// 			loop {
// 				match rx.next().await {
// 					Some((
// 						(RequesterInput::Transaction(transaction_hash), params),
// 						response_tx,
// 					)) => {
// 						let client = client.clone();
// 						let backend = backend.clone();
// 						let frontier_backend = frontier_backend.clone();
// 						let permit_pool = permit_pool.clone();
// 						// Note on spawned tasks https://tokio.rs/tokio/tutorial/spawning#tasks.
// 						//
// 						// Substrate uses the default value for `core_threads` (number of cores of the
// 						// machine running the node) and `max_threads` (512 total).
// 						//
// 						// Task below is spawned in the substrate's built tokio::Runtime, so they share
// 						// the same thread pool as the rest of the service-spawned tasks. Additionally,
// 						// blocking tasks use a more restrictive permit pool shared by trace modules.
// 						// https://docs.rs/tokio/0.2.23/tokio/sync/struct.Semaphore.html
// 						tokio::task::spawn(async move {
// 							let _ = response_tx.send(
// 								async {
// 									let _permit = permit_pool.acquire().await;
// 									tokio::task::spawn_blocking(move || {
// 										Self::handle_transaction_request(
// 											client.clone(),
// 											backend.clone(),
// 											frontier_backend.clone(),
// 											transaction_hash,
// 											params,
// 										)
// 									})
// 									.await
// 									.map_err(|e| {
// 										internal_err(format!(
// 											"Internal error on spawned task : {:?}",
// 											e
// 										))
// 									})?
// 								}
// 								.await,
// 							);
// 						});
// 					}
// 					Some(((RequesterInput::Block(request_block_id), params), response_tx)) => {
// 						let client = client.clone();
// 						let backend = backend.clone();
// 						let frontier_backend = frontier_backend.clone();
// 						let permit_pool = permit_pool.clone();
// 						// Note on spawned tasks https://tokio.rs/tokio/tutorial/spawning#tasks.
// 						//
// 						// Substrate uses the default value for `core_threads` (number of cores of the
// 						// machine running the node) and `max_threads` (512 total).
// 						//
// 						// Task below is spawned in the substrate's built tokio::Runtime, so they share
// 						// the same thread pool as the rest of the service-spawned tasks. Additionally,
// 						// blocking tasks use a more restrictive permit pool shared by trace modules.
// 						// https://docs.rs/tokio/0.2.23/tokio/sync/struct.Semaphore.html
// 						tokio::task::spawn(async move {
// 							println!("--> Request2");
// 							let _ = response_tx.send(
// 								async {
// 									let _permit = permit_pool.acquire().await;

// 									println!("--> Request3");
// 									tokio::task::spawn_blocking(move || {
// 										Self::handle_block_request(
// 											client.clone(),
// 											backend.clone(),
// 											frontier_backend.clone(),
// 											request_block_id,
// 											params,
// 										)
// 									})
// 									.await
// 									.map_err(|e| {
// 										internal_err(format!(
// 											"Internal error on spawned task : {:?}",
// 											e
// 										))
// 									})?
// 								}
// 								.await,
// 							);
// 						});
// 					}
// 					_ => {}
// 				}
// 			}
// 		};
// 		(fut, tx)
// 	}

// 	fn handle_params(params: Option<TraceParams>) -> RpcResult<(TracerInput, single::TraceType)> {
// 		// Set trace input and type
// 		match params {
// 			Some(TraceParams {
// 				tracer: Some(tracer),
// 				..
// 			}) => {
// 				let hash: H128 = sp_io::hashing::twox_128(&tracer.as_bytes()).into();
// 				let blockscout_hash = H128::from_str("0x94d9f08796f91eb13a2e82a6066882f7").unwrap();
// 				let tracer = if hash == blockscout_hash {
// 					Some(TracerInput::Blockscout)
// 				} else if tracer == "callTracer" {
// 					Some(TracerInput::CallTracer)
// 				} else {
// 					None
// 				};
// 				if let Some(tracer) = tracer {
// 					Ok((tracer, single::TraceType::CallList))
// 				} else {
// 					return Err(internal_err(format!(
// 						"javascript based tracing is not available (hash :{:?})",
// 						hash
// 					)));
// 				}
// 			}
// 			Some(params) => Ok((
// 				TracerInput::None,
// 				single::TraceType::Raw {
// 					disable_storage: params.disable_storage.unwrap_or(false),
// 					disable_memory: params.disable_memory.unwrap_or(false),
// 					disable_stack: params.disable_stack.unwrap_or(false),
// 				},
// 			)),
// 			_ => Ok((
// 				TracerInput::None,
// 				single::TraceType::Raw {
// 					disable_storage: false,
// 					disable_memory: false,
// 					disable_stack: false,
// 				},
// 			)),
// 		}
// 	}

// 	fn handle_block_request(
// 		client: Arc<C>,
// 		backend: Arc<BE>,
// 		frontier_backend: Arc<fc_db::Backend<B>>,
// 		request_block_id: RequestBlockId,
// 		params: Option<TraceParams>,
// 	) -> RpcResult<Response> {
// 		let (tracer_input, trace_type) = Self::handle_params(params)?;

// 		let reference_id: BlockId<B> = match request_block_id {
// 			RequestBlockId::Number(n) => Ok(BlockId::Number(n.unique_saturated_into())),
// 			RequestBlockId::Tag(RequestBlockTag::Latest) => {
// 				Ok(BlockId::Number(client.info().best_number))
// 			}
// 			RequestBlockId::Tag(RequestBlockTag::Earliest) => {
// 				Ok(BlockId::Number(0u32.unique_saturated_into()))
// 			}
// 			RequestBlockId::Tag(RequestBlockTag::Pending) => {
// 				Err(internal_err("'pending' blocks are not supported"))
// 			}
// 			RequestBlockId::Hash(eth_hash) => {
// 				match frontier_backend_client::load_hash::<B>(frontier_backend.as_ref(), eth_hash) {
// 					Ok(Some(id)) => Ok(id),
// 					Ok(_) => Err(internal_err("Block hash not found".to_string())),
// 					Err(e) => Err(e),
// 				}
// 			}
// 		}?;

// 		// Get ApiRef. This handle allow to keep changes between txs in an internal buffer.
// 		let api = client.runtime_api();
// 		// Get Blockchain backend
// 		let blockchain = backend.blockchain();
// 		// Get the header I want to work with.
// 		let header = client.header(reference_id).unwrap().unwrap();
// 		// Get parent blockid.
// 		let parent_block_id = BlockId::Hash(*header.parent_hash());

// 		let statuses = api
// 			.current_transaction_statuses(&reference_id)
// 			.map_err(|e| {
// 				internal_err(format!(
// 					"Failed to get Ethereum block data for Substrate block {:?} : {:?}",
// 					request_block_id, e
// 				))
// 			})?;

// 		// Get the extrinsics.
// 		let ext = blockchain.body(reference_id).unwrap().unwrap();
// 		// Known ethereum transaction hashes.
// 		let eth_tx_hashes = statuses
// 			.unwrap()
// 			.iter()
// 			.map(|t| t.transaction_hash)
// 			.collect();

// 		// Trace the block.
// 		let f = || -> RpcResult<_> {
// 			api.initialize_block(&parent_block_id, &header)
// 				.map_err(|e| internal_err(format!("Runtime api access error: {:?}", e)))?;

// 			let _result = api
// 				.trace_block(&parent_block_id, ext, eth_tx_hashes)
// 				.map_err(|e| {
// 					internal_err(format!(
// 						"Blockchain error when replaying block {} : {:?}",
// 						reference_id, e
// 					))
// 				})?
// 				.map_err(|e| {
// 					internal_err(format!(
// 						"Internal runtime error when replaying block {} : {:?}",
// 						reference_id, e
// 					))
// 				})?;
// 			Ok(moonbeam_rpc_primitives_debug::Response::Block)
// 		};

// 		return match trace_type {
// 			single::TraceType::CallList => {
// 				let mut proxy = moonbeam_client_evm_tracing::listeners::CallList::default();
// 				proxy.using(f)?;
// 				proxy.finish_transaction();
// 				let response = match tracer_input {
// 					TracerInput::CallTracer => {
// 						moonbeam_client_evm_tracing::formatters::CallTracer::format(proxy)
// 							.ok_or("Trace result is empty.")
// 							.map_err(|e| internal_err(format!("{:?}", e)))
// 					}
// 					_ => Err(internal_err(format!(
// 						"Bug: failed to resolve the tracer format."
// 					))),
// 				}?;
// 				println!("---> Response {:?}", response);

// 				Ok(Response::Block(response))
// 			}
// 			not_supported => Err(internal_err(format!(
// 				"Bug: `handle_block_request` does not support {:?}.",
// 				not_supported
// 			))),
// 		};
// 	}

// 	/// Replays a transaction in the Runtime at a given block height.
// 	///
// 	/// In order to succesfully reproduce the result of the original transaction we need a correct
// 	/// state to replay over.
// 	///
// 	/// Substrate allows to apply extrinsics in the Runtime and thus creating an overlayed state.
// 	/// This overlayed changes will live in-memory for the lifetime of the ApiRef.
// 	fn handle_transaction_request(
// 		client: Arc<C>,
// 		backend: Arc<BE>,
// 		frontier_backend: Arc<fc_db::Backend<B>>,
// 		transaction_hash: H256,
// 		params: Option<TraceParams>,
// 	) -> RpcResult<Response> {
// 		let (tracer_input, trace_type) = Self::handle_params(params)?;

// 		let (hash, index) = match frontier_backend_client::load_transactions::<B, C>(
// 			client.as_ref(),
// 			frontier_backend.as_ref(),
// 			transaction_hash,
// 			false,
// 		) {
// 			Ok(Some((hash, index))) => (hash, index as usize),
// 			Ok(None) => return Err(internal_err("Transaction hash not found".to_string())),
// 			Err(e) => return Err(e),
// 		};

// 		let reference_id =
// 			match frontier_backend_client::load_hash::<B>(frontier_backend.as_ref(), hash) {
// 				Ok(Some(hash)) => hash,
// 				Ok(_) => return Err(internal_err("Block hash not found".to_string())),
// 				Err(e) => return Err(e),
// 			};
// 		// Get ApiRef. This handle allow to keep changes between txs in an internal buffer.
// 		let api = client.runtime_api();
// 		// Get Blockchain backend
// 		let blockchain = backend.blockchain();
// 		// Get the header I want to work with.
// 		let header = client.header(reference_id).unwrap().unwrap();
// 		// Get parent blockid.
// 		let parent_block_id = BlockId::Hash(*header.parent_hash());

// 		// Get the extrinsics.
// 		let ext = blockchain.body(reference_id).unwrap().unwrap();

// 		// Get the block that contains the requested transaction.
// 		let reference_block = match api.current_block(&reference_id) {
// 			Ok(block) => block,
// 			Err(e) => return Err(internal_err(format!("Runtime block call failed: {:?}", e))),
// 		};

// 		// Get the actual ethereum transaction.
// 		if let Some(block) = reference_block {
// 			let transactions = block.transactions;
// 			if let Some(transaction) = transactions.get(index) {
// 				let f = || -> RpcResult<_> {
// 					api.initialize_block(&parent_block_id, &header)
// 						.map_err(|e| internal_err(format!("Runtime api access error: {:?}", e)))?;

// 					let _result = api
// 						.trace_transaction(&parent_block_id, ext, &transaction)
// 						.map_err(|e| internal_err(format!("Runtime api access error: {:?}", e)))?
// 						.map_err(|e| internal_err(format!("DispatchError: {:?}", e)))?;

// 					Ok(moonbeam_rpc_primitives_debug::Response::Single)
// 				};

// 				return match trace_type {
// 					single::TraceType::Raw {
// 						disable_storage,
// 						disable_memory,
// 						disable_stack,
// 					} => {
// 						let mut proxy = moonbeam_client_evm_tracing::listeners::Raw::new(
// 							disable_storage,
// 							disable_memory,
// 							disable_stack,
// 						);
// 						proxy.using(f)?;
// 						Ok(Response::Single(
// 							moonbeam_client_evm_tracing::formatters::Raw::format(proxy).unwrap(),
// 						))
// 					}
// 					single::TraceType::CallList => {
// 						let mut proxy = moonbeam_client_evm_tracing::listeners::CallList::default();
// 						proxy.using(f)?;
// 						proxy.finish_transaction();
// 						let response = match tracer_input {
// 							TracerInput::Blockscout => {
// 								moonbeam_client_evm_tracing::formatters::Blockscout::format(proxy)
// 									.ok_or("Trace result is empty.")
// 									.map_err(|e| internal_err(format!("{:?}", e)))
// 							}
// 							TracerInput::CallTracer => {
// 								let mut res =
// 									moonbeam_client_evm_tracing::formatters::CallTracer::format(
// 										proxy,
// 									)
// 									.ok_or("Trace result is empty.")
// 									.map_err(|e| internal_err(format!("{:?}", e)))?;
// 								Ok(res.pop().unwrap())
// 							}
// 							_ => Err(internal_err(format!(
// 								"Bug: failed to resolve the tracer format."
// 							))),
// 						}?;
// 						Ok(Response::Single(response))
// 					}
// 					not_supported => Err(internal_err(format!(
// 						"Bug: `handle_transaction_request` does not support {:?}.",
// 						not_supported
// 					))),
// 				};
// 			}
// 		}
// 		Err(internal_err("Runtime block call failed".to_string()))
// 	}
// }
