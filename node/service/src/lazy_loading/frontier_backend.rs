// Copyright 2025 Moonbeam foundation
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

use fc_api::{LogIndexerBackend, TransactionMetadata};
use serde::de::DeserializeOwned;
use sp_core::H256;
use sp_runtime::traits::{Block as BlockT, Header};
use std::sync::Arc;

#[derive(Clone)]
pub struct LazyLoadingFrontierBackend<Block: BlockT> {
	pub(crate) rpc_client: Arc<super::rpc_client::RPC>,
	pub(crate) frontier_backend: Arc<dyn fc_api::Backend<Block> + Send + Sync>,
}

#[async_trait::async_trait]
impl<Block: BlockT + DeserializeOwned> fc_api::Backend<Block> for LazyLoadingFrontierBackend<Block>
where
	<Block::Header as Header>::Number: From<u32>,
{
	async fn block_hash(&self, eth_block_hash: &H256) -> Result<Option<Vec<Block::Hash>>, String> {
		let result = self
			.rpc_client
			.block_by_hash::<Block>(eth_block_hash, false)
			.map_err(|e| format!("failed to get block by hash: {:?}", e))?;

		if let Some(block) = result {
			let block_number: <Block::Header as Header>::Number =
				block.header.number.as_u32().into();

			let substrate_block_hash = self
				.rpc_client
				.block_hash::<Block>(Some(block_number))
				.map_err(|e| format!("failed to get block: {:?}", e))?;

			return Ok(substrate_block_hash.map(|h| vec![h]));
		}

		Ok(None)
	}

	async fn transaction_metadata(
		&self,
		eth_transaction_hash: &H256,
	) -> Result<Vec<TransactionMetadata<Block>>, String> {
		let result = self
			.rpc_client
			.transaction_by_hash(eth_transaction_hash)
			.map_err(|e| format!("failed to get transaction by hash: {:?}", e))?;

		if let Some(tx) = result {
			let block_number: <Block::Header as Header>::Number =
				tx.block_number.unwrap_or_default().as_u32().into();

			let substrate_block_hash = self
				.rpc_client
				.block_hash(Some(block_number))
				.map_err(|e| format!("failed to get block: {:?}", e))?;

			return Ok(vec![TransactionMetadata::<Block> {
				ethereum_index: tx.transaction_index.unwrap_or_default().as_u32(),
				ethereum_block_hash: tx.block_hash.unwrap_or_default(),
				substrate_block_hash: substrate_block_hash.unwrap_or_default(),
			}]);
		}

		Ok(vec![])
	}

	fn log_indexer(&self) -> &dyn LogIndexerBackend<Block> {
		self.frontier_backend.log_indexer()
	}

	async fn first_block_hash(&self) -> Result<Block::Hash, String> {
		self.frontier_backend.first_block_hash().await
	}

	async fn latest_block_hash(&self) -> Result<Block::Hash, String> {
		self.frontier_backend.latest_block_hash().await
	}
}
