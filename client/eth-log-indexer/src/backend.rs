use fc_rpc::{frontier_backend_client, OverrideHandle};
use sc_client_api::backend::{Backend as BackendT, StateBackend, StorageProvider};
use sp_blockchain::HeaderBackend;
use sp_core::H256;
use sp_runtime::{
	generic::BlockId,
	traits::{BlakeTwo256, Block as BlockT, UniqueSaturatedInto},
};
use sqlx::{
	sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions, SqliteQueryResult, SqliteRow},
	ConnectOptions, Error, QueryBuilder, Row, Sqlite,
};
use std::str::FromStr;
use std::sync::Arc;

struct Log {
	block_number: i32,
	address: Vec<u8>,
	topic_1: Vec<u8>,
	topic_2: Vec<u8>,
	topic_3: Vec<u8>,
	topic_4: Vec<u8>,
	log_index: i32,
	transaction_index: i32,
	substrate_block_hash: Vec<u8>,
}

pub struct SqliteBackendConfig {
	pub path: &'static str,
	pub create_if_missing: bool,
}

pub enum BackendConfig {
	Sqlite(SqliteBackendConfig),
}

pub struct Backend<Client, Block: BlockT, BE> {
	pool: SqlitePool,
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
	pub async fn new(
		config: BackendConfig,
		pool_size: u32,
		client: Arc<Client>,
		overrides: Arc<OverrideHandle<Block>>,
	) -> Result<Self, Error> {
		let any_pool = SqlitePoolOptions::new()
			.max_connections(pool_size)
			.connect_lazy_with(
				Self::connect_options(&config)?
					.disable_statement_logging()
					.clone(),
			);
		let _ = Self::create_if_not_exists(&any_pool).await?;
		Ok(Self {
			pool: any_pool,
			client,
			overrides,
			_marker: Default::default(),
		})
	}

	fn connect_options(config: &BackendConfig) -> Result<SqliteConnectOptions, Error> {
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

	pub fn client(&self) -> Arc<Client> {
		self.client.clone()
	}

	pub async fn insert_sync_status(&self, hashes: &Vec<H256>) -> Result<SqliteQueryResult, Error> {
		let mut builder: QueryBuilder<Sqlite> =
			QueryBuilder::new("INSERT INTO sync_status(substrate_block_hash) ");
		builder.push_values(hashes, |mut b, hash| {
			b.push_bind(hash.as_bytes());
		});
		let query = builder.build();
		query.execute(self.pool()).await
	}

	pub fn spawn_logs_task(&self) {
		let pool = self.pool().clone();
		let client = self.client.clone();
		let overrides = self.overrides.clone();
		tokio::task::spawn(async move {
			let _ = async {
				// The overarching update statement, returning the substrate block hashes for this batch.
				match sqlx::query(
					"UPDATE sync_status
                    SET status = 1
                    WHERE substrate_block_hash IN
                        (SELECT substrate_block_hash
                         FROM sync_status
                         WHERE status = 0
                         LIMIT 1000) RETURNING substrate_block_hash",
				)
				.fetch_all(&pool)
				.await
				{
					// TODO important on error we need to rollback the sync_status changes.
					Ok(result) => {
						let logs = tokio::task::spawn_blocking(move || {
							Self::spawn_logs_task_inner(client, overrides, result)
						})
						.await
						.map_err(|_| Error::Protocol("tokio blocking task failed".to_string()))?;
						let mut tx = pool.begin().await.unwrap();
						for log in logs.iter() {
							let _ = sqlx::query!(
								"INSERT INTO logs(
							        block_number,
							        address,
							        topic_1,
							        topic_2,
							        topic_3,
							        topic_4,
							        log_index,
							        transaction_index,
							        substrate_block_hash)
							    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
								log.block_number,
								log.address,
								log.topic_1,
								log.topic_2,
								log.topic_3,
								log.topic_4,
								log.log_index,
								log.transaction_index,
								log.substrate_block_hash,
							)
							.execute(&mut tx)
							.await?;
						}
						tx.commit().await
					}
					Err(_) => Err(Error::Protocol(
						"Db locked in UPDATE `sync_status` failed, will try again".to_string(),
					)),
				}
			}
			.await
			.map_err(|e| println!("Indexer error: {:?}", e));
		});
	}

	fn spawn_logs_task_inner(
		client: Arc<Client>,
		overrides: Arc<OverrideHandle<Block>>,
		rows: Vec<SqliteRow>,
	) -> Vec<Log> {
		let mut logs: Vec<Log> = vec![];
		for row in rows.iter() {
			if let Ok(bytes) = row.try_get::<Vec<u8>, _>(0) {
				let substrate_block_hash = H256::from_slice(&bytes[..]);
				let substrate_block_number: i32 =
					if let Ok(Some(number)) = client.number(substrate_block_hash) {
						UniqueSaturatedInto::<u32>::unique_saturated_into(number) as i32
					} else {
						// TODO either log error or have a error table?
						0i32
					};
				let id = BlockId::Hash(substrate_block_hash);
				let schema = frontier_backend_client::onchain_storage_schema::<Block, Client, BE>(
					client.as_ref(),
					id,
				);
				let handler = overrides
					.schemas
					.get(&schema)
					.unwrap_or(&overrides.fallback);

				let receipts = handler.current_receipts(&id).unwrap_or_default();

				for (transaction_index, receipt) in receipts.iter().enumerate() {
					let receipt_logs = match receipt {
						ethereum::ReceiptV3::Legacy(d)
						| ethereum::ReceiptV3::EIP2930(d)
						| ethereum::ReceiptV3::EIP1559(d) => &d.logs,
					};
					let transaction_index = transaction_index as i32;
					for (log_index, log) in receipt_logs.iter().enumerate() {
						logs.push(Log {
							block_number: substrate_block_number,
							address: log.address.as_bytes().to_owned(),
							topic_1: log
								.topics
								.get(0)
								.unwrap_or(&H256::zero())
								.as_bytes()
								.to_owned(),
							topic_2: log
								.topics
								.get(1)
								.unwrap_or(&H256::zero())
								.as_bytes()
								.to_owned(),
							topic_3: log
								.topics
								.get(2)
								.unwrap_or(&H256::zero())
								.as_bytes()
								.to_owned(),
							topic_4: log
								.topics
								.get(3)
								.unwrap_or(&H256::zero())
								.as_bytes()
								.to_owned(),
							log_index: log_index as i32,
							transaction_index,
							substrate_block_hash: bytes.clone(),
						});
					}
				}
			} else {
				// TODO either log error or have a error table?
				// If we want to track errors in db maybe sync_status table needs an id column
				// and log/error on the id for cases like hash IS NULL.
			}
		}
		logs
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
                log_index INTEGER NOT NULL,
                transaction_index INTEGER NOT NULL,
                substrate_block_hash BLOB NOT NULL,
				UNIQUE (
                    log_index,
                    transaction_index,
                    substrate_block_hash
                )
            );
            CREATE TABLE IF NOT EXISTS sync_status (
                id INTEGER PRIMARY KEY,
                substrate_block_hash BLOB NOT NULL,
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
            COMMIT;",
		)
		.execute(pool)
		.await
	}
}
