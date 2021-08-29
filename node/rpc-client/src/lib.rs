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

//! RPC client

use jsonrpsee_ws_client::{
	types::{traits::Client, v2::params::JsonRpcParams},
	WsClient, WsClientBuilder,
};
use sp_runtime::traits::{Block as BlockT, Header as HeaderT};

pub struct RpcClient(WsClient);

impl RpcClient {
	/// Build a website client that connects to `url`.
	pub async fn new<S: AsRef<str>>(url: S) -> Result<Self, String> {
		WsClientBuilder::default()
			.max_request_body_size(u32::MAX)
			.build(url.as_ref())
			.await
			.map(Self)
			.map_err(|e| format!("`WsClientBuilder` failed to build: {:?}", e))
	}
	/// Get the hash of block number `block_number`
	pub async fn get_block_hash<Block>(
		&self,
		block_number: <Block::Header as HeaderT>::Number,
	) -> Result<Block::Hash, String>
	where
		Block: BlockT + serde::de::DeserializeOwned,
		Block::Header: HeaderT,
	{
		let params = vec![serde_json::Value::String(block_number.to_string())];
		let block_hash = self
			.0
			.request::<Block::Hash>("chain_getBlockHash", JsonRpcParams::Array(params))
			.await
			.map_err(|e| format!("chain_getBlock request failed: {:?}", e))?;

		Ok(block_hash)
	}
	/// Get the signed block identified by `at`.
	pub async fn get_block<Block>(&self, at: Block::Hash) -> Result<Block, String>
	where
		Block: BlockT + serde::de::DeserializeOwned,
		Block::Header: HeaderT,
	{
		let params = vec![hash_to_json::<Block>(at)?];
		let signed_block = self
			.0
			.request::<sp_runtime::generic::SignedBlock<Block>>(
				"chain_getBlock",
				JsonRpcParams::Array(params),
			)
			.await
			.map_err(|e| format!("chain_getBlock request failed: {:?}", e))?;

		Ok(signed_block.block)
	}
}

/// Convert a block hash to a serde json value.
fn hash_to_json<Block: BlockT>(hash: Block::Hash) -> Result<serde_json::Value, String> {
	serde_json::to_value(hash)
		.map_err(|e| format!("Block hash could not be converted to JSON: {:?}", e))
}
