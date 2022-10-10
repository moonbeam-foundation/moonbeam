use sqlx::{Row, Database as DatabaseT, Error, sqlite::{SqlitePool, SqlitePoolOptions, SqliteConnectOptions, SqliteQueryResult}, QueryBuilder, Sqlite, ConnectOptions};
use std::str::FromStr;
use sp_runtime::{generic::BlockId, traits::{BlakeTwo256, Block as BlockT, UniqueSaturatedInto}};
use sp_blockchain::HeaderBackend;
use sc_client_api::backend::{Backend as BackendT, StateBackend, StorageProvider};
use fc_rpc::{frontier_backend_client, OverrideHandle};
use std::sync::Arc;
use sp_core::H256;

// On sqlx::any:
//
// sqlx allows to be generic over the created pool using the Any types, and automatically infers the
// backend parsing the connection options.
//
// https://github.com/launchbadge/sqlx/issues/1978 describes a lifetime issue when using an AnyPool + QueryBuilder,
// which in practice renders useless the Any type for complex builder queries.
//
// Until that issue is resolved we just use the Sqlite as our primary target namespace, although our design supports effortlessly switching to Any.

struct Log {
    block_number: i32,
    address: Vec<u8>,
    topic_1: Vec<u8>,
    topic_2: Vec<u8>,
    topic_3: Vec<u8>,
    topic_4: Vec<u8>,
    transaction_index: i32,
    substrate_block_hash: Vec<u8>
}

pub struct SqliteBackendConfig {
    pub path: &'static str,
    pub create_if_missing: bool,
}

pub enum BackendConfig {
    Sqlite(SqliteBackendConfig)
}

pub struct Backend<Client, Block: BlockT, BE> {
    pool: SqlitePool,
    config: BackendConfig,
    client: Arc<Client>,
    overrides: Arc<OverrideHandle<Block>>,
    _marker: std::marker::PhantomData<BE>,
}
impl<Client, Block: BlockT, BE> Backend<Client, Block, BE>
where
	Block: BlockT<Hash = H256> + Send + Sync,
	Client: StorageProvider<Block, BE> + HeaderBackend<Block> + Send + Sync + 'static,
	BE: BackendT<Block> + 'static,
	BE::State: StateBackend<BlakeTwo256>,
{
    pub async fn new(config: BackendConfig, pool_size: u32, client: Arc<Client>, overrides: Arc<OverrideHandle<Block>>) -> Result<Self, Error> {
        let any_pool = SqlitePoolOptions::new()
            .max_connections(pool_size)
            .connect_lazy_with(Self::any_connect_options(&config)?.disable_statement_logging().clone());
        let _ = Self::create_if_not_exists(&any_pool).await?;
        Ok(Self { pool: any_pool, config, client, overrides, _marker: Default::default() })
    }

    fn any_connect_options(config: &BackendConfig) -> Result<SqliteConnectOptions, Error> {
        match config {
            BackendConfig::Sqlite(config) => {
                let config = sqlx::sqlite::SqliteConnectOptions::from_str(config.path)?
                    .create_if_missing(config.create_if_missing)
                    .into();
                Ok(config)
            }
        }
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    pub async fn insert_sync_status(&self, hashes: &Vec<H256>) -> Result<SqliteQueryResult, Error> {
        let mut builder: QueryBuilder<Sqlite> = QueryBuilder::new(
            "INSERT INTO sync_status(substrate_block_hash) "
        );
        builder.push_values(hashes, |mut b, hash| {
            b.push_bind(hash.as_bytes());
        });
        let query = builder.build();
        query.execute(self.pool()).await
    }

    pub fn insert_logs(&self) {
        let pool = self.pool().clone();
        let client = self.client.clone();
        let overrides = self.overrides.clone();
        tokio::task::spawn(async move {
            if let Ok(result) = sqlx::query(
                "UPDATE sync_status SET status = 1 WHERE substrate_block_hash IN (SELECT substrate_block_hash FROM sync_status WHERE status = 0 LIMIT 10) RETURNING substrate_block_hash"
            )
            .fetch_all(&pool)
            .await {
                // let mut builder: QueryBuilder<Sqlite> = QueryBuilder::new(
                //     "INSERT INTO logs(block_number, address, topic_1, topic_2, topic_3, topic_4, transaction_index, substrate_block_hash) "
                // );
                // builder.push_values(hashes, |mut b, hash| {
                //     b.push_bind(hash.as_bytes());
                // });
                // let query = builder.build();
                // query.execute(self.pool()).await
                let mut logs: Vec<Log> = vec![];
                for row in result.iter() {
                    if let Ok(bytes) = row.try_get::<Vec<u8>,_>(0) {
                        let substrate_block_hash = H256::from_slice(&bytes[..]);
                        let substrate_block_number: i32 = if let Ok(Some(number)) = client.number(substrate_block_hash) {
                            UniqueSaturatedInto::<u32>::unique_saturated_into(number) as i32
                        } else {
                            // TODO either log error or have a error table?
                            0i32
                        };
                        let id = BlockId::Hash(substrate_block_hash);
                        let schema = frontier_backend_client::onchain_storage_schema::<
                            Block,
                            Client,
                            BE,
                        >(client.as_ref(), id);
                        let handler = overrides
                            .schemas
                            .get(&schema)
                            .unwrap_or(&overrides.fallback);

                        let receipts = handler.current_receipts(&id).unwrap_or_default();

                        for (index, receipt) in receipts.iter().enumerate() {
                            let receipt_logs = match receipt {
                                ethereum::ReceiptV3::Legacy(d)
                                | ethereum::ReceiptV3::EIP2930(d)
                                | ethereum::ReceiptV3::EIP1559(d) => &d.logs,
                            };
                            for log in receipt_logs {
                                logs.push(Log {
                                    block_number: substrate_block_number,
                                    address: log.address.as_bytes().to_owned(),
                                    topic_1: log.topics.get(0).unwrap_or(&H256::zero()).as_bytes().to_owned(),
                                    topic_2: log.topics.get(1).unwrap_or(&H256::zero()).as_bytes().to_owned(),
                                    topic_3: log.topics.get(2).unwrap_or(&H256::zero()).as_bytes().to_owned(),
                                    topic_4: log.topics.get(3).unwrap_or(&H256::zero()).as_bytes().to_owned(),
                                    transaction_index: index as i32,
                                    substrate_block_hash: bytes.clone()
                                })
                            }
                        }
                    } else {
                        // TODO either log error or have a error table?
                        // If we want to track errors in db maybe sync_status table needs an id column
                        // and log/error on the id for cases like hash IS NULL.
                    }
                }

                let mut builder: QueryBuilder<Sqlite> = QueryBuilder::new(
                    "INSERT INTO logs(block_number, address, topic_1, topic_2, topic_3, topic_4, transaction_index, substrate_block_hash) "
                );
                builder.push_values(logs.iter(), |mut b, log| {
                    b.push_bind(log.block_number);
                    b.push_bind(&log.address[..]);
                    b.push_bind(&log.topic_1[..]);
                    b.push_bind(&log.topic_2[..]);
                    b.push_bind(&log.topic_3[..]);
                    b.push_bind(&log.topic_4[..]);
                    b.push_bind(log.transaction_index);
                    b.push_bind(&log.substrate_block_hash[..]);
                });
                let query = builder.build();
                let _ = query.execute(&pool).await;
                // let hashes = result.iter()
                //     .filter_map(|row| {
                //         if let Ok(hash) = row.try_get::<Vec<u8>,_>(0) {
                //             return Some(H256::from_slice(&hash[..]));
                //         } else {
                //             // TODO either log error or have a error table?
                //             // If we want to track errors maybe sync_status table needs an id column
                //             // and log/error on the id for cases like hash IS NULL.
                //             return None;
                //         }
                //     })
                //     .collect::<Vec<H256>>();
                
                // let id = BlockId::Hash(*substrate_block_hash);
            }
        });
        // let schema = frontier_backend_client::onchain_storage_schema::<
        //     B,
        //     C,
        //     BE,
        // >(client.as_ref(), id);
    }

    async fn create_if_not_exists(pool: &SqlitePool) -> Result<SqliteQueryResult, Error> {
        sqlx::query(
            "BEGIN;
            CREATE TABLE IF NOT EXISTS logs (
                id INTEGER PRIMARY KEY,
                block_number INTEGER NOT NULL,
                address BLOB NOT NULL,
                topic_1 BLOB NOT NULL,
                topic_2 BLOB NOT NULL,
                topic_3 BLOB NOT NULL,
                topic_4 BLOB NOT NULL,
                transaction_index INTEGER NOT NULL,
                substrate_block_hash BLOB NOT NULL,
                UNIQUE (
                    transaction_index,
                    substrate_block_hash
                )
            );
            CREATE TABLE IF NOT EXISTS sync_status (
                substrate_block_hash BLOB NOT NULL PRIMARY KEY,
                status INTEGER DEFAULT 0 NOT NULL
            );
            CREATE INDEX IF NOT EXISTS block_number_idx ON logs (
                block_number,
                address
            );
            CREATE INDEX IF NOT EXISTS topic_1_idx ON logs (
                block_number,
                topic_1
            );
            CREATE INDEX IF NOT EXISTS topic_2_idx ON logs (
                block_number,
                topic_2
            );
            CREATE INDEX IF NOT EXISTS topic_3_idx ON logs (
                block_number,
                topic_3
            );
            CREATE INDEX IF NOT EXISTS topic_4_idx ON logs (
                block_number,
                topic_4
            );
            COMMIT;"
        ).execute(pool).await
    }
}