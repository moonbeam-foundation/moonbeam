use futures::prelude::*;
use sc_client_api::backend::{Backend as BackendT, StateBackend, StorageProvider};
use sp_api::HeaderT;
use sp_blockchain::{Backend, HeaderBackend};
use sp_core::H256;
use sp_runtime::{
	generic::BlockId,
	traits::{BlakeTwo256, Block as BlockT},
};
use sqlx::Row;
use std::{sync::Arc, time::Duration};

pub struct SyncWorker<Block, Backend, Client>(std::marker::PhantomData<(Block, Backend, Client)>);
impl<Block: BlockT, Backend, Client> SyncWorker<Block, Backend, Client>
where
	Block: BlockT<Hash = H256> + Send + Sync,
	Client: StorageProvider<Block, Backend> + HeaderBackend<Block> + Send + Sync + 'static,
	Backend: BackendT<Block> + 'static,
	Backend::State: StateBackend<BlakeTwo256>,
{
	pub async fn run(
		substrate_backend: Arc<Backend>,
		indexer_backend: Arc<crate::Backend<Client, Block, Backend>>,
		notifications: sc_client_api::ImportNotifications<Block>,
		batch_size: usize,
		interval: Duration,
	) {
		let mut current_batch: Vec<Block::Hash> = vec![];

		let import_interval = futures_timer::Delay::new(interval);
		let backend = substrate_backend.blockchain();
		let notifications = notifications.fuse();

		let mut known_hashes =
			sqlx::query("SELECT substrate_block_hash FROM sync_status ORDER BY id ASC")
				.fetch_all(indexer_backend.pool())
				.await
				.expect("query `sync_status` table")
				.iter()
				.map(|any_row| {
					H256::from_slice(&any_row.try_get::<Vec<u8>, _>(0).unwrap_or_default()[..])
				})
				.collect::<Vec<H256>>();

		let mut resume_at: Option<H256> = None;
		if let Some(hash) = known_hashes.last() {
			let client = indexer_backend.client();
			if let Ok(Some(number)) = client.number(*hash) {
				if let Ok(Some(header)) =
					client.header(sp_runtime::generic::BlockId::Number(number))
				{
					resume_at = Some(*header.parent_hash())
				}
			}
		}

		futures::pin_mut!(import_interval, notifications);
		loop {
			futures::select! {
				_ = (&mut import_interval).fuse() => {
					println!("#############");
					let leaves = backend.leaves();
					if let Ok(mut leaves) = leaves {
						if let Some(hash) = resume_at {
							println!("--> resuming syncing operation at {:?}", hash);
							leaves.push(hash);
							resume_at = None;
						}
						while let Some(leaf) = leaves.pop() {
							if !Self::batch(
								Arc::clone(&indexer_backend),
								batch_size,
								&mut current_batch,
								&mut known_hashes,
								leaf,
								false
							).await {
								break;
							}
							if let Ok(Some(header)) = backend.header(BlockId::Hash(leaf)) {
								let parent_hash = header.parent_hash();
								leaves.push(*parent_hash);
							}
						}
					}
					import_interval.reset(interval);
				},
				notification = notifications.next() => if let Some(notification) = notification {
					let _ = Self::batch(
						Arc::clone(&indexer_backend),
						batch_size,
						&mut current_batch,
						&mut known_hashes,
						notification.hash,
						true
					).await;
				}
			}
		}
	}

	pub async fn batch(
		indexer_backend: Arc<crate::Backend<Client, Block, Backend>>,
		batch_size: usize,
		current_batch: &mut Vec<Block::Hash>,
		known_hashes: &mut Vec<Block::Hash>,
		hash: Block::Hash,
		notified: bool,
	) -> bool {
		if !current_batch.contains(&hash) && !known_hashes.contains(&hash) {
			known_hashes.push(hash);
			if !notified && current_batch.len() < batch_size {
				println!("--> added {:?}", hash);
				current_batch.push(hash);
			} else {
				println!("--> batch");
				current_batch.push(hash);
				let _ = indexer_backend
					.insert_sync_status(current_batch)
					.await
					.map_err(|e| {
						log::error!(
							target: "eth-log-indexer",
							"{}",
							e,
						);
					});
				indexer_backend.spawn_logs_task(batch_size); // Spawn actual logs task
				current_batch.clear();
			}
			return true;
		}
		println!("--> exists {:?}", hash);
		false
	}
}

#[cfg(test)]
mod test {
	use codec::Encode;
	use fc_rpc::{
		EthBlockDataCacheTask, EthTask, OverrideHandle, RuntimeApiStorageOverride,
		SchemaV1Override, SchemaV2Override, SchemaV3Override, StorageOverride,
	};
	use fp_storage::{
		EthereumStorageSchema, ETHEREUM_CURRENT_RECEIPTS, PALLET_ETHEREUM, PALLET_ETHEREUM_SCHEMA,
	};
	use futures::{executor, prelude::*};
	use moonbase_runtime::RuntimeApi;
	use sc_block_builder::BlockBuilderProvider;
	use sc_client_api::BlockchainEvents;
	use sp_consensus::BlockOrigin;
	use sp_core::traits::SpawnEssentialNamed;
	use sp_core::{H160, H256, U256};
	use sp_io::hashing::{blake2_128, twox_128};
	use std::collections::BTreeMap;
	use std::path::Path;
	use std::sync::Arc;
	use substrate_test_runtime_client::{
		prelude::*, DefaultTestClientBuilderExt, TestClientBuilder, TestClientBuilderExt,
	};
	use tempfile::tempdir;

	fn storage_prefix_build(module: &[u8], storage: &[u8]) -> Vec<u8> {
		[twox_128(module), twox_128(storage)].concat().to_vec()
	}

	#[tokio::test]
	async fn interval_indexing_works() {
		let tmp = tempdir().expect("create a temporary directory");
		// Initialize storage with schema V1.
		let builder = TestClientBuilder::new().add_extra_storage(
			PALLET_ETHEREUM_SCHEMA.to_vec(),
			Encode::encode(&EthereumStorageSchema::V3),
		);
		// Backend
		let backend = builder.backend();
		// Client
		let (client, _) = builder.build_with_native_executor::<RuntimeApi, _>(None);
		let mut client = Arc::new(client);
		// Overrides
		let mut overrides_map = BTreeMap::new();
		overrides_map.insert(
			EthereumStorageSchema::V3,
			Box::new(SchemaV3Override::new(client.clone()))
				as Box<dyn StorageOverride<_> + Send + Sync>,
		);
		let overrides = Arc::new(OverrideHandle {
			schemas: overrides_map,
			fallback: Box::new(RuntimeApiStorageOverride::new(client.clone())),
		});
		// Indexer backend
		let indexer_backend = crate::Backend::new(
			crate::BackendConfig::Sqlite(crate::SqliteBackendConfig {
				path: Path::new("sqlite:///")
					.join(tmp.path().strip_prefix("/").unwrap().to_str().unwrap())
					.join("test2.db3")
					.to_str()
					.unwrap(),
				create_if_missing: true,
			}),
			100,
			client.clone(),
			overrides.clone(),
		)
		.await
		.expect("indexer pool to be created");
		// Create blocks
		for nonce in [1, 2, 3].into_iter() {
			let mut builder = client.new_block(Default::default()).unwrap();
			let receipts = Encode::encode(&vec![ethereum::ReceiptV3::EIP1559(
				ethereum::EIP1559ReceiptData {
					status_code: 0u8,
					used_gas: U256::zero(),
					logs_bloom: ethereum_types::Bloom::zero(),
					logs: vec![ethereum::Log {
						address: H160::random(),
						topics: vec![H256::random()],
						data: vec![],
					}],
				},
			)]);
			builder
				.push_storage_change(
					storage_prefix_build(PALLET_ETHEREUM, ETHEREUM_CURRENT_RECEIPTS),
					Some(receipts),
				)
				.unwrap();
			let block = builder.build().unwrap().block;
			executor::block_on(client.import(BlockOrigin::Own, block)).unwrap();
		}

		// Spawn worker after creating the blocks will resolve the interval future.
		// Because the SyncWorker is spawned at service level, in the real world this will only
		// happen when we are in major syncing.
		tokio::task::spawn(async move {
			crate::SyncWorker::run(
				backend.clone(),
				Arc::new(indexer_backend),
				client.clone().import_notification_stream(),
				4,                                 // batch size
				std::time::Duration::from_secs(1), // interval duration
			)
			.await
		});

		futures_timer::Delay::new(std::time::Duration::from_secs(2)).await
	}
}
