use std::{sync::Arc, time::Duration};
use futures::prelude::*;
use sp_blockchain::{Backend, HeaderBackend};
use sp_runtime::{generic::BlockId, traits::{BlakeTwo256, Block as BlockT}};
use sp_api::HeaderT;
use sqlx::Row;
use sp_core::H256;
use sc_client_api::backend::{Backend as BackendT, StateBackend, StorageProvider};

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
            let mut foo_db: Vec<Block::Hash> = vec![];

            let interval_fut = futures_timer::Delay::new(interval);
            let backend = substrate_backend.blockchain();
            let notifications = notifications.fuse();
		    futures::pin_mut!(interval_fut, notifications);

            let mut db_status_data = sqlx::query("SELECT substrate_block_hash FROM sync_status WHERE status = 0")
                .fetch_all(indexer_backend.pool())
                .await
                .expect("query `sync_status` table")
                .iter()
                .map(|any_row| H256::from_slice(&any_row.try_get::<Vec<u8>,_>(0).unwrap()[..]))
                .collect::<Vec<H256>>();

            loop {
                futures::select! {
                    _ = (&mut interval_fut).fuse() => {
                        println!("##################################################################");
                        let leaves = backend.leaves();
                        if let Ok(mut leaves) = leaves {
                            while let Some(leaf) = leaves.pop() {
                                if !Self::batch(Arc::clone(&indexer_backend), &mut hashes, &mut db_status_data, leaf).await {
                                    break;
                                }
                                if let Ok(Some(header)) = backend.header(BlockId::Hash(leaf)) {
                                    let parent_hash = header.parent_hash();
                                    leaves.push(*parent_hash);
                                }
                            }
                        }
                        interval_fut.reset(interval);
                    },
                    imported = notifications.next() => {
                        println!("--> Whatever");
                    }
                }
            }
        }
    }

    pub async fn batch(indexer_backend: Arc<crate::Backend<Client, Block, Backend>>, hashes: &mut Vec<Block::Hash>, db_status_data: &mut Vec<Block::Hash>, hash: Block::Hash) -> bool {
        if hashes.contains(&hash) || db_status_data.contains(&hash) {
            println!("XXXXXXXXXXXXXXXXX CONTAINS");
            false
        } else if hashes.len() < BATCH_SIZE {
            hashes.push(hash);
            true
        } else {
            hashes.push(hash);
            indexer_backend.insert_sync_status(hashes).await; // TODO handle err
            // Spawn actual logs task
            indexer_backend.insert_logs();
            db_status_data.append(hashes);
            println!("!!!!!!!!! DB has now {:?} block hashes", db_status_data.len());
            println!("!!!!!!!!!");
            hashes.clear();
            true
        }
    }
}
