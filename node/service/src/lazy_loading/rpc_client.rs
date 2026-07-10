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

use super::state_cache::StateCache;
use cumulus_primitives_core::BlockT;
use fc_rpc_v2_api::types::H256;
use jsonrpsee::http_client::HttpClient;
use moonbeam_core_primitives::BlockNumber;
use serde::de::DeserializeOwned;
use serde::Serialize;
use sp_api::__private::HeaderT;
use sp_rpc::list::ListOrValue;
use sp_rpc::number::NumberOrHex;
use sp_runtime::generic::SignedBlock;
use sp_storage::{StorageData, StorageKey};
use std::future::Future;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio_retry::strategy::{jitter, ExponentialBackoff};
use tokio_retry::Retry;

/// Upper bound for a single retry back-off delay. Keeps retries from stalling
/// indefinitely while still giving a rate-limited endpoint time to recover.
const MAX_RETRY_BACKOFF: Duration = Duration::from_secs(30);
/// Floor for the exponential back-off base so that even a tiny configured
/// `delay_between_requests` still produces a meaningful back-off between retries.
const MIN_RETRY_BACKOFF_BASE_MS: u64 = 100;

#[derive(Debug, Clone)]
pub struct RPC {
	http_client: HttpClient,
	delay_between_requests_ms: u32,
	max_retries_per_request: u32,
	counter: Arc<AtomicU64>,
	cache: Option<Arc<StateCache>>,
}

impl RPC {
	pub fn new(
		http_client: HttpClient,
		delay_between_requests_ms: u32,
		max_retries_per_request: u32,
		cache: Option<Arc<StateCache>>,
	) -> Self {
		Self {
			http_client,
			delay_between_requests_ms,
			max_retries_per_request,
			counter: Default::default(),
			cache,
		}
	}

	/// Serve `request` from the on-disk cache when possible, otherwise perform it
	/// over the network and persist a successful response.
	///
	/// Only call this for requests whose result is immutable for the given
	/// `cache_key` (i.e. reads pinned to a concrete block hash/number). When no
	/// cache is configured, or `cache_key` is `None`, this simply forwards to the
	/// network with the usual retry behaviour.
	fn cached_request<F, T>(
		&self,
		namespace: &str,
		cache_key: Option<Vec<u8>>,
		request: &dyn Fn() -> F,
	) -> Result<T, jsonrpsee::core::ClientError>
	where
		F: Future<Output = Result<T, jsonrpsee::core::ClientError>>,
		T: Serialize + DeserializeOwned,
	{
		if let (Some(cache), Some(key)) = (self.cache.as_ref(), cache_key.as_ref()) {
			if let Some(bytes) = cache.get(namespace, key) {
				if let Ok(value) = serde_json::from_slice::<T>(&bytes) {
					return Ok(value);
				}
			}
		}

		let result = self.block_on(request)?;

		if let (Some(cache), Some(key)) = (self.cache.as_ref(), cache_key.as_ref()) {
			if let Ok(bytes) = serde_json::to_vec(&result) {
				cache.put(namespace, key, &bytes);
			}
		}

		Ok(result)
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

		let cache_key = at
			.as_ref()
			.and_then(|at| make_cache_key(at, &[key.0.as_slice()]));
		self.cached_request("state_storage_hash", cache_key, request)
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

		let cache_key = at
			.as_ref()
			.and_then(|at| make_cache_key(at, &[key.0.as_slice()]));
		self.cached_request("state_storage", cache_key, request)
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

		let cache_key = at.as_ref().and_then(|at| {
			make_cache_key(
				at,
				&[
					key.as_ref().map(|k| k.0.as_slice()).unwrap_or(&[]),
					&count.to_le_bytes(),
					start_key.as_ref().map(|k| k.0.as_slice()).unwrap_or(&[]),
				],
			)
		});
		let result: Vec<StorageKey> =
			self.cached_request("state_storage_keys_paged", cache_key, request)?;

		Ok(result.iter().map(|item| item.0.clone()).collect())
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

				// Retry request in case of failure. The public fork endpoint
				// (e.g. trace.api.moonbeam.network) can intermittently rate-limit
				// (HTTP 429), time out, or drop connections under CI load. A fixed
				// tiny interval would burn every retry within a few milliseconds
				// without giving the endpoint time to recover, and retrying in
				// lockstep across concurrent requests causes a thundering herd.
				// Use exponential back-off with jitter instead, capped so a single
				// request never stalls for too long. The maximum number of retries
				// is specified by `self.max_retries_per_request`.
				let backoff_base_ms =
					(self.delay_between_requests_ms as u64).max(MIN_RETRY_BACKOFF_BASE_MS);
				let retry_strategy = ExponentialBackoff::from_millis(2)
					.factor(backoff_base_ms)
					.max_delay(MAX_RETRY_BACKOFF)
					.map(jitter)
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

/// Build a collision-resistant cache key from the block selector `at` and a set
/// of additional parts (e.g. the storage key and paging parameters). Each part
/// is length-prefixed so different argument combinations can never alias. Returns
/// `None` if the selector cannot be serialized, in which case the caller should
/// skip caching and go straight to the network.
fn make_cache_key<Hash: Serialize>(at: &Hash, parts: &[&[u8]]) -> Option<Vec<u8>> {
	let at_bytes = serde_json::to_vec(at).ok()?;
	let mut key = Vec::new();
	key.extend_from_slice(&(at_bytes.len() as u32).to_le_bytes());
	key.extend_from_slice(&at_bytes);
	for part in parts {
		key.extend_from_slice(&(part.len() as u32).to_le_bytes());
		key.extend_from_slice(part);
	}
	Some(key)
}
