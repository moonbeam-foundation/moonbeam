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

use ethereum_types::H256;
use jsonrpc_core::Result as RpcResult;
use jsonrpc_core::{Error as RpcError, ErrorCode};
pub use moonbeam_rpc_core_debug::{Debug as DebugT, DebugServer, StepLog, TraceExecutorResponse};
use sc_client_api::backend::AuxStore;
use sp_api::{BlockId, HeaderT, ProvideRuntimeApi};
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};
use sp_runtime::traits::Block as BlockT;
use std::{marker::PhantomData, sync::Arc};

use moonbeam_rpc_primitives_debug::DebugRuntimeApi;

pub fn internal_err<T: ToString>(message: T) -> RpcError {
	RpcError {
		code: ErrorCode::InternalError,
		message: message.to_string(),
		data: None,
	}
}

pub struct Debug<B: BlockT, C> {
	client: Arc<C>,
	_marker: PhantomData<B>,
}

impl<B: BlockT, C> Debug<B, C> {
	pub fn new(client: Arc<C>) -> Self {
		Self {
			client,
			_marker: PhantomData,
		}
	}
}

impl<B, C> Debug<B, C>
where
	C: ProvideRuntimeApi<B> + AuxStore,
	C: HeaderMetadata<B, Error = BlockChainError> + HeaderBackend<B>,
	C: Send + Sync + 'static,
	B: BlockT<Hash = H256> + Send + Sync + 'static,
	C::Api: DebugRuntimeApi<B>,
{
	// Asumes there is only one mapped canonical block in the AuxStore, otherwise something is wrong
	fn load_hash(&self, hash: H256) -> RpcResult<Option<BlockId<B>>> {
		let hashes = match fc_consensus::load_block_hash::<B, _>(self.client.as_ref(), hash)
			.map_err(|err| internal_err(format!("fetch aux store failed: {:?}", err)))?
		{
			Some(hashes) => hashes,
			None => return Ok(None),
		};
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
		let mut transactions: Vec<(H256, u32)> = Vec::new();
		match fc_consensus::load_transaction_metadata(self.client.as_ref(), transaction_hash)
			.map_err(|err| internal_err(format!("fetch aux store failed: {:?}", err)))?
		{
			Some(metadata) => {
				for (block_hash, index) in metadata {
					match self
						.load_hash(block_hash)
						.map_err(|err| internal_err(format!("{:?}", err)))?
					{
						Some(_) => {
							transactions.push((block_hash, index));
						}
						_ => {}
					};
				}
			}
			None => return Ok(None),
		};

		if transactions.len() == 1 {
			return Ok(Some(transactions[0]));
		}
		Ok(None)
	}
}

impl<B, C> DebugT for Debug<B, C>
where
	C: ProvideRuntimeApi<B> + AuxStore,
	C: HeaderMetadata<B, Error = BlockChainError> + HeaderBackend<B>,
	C: Send + Sync + 'static,
	B: BlockT<Hash = H256> + Send + Sync + 'static,
	C::Api: DebugRuntimeApi<B>,
{
	fn trace_transaction(&self, transaction_hash: H256) -> RpcResult<TraceExecutorResponse> {
		let (hash, index) = match self
			.load_transactions(transaction_hash)
			.map_err(|err| internal_err(format!("{:?}", err)))?
		{
			Some((hash, index)) => (hash, index as usize),
			None => return Err(internal_err(format!("Transaction hash not found"))),
		};

		let id = match self
			.load_hash(hash)
			.map_err(|err| internal_err(format!("{:?}", err)))?
		{
			Some(hash) => hash,
			_ => return Err(internal_err(format!("Block hash not found"))),
		};

		let res = self
			.client
			.runtime_api()
			.trace_transaction(&id, index as u32)
			.map_err(|err| internal_err(format!("Runtime call failed: {:?}", err)))?
			.unwrap();

		Ok(TraceExecutorResponse {
			gas: res.gas,
			return_value: res.return_value,
			step_logs: res
				.step_logs
				.iter()
				.map(|s| StepLog {
					depth: s.depth,
					gas: s.gas,
					gas_cost: s.gas_cost,
					memory: s.memory.clone(),
					op: s.op.clone(),
					pc: s.pc,
					stack: s.stack.clone(),
					storage: s.storage.clone(),
				})
				.collect(),
		})
	}
}
