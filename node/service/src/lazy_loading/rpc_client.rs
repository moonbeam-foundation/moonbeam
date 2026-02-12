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

use cumulus_primitives_core::BlockT;
use fc_rpc_v2_api::types::H256;
use jsonrpsee::http_client::HttpClient;
use moonbeam_core_primitives::BlockNumber;
use serde::de::DeserializeOwned;
use sp_api::__private::HeaderT;
use sp_rpc::list::ListOrValue;
use sp_rpc::number::NumberOrHex;
use sp_runtime::generic::SignedBlock;
use sp_storage::{StorageData, StorageKey};
use std::future::Future;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio_retry::strategy::FixedInterval;
use tokio_retry::Retry;

#[derive(Debug, Clone)]
pub struct RPC {
	http_client: HttpClient,
	delay_between_requests_ms: u32,
	max_retries_per_request: u32,
	counter: Arc<AtomicU64>,
}

impl RPC {
	pub fn new(
		http_client: HttpClient,
		delay_between_requests_ms: u32,
		max_retries_per_request: u32,
	) -> Self {
		Self {
			http_client,
			delay_between_requests_ms,
			max_retries_per_request,
			counter: Default::default(),
		}
	}
	pub fn system_chain(&self) -> Result<String, jsonrpsee::core::ClientError> {
		let request = &|| {
			substrate_rpc_client::SystemApi::<H256, BlockNumber>::system_chain(&self.http_client)
		};

		self.block_on(request)
	}

	pub fn system_properties(
		&self,
	) -> Result<sc_chain_spec::Properties, jsonrpsee::core::ClientError> {
		let request = &|| {
			substrate_rpc_client::SystemApi::<H256, BlockNumber>::system_properties(
				&self.http_client,
			)
		};

		self.block_on(request)
	}

	pub fn block<Block, Hash: Clone>(
		&self,
		hash: Option<Hash>,
	) -> Result<Option<SignedBlock<Block>>, jsonrpsee::core::ClientError>
	where
		Block: BlockT + DeserializeOwned,
		Hash: 'static + Send + Sync + sp_runtime::Serialize + DeserializeOwned,
	{
		let request = &|| {
			substrate_rpc_client::ChainApi::<
				BlockNumber,
				Hash,
				Block::Header,
				SignedBlock<Block>,
			>::block(&self.http_client, hash.clone())
		};

		self.block_on(request)
	}

	pub fn block_hash<Block: BlockT + DeserializeOwned>(
		&self,
		block_number: Option<<Block::Header as HeaderT>::Number>,
	) -> Result<Option<Block::Hash>, jsonrpsee::core::ClientError> {
		let request = &|| {
			substrate_rpc_client::ChainApi::<
				<Block::Header as HeaderT>::Number,
				Block::Hash,
				Block::Header,
				SignedBlock<Block>,
			>::block_hash(
				&self.http_client,
				block_number.map(|n| ListOrValue::Value(NumberOrHex::Hex(n.into()))),
			)
		};

		self.block_on(request).map(|ok| match ok {
			ListOrValue::List(v) => v.get(0).map_or(None, |some| *some),
			ListOrValue::Value(v) => v,
		})
	}

	pub fn header<Block: BlockT + DeserializeOwned>(
		&self,
		hash: Option<Block::Hash>,
	) -> Result<Option<Block::Header>, jsonrpsee::core::ClientError> {
		let request = &|| {
			substrate_rpc_client::ChainApi::<
				BlockNumber,
				Block::Hash,
				Block::Header,
				SignedBlock<Block>,
			>::header(&self.http_client, hash)
		};

		self.block_on(request)
	}

	pub fn storage_hash<
		Hash: 'static + Clone + Sync + Send + DeserializeOwned + sp_runtime::Serialize,
	>(
		&self,
		key: StorageKey,
		at: Option<Hash>,
	) -> Result<Option<Hash>, jsonrpsee::core::ClientError> {
		let request = &|| {
			substrate_rpc_client::StateApi::<Hash>::storage_hash(
				&self.http_client,
				key.clone(),
				at.clone(),
			)
		};

		self.block_on(request)
	}

	pub fn storage<
		Hash: 'static + Clone + Sync + Send + DeserializeOwned + sp_runtime::Serialize + core::fmt::Debug,
	>(
		&self,
		key: StorageKey,
		at: Option<Hash>,
	) -> Result<Option<StorageData>, jsonrpsee::core::ClientError> {
		let request = &|| {
			substrate_rpc_client::StateApi::<Hash>::storage(
				&self.http_client,
				key.clone(),
				at.clone(),
			)
		};

		self.block_on(request)
	}

	pub fn storage_keys_paged<
		Hash: 'static + Clone + Sync + Send + DeserializeOwned + sp_runtime::Serialize,
	>(
		&self,
		key: Option<StorageKey>,
		count: u32,
		start_key: Option<StorageKey>,
		at: Option<Hash>,
	) -> Result<Vec<sp_state_machine::StorageKey>, jsonrpsee::core::ClientError> {
		let request = &|| {
			substrate_rpc_client::StateApi::<Hash>::storage_keys_paged(
				&self.http_client,
				key.clone(),
				count.clone(),
				start_key.clone(),
				at.clone(),
			)
		};
		let result = self.block_on(request);

		match result {
			Ok(result) => Ok(result.iter().map(|item| item.0.clone()).collect()),
			Err(err) => Err(err),
		}
	}

	pub fn transaction_by_hash(
		&self,
		eth_transaction_hash: &H256,
	) -> Result<Option<fc_rpc_v2_api::types::Transaction>, jsonrpsee::core::ClientError> {
		let request = &|| {
			fc_rpc_v2_api::eth::EthTransactionApiClient::transaction_by_hash(
				&self.http_client,
				eth_transaction_hash.clone(),
			)
		};

		self.block_on(request)
	}

	pub fn block_by_hash(
		&self,
		eth_block_hash: &H256,
		full: bool,
	) -> Result<Option<fc_rpc_v2_api::types::Block>, jsonrpsee::core::ClientError> {
		let request = &|| {
			fc_rpc_v2_api::eth::EthBlockApiClient::block_by_hash(
				&self.http_client,
				eth_block_hash.clone(),
				full,
			)
		};

		self.block_on(request)
	}

	pub fn block_by_number(
		&self,
		block_number: fc_rpc_v2_api::types::BlockNumberOrTag,
		full: bool,
	) -> Result<Option<fc_rpc_v2_api::types::Block>, jsonrpsee::core::ClientError> {
		let request = &|| {
			fc_rpc_v2_api::eth::EthBlockApiClient::block_by_number(
				&self.http_client,
				block_number.clone(),
				full,
			)
		};

		self.block_on(request)
	}

	fn block_on<F, T, E>(&self, f: &dyn Fn() -> F) -> Result<T, E>
	where
		F: Future<Output = Result<T, E>>,
	{
		use tokio::runtime::Handle;

		let id = self.counter.fetch_add(1, Ordering::SeqCst);
		let start = std::time::Instant::now();

		tokio::task::block_in_place(move || {
			Handle::current().block_on(async move {
				let delay_between_requests =
					Duration::from_millis(self.delay_between_requests_ms.into());

				let start_req = std::time::Instant::now();
				log::debug!(
					target: super::LAZY_LOADING_LOG_TARGET,
					"Sending request: {}",
					id
				);

				// Explicit request delay, to avoid getting 429 errors
				let _ = tokio::time::sleep(delay_between_requests).await;

				// Retry request in case of failure
				// The maximum number of retries is specified by `self.max_retries_per_request`
				let retry_strategy = FixedInterval::new(delay_between_requests)
					.take(self.max_retries_per_request as usize);
				let result = Retry::spawn(retry_strategy, f).await;

				log::debug!(
					target: super::LAZY_LOADING_LOG_TARGET,
					"Completed request (id: {}, successful: {}, elapsed_time: {:?}, query_time: {:?})",
					id,
					result.is_ok(),
					start.elapsed(),
					start_req.elapsed()
				);

				result
			})
		})
	}
}
