// Copyright 2024 Moonbeam foundation
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
use sc_chain_spec::{get_extension, BuildGenesisBlock, GenesisBlockBuilder};
use sc_client_api::{BadBlocks, BlockImportOperation, ForkBlocks, NewBlockState};
use sc_client_db::{Backend, DatabaseSettings};
use sc_executor::RuntimeVersionOf;
use sc_network_common::sync::SyncMode;

use moonbeam_cli_opt::LazyLoadingConfig;
use sc_service::{ClientConfig, Configuration, Error, KeystoreContainer, TaskManager};
use sc_telemetry::TelemetryHandle;
use sp_core::traits::CodeExecutor;
use sp_runtime::traits::Header as HeaderT;
use sp_runtime::traits::NumberFor;
use sp_storage::{StateVersion, Storage};
use std::str::FromStr;
use std::sync::Arc;

/// Full client type.
pub type TForkClient<TBl, TRtApi, TExec> =
	sc_service::Client<TForkBackend<TBl>, TForkCallExecutor<TBl, TExec>, TBl, TRtApi>;

/// Full client backend type.
pub type TForkBackend<TBl> = sc_client_db::fork_backend::Backend<TBl, Backend<TBl>>;

/// Full client call executor type.
pub type TForkCallExecutor<TBl, TExec> =
	sc_service::LocalCallExecutor<TBl, TForkBackend<TBl>, TExec>;

pub type TForkParts<TBl, TRtApi, TExec> = (
	TForkClient<TBl, TRtApi, TExec>,
	Arc<TForkBackend<TBl>>,
	KeystoreContainer,
	TaskManager,
);

/// Create the initial parts of a fork node with the default genesis block builder.
pub fn new_fork_parts<TBl, TRtApi, TExec>(
	config: &Configuration,
	lazy_loading_config: &LazyLoadingConfig,
	telemetry: Option<TelemetryHandle>,
	executor: TExec,
) -> Result<TForkParts<TBl, TRtApi, TExec>, Error>
where
	TBl: BlockT + sp_runtime::DeserializeOwned,
	TExec: CodeExecutor + RuntimeVersionOf + Clone,
{
	new_fork_parts_record_import(config, lazy_loading_config, telemetry, executor, false)
}

/// Create the initial parts of a fork node with the default genesis block builder.
pub fn new_fork_parts_record_import<TBl, TRtApi, TExec>(
	config: &Configuration,
	lazy_loading_config: &LazyLoadingConfig,
	telemetry: Option<TelemetryHandle>,
	executor: TExec,
	enable_import_proof_recording: bool,
) -> Result<TForkParts<TBl, TRtApi, TExec>, Error>
where
	TBl: BlockT + sp_runtime::DeserializeOwned,
	TExec: CodeExecutor + RuntimeVersionOf + Clone,
{
	use sc_client_api::Backend;
	let backend = new_lazy_loading_backend(config.db_config(), &lazy_loading_config)?;

	let mut op = backend.begin_operation().unwrap();

	let state_root = op
		.set_genesis_state(
			Storage {
				top: vec![(
				sp_core::storage::well_known_keys::CODE.to_vec(),
				moonbeam_runtime::WASM_BINARY.expect(
					"Development wasm binary is not available. This means the client is built with \
						 `SKIP_WASM_BUILD` flag and it is only usable for production chains. Please rebuild with \
						 the flag disabled.",
				).to_vec()
			)]
				.into_iter()
				.collect(),
				children_default: Default::default(),
			},
			true,
			StateVersion::V1,
		)
		.unwrap();

	let header: TBl::Header = TBl::Header::new(
		Default::default(),
		Default::default(),
		state_root,
		Default::default(),
		Default::default(),
	);

	op.set_block_data(
		header.clone(),
		Some(vec![]),
		None,
		None,
		NewBlockState::Best,
	)
	.unwrap();

	backend.commit_operation(op).unwrap();

	let genesis_block_builder = GenesisBlockBuilder::new(
		config.chain_spec.as_storage_builder(),
		!config.no_genesis(),
		backend.clone(),
		executor.clone(),
	)?;

	new_fork_parts_with_genesis_builder(
		config,
		telemetry,
		executor,
		backend,
		genesis_block_builder,
		enable_import_proof_recording,
	)
}

/// Create the initial parts of a fork node.
pub fn new_fork_parts_with_genesis_builder<TBl, TRtApi, TExec, TBuildGenesisBlock>(
	config: &Configuration,
	telemetry: Option<TelemetryHandle>,
	executor: TExec,
	backend: Arc<TForkBackend<TBl>>,
	genesis_block_builder: TBuildGenesisBlock,
	enable_import_proof_recording: bool,
) -> Result<TForkParts<TBl, TRtApi, TExec>, Error>
	where
		TBl: BlockT + sp_runtime::DeserializeOwned,
		TExec: CodeExecutor + RuntimeVersionOf + Clone,
		TBuildGenesisBlock: BuildGenesisBlock<
			TBl,
			BlockImportOperation = <TForkBackend<TBl> as sc_client_api::backend::Backend<TBl>>::BlockImportOperation
		>,
{
	let keystore_container = KeystoreContainer::new(&config.keystore)?;

	let task_manager = {
		let registry = config.prometheus_config.as_ref().map(|cfg| &cfg.registry);
		TaskManager::new(config.tokio_handle.clone(), registry)?
	};

	let chain_spec = &config.chain_spec;
	let fork_blocks = get_extension::<ForkBlocks<TBl>>(chain_spec.extensions())
		.cloned()
		.unwrap_or_default();

	let bad_blocks = get_extension::<BadBlocks<TBl>>(chain_spec.extensions())
		.cloned()
		.unwrap_or_default();

	let client = {
		let extensions = sc_client_api::execution_extensions::ExecutionExtensions::new(
			None,
			Arc::new(executor.clone()),
		);

		let wasm_runtime_substitutes = config
			.chain_spec
			.code_substitutes()
			.into_iter()
			.map(|(n, c)| {
				let number = NumberFor::<TBl>::from_str(&n).map_err(|_| {
					Error::Application(Box::from(format!(
						"Failed to parse `{}` as block number for code substitutes. \
						 In an old version the key for code substitute was a block hash. \
						 Please update the chain spec to a version that is compatible with your node.",
						n
					)))
				})?;
				Ok((number, c))
			})
			.collect::<Result<std::collections::HashMap<_, _>, Error>>()?;

		let client = sc_service::new_client(
			backend.clone(),
			executor,
			genesis_block_builder,
			fork_blocks,
			bad_blocks,
			extensions,
			Box::new(task_manager.spawn_handle()),
			config
				.prometheus_config
				.as_ref()
				.map(|config| config.registry.clone()),
			telemetry,
			ClientConfig {
				offchain_worker_enabled: config.offchain_worker.enabled,
				offchain_indexing_api: config.offchain_worker.indexing_enabled,
				wasmtime_precompiled: config.wasmtime_precompiled.clone(),
				wasm_runtime_overrides: config.wasm_runtime_overrides.clone(),
				no_genesis: matches!(
					config.network.sync_mode,
					SyncMode::LightState { .. } | SyncMode::Warp { .. }
				),
				wasm_runtime_substitutes,
				enable_import_proof_recording,
			},
		)?;

		client
	};

	Ok((client, backend, keystore_container, task_manager))
}

/// Create an instance of default DB-backend backend.
pub fn new_lazy_loading_backend<Block>(
	settings: DatabaseSettings,
	lazy_loading_config: &LazyLoadingConfig,
) -> Result<Arc<sc_client_db::fork_backend::Backend<Block, Backend<Block>>>, sp_blockchain::Error>
where
	Block: BlockT + sp_runtime::DeserializeOwned,
{
	const CANONICALIZATION_DELAY: u64 = 4096;

	log::error!("RPC: {:?}\n\n", lazy_loading_config.state_rpc);
	let uri: String = lazy_loading_config.state_rpc.clone().into();

	let http_client = jsonrpsee::http_client::HttpClientBuilder::default()
		.max_request_size(u32::MAX)
		.max_response_size(u32::MAX)
		.request_timeout(std::time::Duration::from_secs(60 * 5))
		.build(uri)
		.map_err(|e| {
			log::error!("error: {:?}", e);
			sp_blockchain::Error::Backend("failed to build http client".to_string())
		})?;

	let db = Backend::new(settings, CANONICALIZATION_DELAY)?;

	Ok(Arc::new(sc_client_db::fork_backend::Backend::new(
		db,
		Arc::new(http_client),
	)))
}
