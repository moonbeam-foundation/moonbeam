// Copyright 2019-2021 PureStake Inc.
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

use crate::parse;
use remote_externalities::{Mode, OfflineConfig, OnlineConfig, SnapshotConfig};
use rpc_client::RpcClient;
use sc_service::NativeExecutionDispatch;
use sp_core::offchain::testing::{TestOffchainExt, TestTransactionPoolExt};
use sp_core::offchain::{OffchainDbExt, OffchainWorkerExt, TransactionPoolExt};
use sp_core::storage::{well_known_keys, StorageData, StorageKey};
use sp_core::twox_128;
use sp_keystore::testing::KeyStore;
use sp_keystore::KeystoreExt;
use sp_runtime::traits::{Block as BlockT, Header as HeaderT, NumberFor};
use std::io::prelude::*;
use std::{fmt::Debug, path::PathBuf, str::FromStr};
use structopt::StructOpt;

const BLOCKS_PATH: &str = "snapshots/blocks/";
const STATES_PATH: &str = "snapshots/states/";

/// Command for replay blocks on live chain conditions
#[derive(Debug, StructOpt)]
pub struct ReplayBlocksCommand {
	#[structopt(flatten)]
	shared: sc_cli::SharedParams,

	/// Number of blocks to replay
	#[structopt(short, default_value = "1")]
	n: usize,

	/// The execution strategy that should be used for replay blocks
	#[structopt(
		long = "execution",
		value_name = "STRATEGY",
		possible_values = &sc_cli::ExecutionStrategy::variants(),
		case_insensitive = true,
		default_value = "Native",
	)]
	execution: sc_cli::ExecutionStrategy,

	/// Method for executing Wasm runtime code.
	#[structopt(
		long = "wasm-execution",
		value_name = "METHOD",
		possible_values = &sc_cli::WasmExecutionMethod::variants(),
		case_insensitive = true,
		default_value = "Compiled"
	)]
	wasm_method: sc_cli::WasmExecutionMethod,

	/// The number of 64KB pages to allocate for Wasm execution. Defaults to
	/// sc_service::Configuration.default_heap_pages.
	#[structopt(long)]
	pub heap_pages: Option<u64>,

	/// The fist block to replay.
	/// This can be a block number or a block hash.
	#[structopt(
		short,
		long,
		multiple = false,
		parse(try_from_str = parse::block_number_or_hash),
	)]
	from: crate::parse::BlockNumberOrHash,

	#[structopt(long, parse(from_os_str))]
	ouput_storage: Option<PathBuf>,

	/// The url to connect to.
	// TODO having this a shared parm is a temporary hack; the url is used just
	// to get the header/block. We should try and get that out of state, OR allow
	// the user to feed in a header/block via file.
	// https://github.com/paritytech/substrate/issues/9027
	#[structopt(short, long, default_value = "ws://localhost:9944", parse(try_from_str = parse::url))]
	url: String,
}

impl sc_cli::CliConfiguration for ReplayBlocksCommand {
	fn shared_params(&self) -> &sc_cli::SharedParams {
		&self.shared
	}

	fn chain_id(&self, _is_dev: bool) -> sc_cli::Result<String> {
		Ok(match self.shared.chain {
			Some(ref chain) => chain.clone(),
			None => "dev".into(),
		})
	}
}

impl ReplayBlocksCommand {
	pub async fn run<Block, ExecDispatch>(
		&self,
		config: sc_service::Configuration,
		wasm_code: Option<&'static [u8]>,
	) -> sc_cli::Result<()>
	where
		Block: BlockT + serde::de::DeserializeOwned,
		Block::Header: serde::de::DeserializeOwned,
		Block::Hash: FromStr,
		<Block::Hash as FromStr>::Err: Debug,
		NumberFor<Block>: FromStr,
		<NumberFor<Block> as FromStr>::Err: Debug,
		ExecDispatch: NativeExecutionDispatch + 'static,
	{
		let wasm_method = self.wasm_method;
		let execution = self.execution;
		let heap_pages = self.heap_pages.or(config.default_heap_pages);

		//let mut changes = Default::default();
		let max_runtime_instances = config.max_runtime_instances;
		let executor = sc_executor::NativeExecutor::<ExecDispatch>::new(
			wasm_method.into(),
			heap_pages,
			max_runtime_instances,
		);

		let rpc_client = RpcClient::new(self.url.clone()).await?;

		// Get blocks
		let blocks = get_blocks::<Block>(&self.from, self.n, &rpc_client).await?;
		let block_number = blocks[0].header().number();
		let block_hash = blocks[0].header().hash();

		// TODO uncomment when substrate upgraded
		//check_spec_name::<Block>(self.url.clone(), config.chain_spec.name().to_string()).await;

		let states_folder = PathBuf::from(STATES_PATH);
		if !states_folder.exists() {
			std::fs::create_dir_all(&states_folder)?;
		}
		let state_path = states_folder.join(format!("state_{}_{}", block_number, block_hash));
		let mode = if state_path.exists() {
			let mode = Mode::<Block>::Offline(OfflineConfig {
				state_snapshot: SnapshotConfig::new(state_path),
			});

			mode
		} else {
			let parent_hash = blocks[0].header().parent_hash();

			let mode = Mode::Online(OnlineConfig {
				transport: self.url.clone().into(),
				state_snapshot: Some(SnapshotConfig::new(state_path)),
				modules: Vec::new(),
				at: Some(parent_hash.to_owned()),
				..Default::default()
			});

			mode
		};

		let ext = {
			let mut builder = remote_externalities::Builder::<Block>::new()
				.mode(mode)
				.inject_hashed_key(
					&[twox_128(b"System"), twox_128(b"LastRuntimeUpgrade")].concat(),
				);
			if let Some(wasm_code) = wasm_code {
				builder = builder.inject_key_value(&[(
					StorageKey(well_known_keys::CODE.to_vec()),
					StorageData(wasm_code.to_vec()),
				)]);
			}
			let mut ext = builder.build().await?;

			// register externality extensions in order to provide host interface for OCW to the
			// runtime.
			let (offchain, _offchain_state) = TestOffchainExt::new();
			let (pool, _pool_state) = TestTransactionPoolExt::new();
			ext.register_extension(OffchainDbExt::new(offchain.clone()));
			ext.register_extension(OffchainWorkerExt::new(offchain));
			ext.register_extension(KeystoreExt(sc_service::Arc::new(KeyStore::new())));
			ext.register_extension(TransactionPoolExt::new(pool));

			ext
		};

		for block in blocks {
			let block_number = *block.header().number();
			let block_hash = block.header().hash();

			// A digest item gets added when the runtime is processing the block, so we need to pop
			// the last one to be consistent with what a gossiped block would contain.
			let (mut header, extrinsics) = block.deconstruct();
			header.digest_mut().pop();
			let block = Block::new(header, extrinsics);

			let mut changes = Default::default();
			let _encoded_result = sp_state_machine::StateMachine::<_, _, NumberFor<Block>, _>::new(
				&ext.backend,
				None,
				&mut changes,
				&executor,
				"Core_execute_block",
				block.encode().as_ref(),
				Default::default(),
				&sp_state_machine::backend::BackendRuntimeCode::new(&ext.backend).runtime_code()?,
				sp_core::testing::TaskExecutor::new(),
			)
			.execute(execution.into())
			.map_err(|e| {
				format!(
					"failed to execute 'Core_execute_block' at block#{}-{}: {:?}",
					block_number, block_hash, e
				)
			})?;

			log::info!(target: "replay", "block #{}-{} executed without errors.", block_number, block_hash);
		}

		// Get storage
		if let Some(ref ouput_storage) = self.ouput_storage {
			let storage = ext.backend.into_storage().drain();
			let mut json_map = serde_json::Map::new();
			for (k, (v, _i)) in storage {
				json_map.insert(
					format!("0x{:x}", k),
					serde_json::Value::String(format!("0x{}", hex::encode(v))),
				);
			}

			let mut file = std::fs::File::create(ouput_storage)?;
			file.write_all(serde_json::Value::Object(json_map).to_string().as_ref())?;
		}

		Ok(())
	}
}

async fn get_blocks<Block>(
	from: &parse::BlockNumberOrHash,
	n: usize,
	rpc_client: &RpcClient,
) -> sc_cli::Result<Vec<Block>>
where
	Block: BlockT + serde::de::DeserializeOwned,
	Block::Header: serde::de::DeserializeOwned,
	Block::Hash: FromStr,
	<Block::Hash as FromStr>::Err: Debug,
{
	let (from, by_number) = match &from {
		parse::BlockNumberOrHash::BlockNumber(block_number) => {
			(BlockRef::BlockNumber((*block_number).into()), true)
		}
		parse::BlockNumberOrHash::BlockHash(block_hash) => (
			BlockRef::BlockHash(parse::str_to_block_hash::<Block>(block_hash)?),
			false,
		),
	};

	let mut blocks = Vec::with_capacity(n);
	let first_block = get_block::<Block>(from, rpc_client).await?;
	let first_block_number = *(first_block.header().number());
	blocks.push(first_block);
	for i in 1..n {
		let block_number = first_block_number + (i as u32).into();
		let block_ref = if by_number {
			BlockRef::BlockNumber(block_number)
		} else {
			log::info!("Get hash of block #{} from network", block_number);
			let block_hash = rpc_client.get_block_hash::<Block>(block_number).await?;
			BlockRef::BlockHash(block_hash)
		};
		blocks.push(get_block::<Block>(block_ref, rpc_client).await?);
	}
	Ok(blocks)
}

enum BlockRef<Block: BlockT> {
	BlockNumber(<Block::Header as HeaderT>::Number),
	BlockHash(Block::Hash),
}

async fn get_block<Block>(
	block_ref: BlockRef<Block>,
	rpc_client: &RpcClient,
) -> sc_cli::Result<Block>
where
	Block: BlockT + serde::de::DeserializeOwned,
	Block::Header: serde::de::DeserializeOwned,
	Block::Hash: FromStr,
	<Block::Hash as FromStr>::Err: Debug,
{
	let blocks_path = PathBuf::from(BLOCKS_PATH);
	if !blocks_path.exists() {
		std::fs::create_dir_all(&blocks_path)?;
	}
	let block_file_path = match &block_ref {
		BlockRef::BlockNumber(block_number) => {
			blocks_path.join(format!("block_{}.bin", block_number))
		}
		BlockRef::BlockHash(block_hash) => blocks_path.join(format!("block_{}.bin", block_hash)),
	};
	if block_file_path.exists() {
		let bytes = std::fs::read(block_file_path)?;
		Ok(Block::decode(&mut &*bytes)?)
	} else {
		let block_hash = match &block_ref {
			BlockRef::BlockNumber(block_number) => {
				log::info!("Get hash of block number {} from network", block_number);
				rpc_client
					.get_block_hash::<Block>((*block_number).into())
					.await?
			}
			BlockRef::BlockHash(block_hash) => *block_hash,
		};
		log::warn!("block_hash={:?}", block_hash);
		log::info!("Get content of block {} from network", block_hash);
		let block = rpc_client.get_block::<Block>(block_hash).await?;
		let mut file = std::fs::File::create(block_file_path)?;
		block.using_encoded(|buf| file.write_all(buf))?;
		Ok(block)
	}
}

/*/// Check the spec_name of an `ext`
///
/// If the version does not exist, or if it does not match with the given, it emits a warning.
async fn check_spec_name<Block: BlockT + serde::de::DeserializeOwned>(
	uri: String,
	expected_spec_name: String,
) {
	let expected_spec_name = expected_spec_name.to_lowercase();
	match remote_externalities::rpc_api::get_runtime_version::<Block, _>(uri.clone(), None)
		.await
		.map(|version| String::from(version.spec_name.clone()))
		.map(|spec_name| spec_name.to_lowercase())
	{
		Ok(spec) if spec == expected_spec_name => {
			log::debug!("found matching spec name: {:?}", spec);
		},
		Ok(spec) => {
			log::warn!(
				"version mismatch: remote spec name: '{}', expected (local chain spec, aka. `--chain`): '{}'",
				spec,
				expected_spec_name,
			);
		},
		Err(why) => {
			log::error!("failed to fetch runtime version from {}: {:?}", uri, why);
		},
	}
}*/
