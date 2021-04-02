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
use fc_rpc::{frontier_backend_client, internal_err};
use fp_rpc::EthereumRuntimeRPCApi;
use jsonrpc_core::Result as RpcResult;
use moonbeam_rpc_primitives_debug::{single, DebugRuntimeApi};
use sc_client_api::backend::Backend;
use sp_api::{BlockId, HeaderT, ProvideRuntimeApi};
use sp_block_builder::BlockBuilder;
use sp_blockchain::{
	Backend as BlockchainBackend, Error as BlockChainError, HeaderBackend, HeaderMetadata,
};
use sp_runtime::traits::Block as BlockT;
use std::{str::FromStr, sync::Arc};

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

impl<B, C, BE> DebugT for Debug<B, C, BE>
where
	BE: Backend<B> + 'static,
	C: ProvideRuntimeApi<B>,
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
		let (hash, index) = match frontier_backend_client::load_transactions::<B, C>(
			self.client.as_ref(),
			self.frontier_backend.as_ref(),
			transaction_hash,
		)
		.map_err(|err| internal_err(format!("{:?}", err)))?
		{
			Some((hash, index)) => (hash, index as usize),
			None => return Err(internal_err("Transaction hash not found".to_string())),
		};

		let reference_id = match frontier_backend_client::load_hash::<B, C>(
			self.client.as_ref(),
			self.frontier_backend.as_ref(),
			hash,
		)
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
				tracer: Some(tracer),
				..
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
