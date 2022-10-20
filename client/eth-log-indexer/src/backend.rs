use fc_rpc::{frontier_backend_client, OverrideHandle};
use sc_client_api::backend::{Backend as BackendT, StateBackend, StorageProvider};
use sp_blockchain::HeaderBackend;
use sp_core::H256;
use sp_runtime::{
	generic::BlockId,
	traits::{BlakeTwo256, Block as BlockT, UniqueSaturatedInto},
};
use sqlx::{
	sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions, SqliteQueryResult},
	ConnectOptions, Error, QueryBuilder, Row, Sqlite,
};
use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug, Eq, PartialEq)]
pub struct Log {
	pub block_number: i32,
	pub address: Vec<u8>,
	pub topic_1: Vec<u8>,
	pub topic_2: Vec<u8>,
	pub topic_3: Vec<u8>,
	pub topic_4: Vec<u8>,
	pub log_index: i32,
	pub transaction_index: i32,
	pub substrate_block_hash: Vec<u8>,
}

pub struct SqliteBackendConfig<'a> {
	pub path: &'a str,
	pub create_if_missing: bool,
}

pub enum BackendConfig<'a> {
	Sqlite(SqliteBackendConfig<'a>),
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
		config: BackendConfig<'_>,
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

	pub(crate) async fn insert_sync_status(&self, hashes: &Vec<H256>) -> Result<SqliteQueryResult, Error> {
		let mut builder: QueryBuilder<Sqlite> =
			QueryBuilder::new("INSERT INTO sync_status(substrate_block_hash) ");
		builder.push_values(hashes, |mut b, hash| {
			b.push_bind(hash.as_bytes());
		});
		let query = builder.build();
		query.execute(self.pool()).await
	}

	pub(crate) fn spawn_logs_task(&self, batch_size: usize) {
		let pool = self.pool().clone();
		let client = self.client.clone();
		let overrides = self.overrides.clone();
		tokio::task::spawn(async move {
			let mut to_index: Vec<H256> = vec![];
			let _ = async {
				// The overarching update statement, returning the substrate block hashes for this batch.
				// If the task fails after the update, the returned block hashes will be used to try to
				// rollback to status 0.
				let q = format!(
					"UPDATE sync_status
                    SET status = 1
                    WHERE substrate_block_hash IN
                        (SELECT substrate_block_hash
                         FROM sync_status
                         WHERE status = 0
                         LIMIT {}) RETURNING substrate_block_hash",
					batch_size
				);
				match sqlx::query(&q).fetch_all(&pool).await {
					Ok(result) => {
						for row in result.iter() {
							if let Ok(bytes) = row.try_get::<Vec<u8>, _>(0) {
								to_index.push(H256::from_slice(&bytes[..]));
							} else {
								log::error!(
									target: "eth-log-indexer",
									"unable to get row value"
								);
							}
						}
						let to_index = Arc::new(to_index);
						let to_index_inner = to_index.clone();
						// Spawn a blocking task to get log data from substrate backend.
						let logs = tokio::task::spawn_blocking(move || {
							Self::spawn_logs_task_inner(client, overrides, to_index_inner)
						})
						.await
						.map_err(|_| {
							(
								to_index.clone(),
								Error::Protocol("tokio blocking task failed".to_string()),
							)
						})?;
						let mut tx = pool.begin().await.map_err(|e| (to_index.clone(), e))?;
						// TODO VERIFY statements limit per transaction in sqlite if any  
						for log in logs.iter() {
							let _ = sqlx::query!(
								"INSERT OR IGNORE INTO logs(
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
							.await
							.map_err(|e| (to_index.clone(), e))?;
						}
						tx.commit().await.map_err(|e| (to_index.clone(), e))
					}
					Err(e) => Err((Arc::new(vec![]), e)),
				}
			}
			.await
			.map_err(|(to_rollback, e)| {
				// On error try to rollback `sync_status` table
				tokio::task::spawn(async move {
					let err = |e, to_rollback| {
						log::error!(
							target: "eth-log-indexer",
							"Tried to rollback but failed {:?}. Inconsistent database for {:?}",
							e,
							to_rollback
						);
					};
					let mut tx = pool
						.begin()
						.await
						.map_err(|e| err(e, &to_rollback))
						.unwrap();
					for hash in to_rollback.iter() {
						let hash = hash.as_bytes().to_owned();
						let _ = sqlx::query!(
							"UPDATE sync_status
							SET status = 0
							WHERE substrate_block_hash = ?",
							hash,
						)
						.execute(&mut tx)
						.await
						.map_err(|e| err(e, &to_rollback));
					}
					let _ = tx.commit().await.map_err(|e| err(e, &to_rollback));
				});
				log::error!(
					target: "eth-log-indexer",
					"{}",
					e
				);
			});
		});
	}

	fn spawn_logs_task_inner(
		client: Arc<Client>,
		overrides: Arc<OverrideHandle<Block>>,
		hashes: Arc<Vec<H256>>,
	) -> Vec<Log> {
		let mut logs: Vec<Log> = vec![];
		for substrate_block_hash in hashes.iter() {
			let substrate_block_number: i32 =
				if let Ok(Some(number)) = client.number(*substrate_block_hash) {
					UniqueSaturatedInto::<u32>::unique_saturated_into(number) as i32
				} else {
					log::error!(
						target: "eth-log-indexer",
						"Cannot find number for substrate hash {}",
						substrate_block_hash
					);
					0i32
				};
			let id = BlockId::Hash(*substrate_block_hash);
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
						substrate_block_hash: substrate_block_hash.as_bytes().to_owned(),
					});
				}
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
                status INTEGER DEFAULT 0 NOT NULL,
				UNIQUE (
                    substrate_block_hash
                )
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
