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
use fc_rpc::frontier_backend_client::{self, is_canon};

use jsonrpsee::types::error::ErrorObject;
use jsonrpsee::{core::RpcResult, proc_macros::rpc};
use sp_blockchain::HeaderBackend;
use sp_core::H256;
use sp_runtime::traits::Block;
use std::ops::Deref;
use std::{marker::PhantomData, sync::Arc};

/// An RPC endpoint to check for finality of blocks and transactions in Moonbeam
#[rpc(server)]
#[async_trait::async_trait]
pub trait MoonbeamFinalityApi {
	/// Reports whether a Substrate or Ethereum block is finalized.
	/// Returns false if the block is not found.
	#[method(name = "moon_isBlockFinalized")]
	async fn is_block_finalized(&self, block_hash: H256) -> RpcResult<bool>;

	/// Reports whether an Ethereum transaction is finalized.
	/// Returns false if the transaction is not found
	#[method(name = "moon_isTxFinalized")]
	async fn is_tx_finalized(&self, tx_hash: H256) -> RpcResult<bool>;

	/// Gets the latest block hash that is fully indexed in frontier's backend.
	#[method(name = "moon_getLatestBlockHash")]
	async fn get_latest_block_hash(&self) -> RpcResult<H256>;
}

pub struct MoonbeamFinality<B: Block, C> {
	pub backend: Arc<dyn fc_api::Backend<B>>,
	pub client: Arc<C>,
	_phdata: PhantomData<B>,
}

impl<B: Block, C> MoonbeamFinality<B, C> {
	pub fn new(client: Arc<C>, backend: Arc<dyn fc_api::Backend<B>>) -> Self {
		Self {
			backend,
			client,
			_phdata: Default::default(),
		}
	}
}

#[async_trait::async_trait]
impl<B, C> MoonbeamFinalityApiServer for MoonbeamFinality<B, C>
where
	B: Block<Hash = H256>,
	C: HeaderBackend<B> + Send + Sync + 'static,
{
	async fn is_block_finalized(&self, raw_hash: H256) -> RpcResult<bool> {
		let client = self.client.clone();
		is_block_finalized_inner::<B, C>(self.backend.as_ref(), &client, raw_hash).await
	}

	async fn is_tx_finalized(&self, tx_hash: H256) -> RpcResult<bool> {
		let client = self.client.clone();

		if let Some((ethereum_block_hash, _ethereum_index)) =
			frontier_backend_client::load_transactions::<B, C>(
				&client,
				self.backend.as_ref(),
				tx_hash,
				true,
			)
			.await?
		{
			is_block_finalized_inner::<B, C>(self.backend.as_ref(), &client, ethereum_block_hash)
				.await
		} else {
			Ok(false)
		}
	}

	async fn get_latest_block_hash(&self) -> RpcResult<H256> {
		let res = self.backend.deref().latest_block_hash().await;
		match res {
			Ok(val) => Ok(val),
			Err(e) => Err(ErrorObject::owned(
				jsonrpsee::types::error::UNKNOWN_ERROR_CODE,
				"No synced block",
				Some(e),
			)),
		}
	}
}

async fn is_block_finalized_inner<B: Block<Hash = H256>, C: HeaderBackend<B> + 'static>(
	backend: &(dyn fc_api::Backend<B>),
	client: &C,
	raw_hash: H256,
) -> RpcResult<bool> {
	let substrate_hash =
		match frontier_backend_client::load_hash::<B, C>(client, backend, raw_hash).await? {
			// If we find this hash in the frontier data base, we know it is an eth hash
			Some(hash) => hash,
			// Otherwise, we assume this is a Substrate hash.
			None => raw_hash,
		};

	// First check whether the block is in the best chain
	if !is_canon(client, substrate_hash) {
		return Ok(false);
	}

	// At this point we know the block in question is in the current best chain.
	// It's just a question of whether it is in the finalized prefix or not
	let query_height = client
		.number(substrate_hash)
		.expect("No sp_blockchain::Error should be thrown when looking up hash")
		.expect("Block is already known to be canon, so it must be in the chain");
	let finalized_height = client.info().finalized_number;

	Ok(query_height <= finalized_height)
}
