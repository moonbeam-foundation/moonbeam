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
use frame_system::AccountInfo;
use jsonrpsee::{
	core::{Error as JsonRpcError, RpcResult},
	proc_macros::rpc,
};
use moonbeam_core_primitives::{AccountId, BlockNumber, Hash};
use sc_client_api::backend::{Backend, StateBackend, StorageProvider};
use sc_rpc_api::DenyUnsafe;
use sp_blockchain::HeaderBackend;
use sp_core::{hexdisplay::HexDisplay, storage::StorageKey};
use sp_rpc::number::NumberOrHex;
use sp_runtime::traits::{BlakeTwo256, Block as BlockT};
use std::{marker::PhantomData, sync::Arc};

const LOG_TARGET: &str = "moon-accounts";

/// An RPC endpoint to check for finality of blocks and transactions in Moonbeam
#[rpc(server)]
pub trait MoonbeamCheckApi {
	/// Reports whether a Substrate or Ethereum block is finalized.
	/// Returns false if the block is not found.
	#[method(name = "moon_accounts")]
	fn check(&self, block_number: Option<NumberOrHex>) -> RpcResult<u32>;
}

pub struct MoonbeamCheck<B: BlockT, C, BE /*, T*/> {
	pub client: Arc<C>,
	_phdata: PhantomData<B>,
	_bedata: PhantomData<BE>,
	//_codata: PhantomData<T>,
	/// Whether to deny unsafe calls
	deny_unsafe: DenyUnsafe,
}

impl<B: BlockT, C, BE /*, T*/> MoonbeamCheck<B, C, BE /*, T*/> {
	pub fn new(client: Arc<C>, deny_unsafe: DenyUnsafe) -> Self {
		Self {
			client,
			_phdata: Default::default(),
			_bedata: Default::default(),
			//_codata: Default::default(),
			deny_unsafe,
		}
	}
}

impl<B, C, BE /*, T*/> MoonbeamCheckApiServer for MoonbeamCheck<B, C, BE /*, T*/>
where
	BE: Backend<B> + 'static,
	BE::State: StateBackend<BlakeTwo256>,
	B: BlockT,
	C: StorageProvider<B, BE>,
	C: HeaderBackend<B> + Send + Sync + 'static,
	//T: frame_system::Config<AccountId = AccountId, Hash = Hash>+ Send + Sync + 'static,
{
	fn check(&self, number: Option<NumberOrHex>) -> RpcResult<u32> {
		self.deny_unsafe.check_if_safe()?;

		let block_hash = match number {
			None => Ok(Some(self.client.info().best_hash)),
			Some(num_or_hex) => {
				let block_num: u32 = num_or_hex.try_into().map_err(|_| {
					JsonRpcError::Custom(format!(
						"`{:?}` > u32::MAX, the max block number is u32.",
						num_or_hex
					))
				})?;
				let block_num = <BlockNumber>::from(block_num);
				self.client
					.hash(block_num.into())
					.map_err(|err| JsonRpcError::Custom(err.to_string()))
			}
		}
		.expect("Failed to get block hash")
		.unwrap();

		let accounts_prefix = StorageKey(frame_support::storage::storage_prefix(b"System", b"Account").to_vec());

		let mut last_key: Option<StorageKey> = None;
		let mut keys_count: usize = 0;
		let max_items = 10000;

		loop {
			let page = self
				.client
				.storage_keys(
					block_hash,
					Some(&accounts_prefix),
					last_key.as_ref(),
				)
				.map_err(|e| {
					log::error!(target: LOG_TARGET, "Error = {:?}", e);
					JsonRpcError::Custom(format!("rpc get_keys failed"))
				})
				.unwrap()
				.take(max_items);
			let mut page_len = 1;
			let new_last_key = page.reduce(|_, k| {
				page_len += 1;
				k
			}).unwrap();
			keys_count += page_len;

			if page_len < max_items as usize {
				log::debug!(target: LOG_TARGET, "last page received: {}", page_len);
				break;
			} else {
				log::debug!(
					target: LOG_TARGET,
					"new total = {}, full page received: {}",
					page_len,
					HexDisplay::from(&new_last_key.clone())
				);
				last_key = Some(new_last_key.clone());
			}
		}

		Ok(keys_count.try_into().unwrap())
	}
}
