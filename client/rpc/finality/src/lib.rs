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
use fc_rpc::{
	frontier_backend_client::{self, is_canon},
	internal_err,
};
use futures::{future::BoxFuture, FutureExt as _};
use jsonrpc_core::Result as RpcResult;
use jsonrpc_derive::rpc;
use sp_core::H256;
use std::{marker::PhantomData, sync::Arc};
//TODO ideally we wouldn't depend on BlockId here. Can we change frontier
// so it's load_hash helper returns an H256 instead of wrapping it in a BlockId?
use fc_db::Backend as FrontierBackend;
use sp_api::BlockId;
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::Block;

/// An RPC endpoint to check for finality of blocks and transactions in Moonbeam
#[rpc(server)]
pub trait MoonbeamFinalityApi {
	/// Reports whether a Moonbeam or Ethereum block is finalized.
	/// Errors if the block is not found. //TODO reevaluate design later. Could just return false.
	/// TODO Should I be generic over the hash type? Probably; ChainApi is. Although chain api isn't moonbeam specific so...
	#[rpc(name = "moon_isBlockFinalized")]
	fn is_block_finalized(&self, block_hash: H256) -> BoxFuture<'static, RpcResult<bool>>;

	/// Reports whether a Moonbeam or Ethereum transaction is finalized.
	#[rpc(name = "moon_isTxFinalized")]
	fn is_tx_finalized(&self, tx_hash: H256) -> BoxFuture<'static, RpcResult<bool>>;
}

pub struct MoonbeamFinality<B: Block, C> {
	pub backend: Arc<FrontierBackend<B>>,
	pub client: Arc<C>,
	_phdata: PhantomData<B>,
}

impl<B, C> MoonbeamFinalityApi for MoonbeamFinality<B, C>
where
	C: Send + Sync + 'static,
	B: Block<Hash = H256>,
	C: HeaderBackend<B>,
{
	fn is_block_finalized(&self, raw_hash: H256) -> BoxFuture<'static, RpcResult<bool>> {
		let backend = self.backend.clone();
		let client = self.client.clone();
		async move {
			let substrate_hash =
				match frontier_backend_client::load_hash::<B>(backend.as_ref(), raw_hash)
					.expect("frontier lookup should not error?")
				{
					// If we find this hash in the frontier data base, we know it is an eth hash
					Some(BlockId::Hash(hash)) => hash,
					Some(BlockId::Number(_)) => panic!("is_canon test only works with hashes."),
					// Otherwise, we assume this is a Substrate hash.
					None => raw_hash,
				};

			// Confirm that this block is in the best chain
			if !is_canon(client.as_ref(), substrate_hash) {
				return Ok(false);
			}

			// At this point we know the block in question is in the current best chain.
			// It's just a question of whether it is in the finalized prefix or not
			let query_height = client
				.number(substrate_hash)
				.expect("Blocks in best chain should have number")
				.expect("uhhhh");
			let finalized_height = client.info().finalized_number;

			Ok(query_height <= finalized_height)
		}
		.boxed()
	}

	fn is_tx_finalized(&self, tx_hash: H256) -> BoxFuture<'static, RpcResult<bool>> {
		async move {
			// Our goal here is to get a Substrate block, and then follow the same approach as
			// we did for the block check

			Ok(false)
		}
		.boxed()
	}
}
