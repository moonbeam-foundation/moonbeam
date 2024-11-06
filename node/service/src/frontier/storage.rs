// Copyright 2024 Moonbeam foundation
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

use block_patches::MOONBASE_BLOCK_PATCHES;

use std::sync::Arc;
use fc_rpc::{StorageOverride, StorageOverrideHandler};

use ethereum::{BlockV2, ReceiptV3};
use ethereum_types::{Address, H256, U256};
// Substrate
use sc_client_api::{backend::Backend, StorageProvider};
use sp_api::ProvideRuntimeApi;
use sp_runtime::{traits::Block as BlockT, Permill};
// Frontier
use fp_rpc::{EthereumRuntimeRPCApi, TransactionStatus};
use crate::frontier::block_patches;

/// A storage override for runtimes that use different ethereum schema.
///
/// It fetches data from the state backend, with some assumptions about pallet-ethereum's storage
/// schema, as a preference. However, if there is no ethereum schema in the state, it'll use the
/// runtime API as fallback implementation.
///
/// It is used to avoid spawning the runtime and the overhead associated with it.
#[derive(Clone)]
pub struct FrontierStorageOverrideHandler<B, C, BE> {
	handler: StorageOverrideHandler<B, C, BE>,
}

impl<B, C, BE> FrontierStorageOverrideHandler<B, C, BE> {
	pub fn new(client: Arc<C>) -> Self {
		Self {
			handler: StorageOverrideHandler::<B, C, BE>::new(client.clone())
		}
	}
}

impl<B, C, BE> StorageOverride<B> for FrontierStorageOverrideHandler<B, C, BE>
	where
		B: BlockT,
		C: ProvideRuntimeApi<B>,
		C::Api: EthereumRuntimeRPCApi<B>,
		C: StorageProvider<B, BE> + Send + Sync + 'static,
		BE: Backend<B> + 'static,
{
	fn account_code_at(&self, at: B::Hash, address: Address) -> Option<Vec<u8>> {
		self.handler.account_code_at(at, address)
	}

	fn account_storage_at(&self, at: B::Hash, address: Address, index: U256) -> Option<H256> {
		self.handler.account_storage_at(at, address, index)
	}

	fn current_block(&self, at: B::Hash) -> Option<BlockV2> {
		let block = self.handler.current_block(at);
		block.map(|mut block| {
			for patch in MOONBASE_BLOCK_PATCHES {
				if block.header.hash() == patch.hash {
					block.transactions = block.transactions.iter().filter_map(|tx| {
						if patch.invalid_transaction.contains(&tx.hash()) {
							None
						} else {
							Some(tx.clone())
						}
					}).collect();
				}
			}
			block
		})
	}

	fn current_receipts(&self, at: B::Hash) -> Option<Vec<ReceiptV3>> {
		self.handler.current_receipts(at)
	}

	fn current_transaction_statuses(&self, at: B::Hash) -> Option<Vec<TransactionStatus>> {
		self.handler.current_transaction_statuses(at)
	}

	fn elasticity(&self, at: B::Hash) -> Option<Permill> {
		self.handler.elasticity(at)
	}

	fn is_eip1559(&self, at: B::Hash) -> bool {
		self.handler.is_eip1559(at)
	}
}
