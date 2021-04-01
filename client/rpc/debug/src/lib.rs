// Copyright 2019-2020 PureStake Inc.
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

pub use moonbeam_rpc_core_debug::{Debug as DebugT, DebugServer, TraceParams};

use ethereum_types::{H128, H256};
use fp_rpc::EthereumRuntimeRPCApi;
use jsonrpc_core::Result as RpcResult;
use jsonrpc_core::{Error as RpcError, ErrorCode};
use moonbeam_rpc_primitives_debug::{single, DebugRuntimeApi};
use sc_client_api::backend::{AuxStore, Backend, StateBackend};
use sp_api::{BlockId, HeaderT, ProvideRuntimeApi};
use sp_block_builder::BlockBuilder;
use sp_blockchain::{
	Backend as BlockchainBackend, Error as BlockChainError, HeaderBackend, HeaderMetadata,
};
use sp_runtime::traits::{BlakeTwo256, Block as BlockT};
use std::{marker::PhantomData, str::FromStr, sync::Arc};

pub fn internal_err<T: ToString>(message: T) -> RpcError {
	RpcError {
		code: ErrorCode::InternalError,
		message: message.to_string(),
		data: None,
	}
}

pub struct Debug<B: BlockT, C, BE> {
	client: Arc<C>,
	backend: Arc<BE>,
	frontier_backend: Arc<fc_db::Backend<B>>,
}

impl<B: BlockT, C, BE> Debug<B, C, BE> {
	pub fn new(client: Arc<C>, backend: Arc<BE>, frontier_backend: Arc<fc_db::Backend<B>>) -> Self {
		Self {
			client,
			backend,
			frontier_backend,
		}
	}
}

impl<B, C, BE> Debug<B, C, BE>
where
	BE: Backend<B> + 'static,
	BE::State: StateBackend<BlakeTwo256>,
	BE::Blockchain: BlockchainBackend<B>,
	C: ProvideRuntimeApi<B> + AuxStore,
	C: HeaderMetadata<B, Error = BlockChainError> + HeaderBackend<B>,
	C: Send + Sync + 'static,
	B: BlockT<Hash = H256> + Send + Sync + 'static,
	C::Api: BlockBuilder<B>,
	C::Api: DebugRuntimeApi<B>,
	C::Api: EthereumRuntimeRPCApi<B>,
{
	// Asumes there is only one mapped canonical block in the AuxStore, otherwise something is wrong
	fn load_hash(&self, hash: H256) -> RpcResult<Option<BlockId<B>>> {
		let hashes = self
			.frontier_backend
			.mapping()
			.block_hashes(&hash)
			.map_err(|err| internal_err(format!("fetch aux store failed: {:?}", err)))?;
		let out: Vec<H256> = hashes
			.into_iter()
			.filter_map(|h| if self.is_canon(h) { Some(h) } else { None })
			.collect();

		if out.len() == 1 {
			return Ok(Some(BlockId::Hash(out[0])));
		}
		Ok(None)
	}

	fn is_canon(&self, target_hash: H256) -> bool {
		if let Ok(Some(number)) = self.client.number(target_hash) {
			if let Ok(Some(header)) = self.client.header(BlockId::Number(number)) {
				return header.hash() == target_hash;
			}
		}
		false
	}

	fn load_transactions(&self, transaction_hash: H256) -> RpcResult<Option<(H256, u32)>> {
		let transaction_metadata = self
			.frontier_backend
			.mapping()
			.transaction_metadata(&transaction_hash)
			.map_err(|err| internal_err(format!("fetch aux store failed: {:?}", err)))?;

		if transaction_metadata.len() == 1 {
			Ok(Some((
				transaction_metadata[0].ethereum_block_hash,
				transaction_metadata[0].ethereum_index,
			)))
		} else if transaction_metadata.len() > 1 {
			transaction_metadata
				.iter()
				.find(|meta| self.is_canon(meta.block_hash))
				.map_or(
					Ok(Some((
						transaction_metadata[0].ethereum_block_hash,
						transaction_metadata[0].ethereum_index,
					))),
					|meta| Ok(Some((meta.ethereum_block_hash, meta.ethereum_index))),
				)
		} else {
			Ok(None)
		}
	}
}

impl<B, C, BE> DebugT for Debug<B, C, BE>
where
	BE: Backend<B> + 'static,
	BE::State: StateBackend<BlakeTwo256>,
	BE::Blockchain: BlockchainBackend<B>,
	C: ProvideRuntimeApi<B> + AuxStore,
	C: HeaderMetadata<B, Error = BlockChainError> + HeaderBackend<B>,
	C: Send + Sync + 'static,
	B: BlockT<Hash = H256> + Send + Sync + 'static,
	C::Api: BlockBuilder<B>,
	C::Api: DebugRuntimeApi<B>,
	C::Api: EthereumRuntimeRPCApi<B>,
{
	fn trace_transaction(
		&self,
		transaction_hash: H256,
		params: Option<TraceParams>,
	) -> RpcResult<single::TransactionTrace> {
		let (hash, index) = match self
			.load_transactions(transaction_hash)
			.map_err(|err| internal_err(format!("{:?}", err)))?
		{
			Some((hash, index)) => (hash, index as usize),
			None => return Err(internal_err("Transaction hash not found".to_string())),
		};

		let reference_id = match self
			.load_hash(hash)
			.map_err(|err| internal_err(format!("{:?}", err)))?
		{
			Some(hash) => hash,
			_ => return Err(internal_err("Block hash not found".to_string())),
		};

		// Get ApiRef. This handle allow to keep changes between txs in an internal buffer.
		let api = self.client.runtime_api();
		// Get Blockchain backend
		let blockchain = self.backend.blockchain();
		// Get the header I want to work with.
		let header = self.client.header(reference_id).unwrap().unwrap();
		// Get parent blockid.
		let parent_block_id = BlockId::Hash(*header.parent_hash());

		// Get the extrinsics.
		let ext = blockchain.body(reference_id).unwrap().unwrap();

		// Get the block that contains the requested transaction.
		let reference_block = api
			.current_block(&reference_id)
			.map_err(|err| internal_err(format!("Runtime block call failed: {:?}", err)))?;

		// Set trace type
		let trace_type = match params {
			Some(TraceParams {
				tracer: Some(tracer), ..
			}) => {
				let hash: H128 = sp_io::hashing::twox_128(&tracer.as_bytes()).into();
				let blockscout_hash = H128::from_str("0x94d9f08796f91eb13a2e82a6066882f7").unwrap();
				if hash == blockscout_hash {
					single::TraceType::CallList
				} else {
					return Err(internal_err(format!(
						"javascript based tracing is not available (hash :{:?})",
						hash
					)));
				}
			}
			_ => single::TraceType::Raw,
		};

		// Get the actual ethereum transaction.
		if let Some(block) = reference_block {
			let transactions = block.transactions;
			if let Some(transaction) = transactions.get(index) {
				let res = api
					.trace_transaction(&parent_block_id, ext, transaction, trace_type)
					.map_err(|err| internal_err(format!("Runtime trace call failed: {:?}", err)))?
					.unwrap();

				return Ok(res);
			}
		}

		Err(internal_err("Runtime block call failed".to_string()))
	}
}
