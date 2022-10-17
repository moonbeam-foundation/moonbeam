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

const BATCH_SIZE: usize = 1000;

pub struct SyncWorker<Block, Backend, Client>(std::marker::PhantomData<(Block, Backend, Client)>);
impl<Block: BlockT, Backend, Client> SyncWorker<Block, Backend, Client>
where
	Block: BlockT<Hash = H256> + Send + Sync,
	Client: StorageProvider<Block, Backend> + HeaderBackend<Block> + Send + Sync + 'static,
	Backend: BackendT<Block> + 'static,
	Backend::State: StateBackend<BlakeTwo256>,
{
	pub fn run(
		substrate_backend: Arc<Backend>,
		indexer_backend: Arc<crate::Backend<Client, Block, Backend>>,
		notifications: sc_client_api::ImportNotifications<Block>,
		interval: Duration,
	) -> impl Future<Output = ()> {
		async move {
			let mut hashes: Vec<Block::Hash> = vec![];

			let import_interval = futures_timer::Delay::new(interval);
			let backend = substrate_backend.blockchain();
			let notifications = notifications.fuse();

			let existing = sqlx::query(
				"SELECT substrate_block_hash FROM sync_status ORDER BY id DESC LIMIT 1",
			)
			.fetch_one(indexer_backend.pool())
			.await;

			futures::pin_mut!(import_interval, notifications);
			loop {
				futures::select! {
					_ = (&mut import_interval).fuse() => {
						println!("##################################################################");
						let leaves = backend.leaves();
						if let Ok(mut leaves) = leaves {
							if let Ok(ref row) = existing {
								let client = indexer_backend.client();
								let hash = H256::from_slice(&row.try_get::<Vec<u8>, _>(0).unwrap_or_default()[..]);
								if let Ok(Some(number)) = client.number(hash) {
									if let Ok(Some(header)) = client.header(sp_runtime::generic::BlockId::Number(number)) {
										println!("--> resuming syncing operation at {:?}", header.parent_hash());
										leaves.push(*header.parent_hash());
									}
								}
							}
							while let Some(leaf) = leaves.pop() {
								if !Self::batch(Arc::clone(&indexer_backend), &mut hashes, leaf, false).await {
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
						let _ = Self::batch(Arc::clone(&indexer_backend), &mut hashes, notification.hash, true).await;
					}
				}
			}
		}
	}

	pub async fn batch(
		indexer_backend: Arc<crate::Backend<Client, Block, Backend>>,
		hashes: &mut Vec<Block::Hash>,
		hash: Block::Hash,
		notified: bool,
	) -> bool {
		let bytes = hash.as_bytes();
		let already_synced = sqlx::query!(
			"SELECT id FROM sync_status WHERE substrate_block_hash = ?1",
			bytes
		)
		.fetch_one(indexer_backend.pool())
		.await;
		if already_synced.is_ok() {
			println!("XX already synced {:?}", hash);
			return false;
		}
		if !hashes.contains(&hash) {
			if !notified && hashes.len() < BATCH_SIZE {
				hashes.push(hash);
			} else {
				println!("--> batch");
				hashes.push(hash);
				let _ = indexer_backend.insert_sync_status(hashes).await; // TODO handle err
				indexer_backend.spawn_logs_task(); // Spawn actual logs task
				hashes.clear();
			}
		}
		true
	}
}
