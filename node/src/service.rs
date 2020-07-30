//! Service and ServiceFactory implementation. Specialized wrapper over substrate service.

use std::sync::Arc;
use std::time::Duration;
use sc_client_api::{ExecutorProvider, RemoteBackend};
use sc_consensus_manual_seal::{self as manual_seal};
use moonbeam_runtime::{self, opaque::Block, RuntimeApi, Hash};
use sc_service::{error::Error as ServiceError, Configuration, ServiceComponents, TaskManager};
use sp_inherents::InherentDataProviders;
use sc_executor::native_executor_instance;
pub use sc_executor::NativeExecutor;
use sp_consensus_aura::sr25519::{AuthorityPair as AuraPair};
use sc_finality_grandpa::{
	FinalityProofProvider as GrandpaFinalityProofProvider, StorageAndProofProvider, SharedVoterState,
};

// Our native executor instance.
native_executor_instance!(
	pub Executor,
	moonbeam_runtime::api::dispatch,
	moonbeam_runtime::native_version,
);

type FullClient = sc_service::TFullClient<Block, RuntimeApi, Executor>;
type FullBackend = sc_service::TFullBackend<Block>;
type FullSelectChain = sc_consensus::LongestChain<FullBackend, Block>;

pub enum ConsensusResult {
	GrandPa((
		sc_finality_grandpa::GrandpaBlockImport<FullBackend, Block, FullClient, FullSelectChain>,
		sc_finality_grandpa::LinkHalf<Block, FullClient, FullSelectChain>
	)),
	ManualSeal
}

pub fn new_full_params(config: Configuration, manual_seal: bool) -> Result<(
	sc_service::ServiceParams<
		Block, FullClient,
		sp_consensus::import_queue::BasicQueue<Block, sp_api::TransactionFor<FullClient, Block>>,
		sc_transaction_pool::FullPool<Block, FullClient>,
		crate::rpc::IoHandler, FullBackend,
	>,
	FullSelectChain,
	sp_inherents::InherentDataProviders,
	futures::channel::mpsc::Receiver<sc_consensus_manual_seal::rpc::EngineCommand<Hash>>,
	ConsensusResult
), ServiceError> {
	let inherent_data_providers = sp_inherents::InherentDataProviders::new();

	let (client, backend, keystore, task_manager) =
		sc_service::new_full_parts::<Block, RuntimeApi, Executor>(&config)?;
	let client = Arc::new(client);

	let select_chain = sc_consensus::LongestChain::new(backend.clone());

	let pool_api = sc_transaction_pool::FullChainApi::new(
		client.clone(), config.prometheus_registry(),
	);
	let transaction_pool = sc_transaction_pool::BasicPool::new_full(
		config.transaction_pool.clone(),
		std::sync::Arc::new(pool_api),
		config.prometheus_registry(),
		task_manager.spawn_handle(),
		client.clone(),
	);

	// Channel for the rpc handler to communicate with the authorship task.
	let (command_sink, commands_stream) = futures::channel::mpsc::channel(1000);

	let is_authority = config.role.is_authority();

	let rpc_extensions_builder = {
		let client = client.clone();
		let pool = transaction_pool.clone();
		let select_chain = select_chain.clone();

		Box::new(move |deny_unsafe| {
			let deps = crate::rpc::FullDeps {
				client: client.clone(),
				pool: pool.clone(),
				select_chain: select_chain.clone(),
				deny_unsafe,
				is_authority,
				command_sink: Some(command_sink.clone())
			};

			crate::rpc::create_full(deps)
		})
	};

	if manual_seal {
		inherent_data_providers
			.register_provider(sp_timestamp::InherentDataProvider)
			.map_err(Into::into)
			.map_err(sp_consensus::error::Error::InherentData)?;

		let import_queue = sc_consensus_manual_seal::import_queue(
			Box::new(client.clone()),
			&task_manager.spawn_handle(),
			config.prometheus_registry(),
		);


		let params = sc_service::ServiceParams {
			backend, client, import_queue, keystore, task_manager, transaction_pool, rpc_extensions_builder,
			config,
			block_announce_validator_builder: None,
			finality_proof_request_builder: Some(Box::new(sc_network::config::DummyFinalityProofRequestBuilder)),
			finality_proof_provider: None,
			on_demand: None,
			remote_blockchain: None,
		};

		return Ok((
			params, select_chain, inherent_data_providers, commands_stream,
			ConsensusResult::ManualSeal
		));
	}

	let (grandpa_block_import, grandpa_link) = sc_finality_grandpa::block_import(
		client.clone(), &(client.clone() as Arc<_>), select_chain.clone(),
	)?;

	let aura_block_import = sc_consensus_aura::AuraBlockImport::<_, _, _, AuraPair>::new(
		grandpa_block_import.clone(), client.clone(),
	);

	let import_queue = sc_consensus_aura::import_queue::<_, _, _, AuraPair, _>(
		sc_consensus_aura::slot_duration(&*client)?,
		aura_block_import,
		Some(Box::new(grandpa_block_import.clone())),
		None,
		client.clone(),
		inherent_data_providers.clone(),
		&task_manager.spawn_handle(),
		config.prometheus_registry(),
	)?;

	let provider = client.clone() as Arc<dyn StorageAndProofProvider<_, _>>;
	let finality_proof_provider =
		Arc::new(GrandpaFinalityProofProvider::new(backend.clone(), provider));


	let params = sc_service::ServiceParams {
		backend, client, import_queue, keystore, task_manager, transaction_pool, rpc_extensions_builder,
		config,
		block_announce_validator_builder: None,
		finality_proof_request_builder: None,
		finality_proof_provider: Some(finality_proof_provider),
		on_demand: None,
		remote_blockchain: None,
	};

	Ok((
		params, select_chain, inherent_data_providers, commands_stream,
		ConsensusResult::GrandPa((grandpa_block_import, grandpa_link))
	))
}

/// Builds a new service for a full client.
pub fn new_full(config: Configuration, manual_seal: bool) -> Result<TaskManager, ServiceError> {
	let (
		params, select_chain, inherent_data_providers,
		commands_stream, consensus_result
	) = new_full_params(config, manual_seal)?;

	let (
		role, force_authoring, name, enable_grandpa, prometheus_registry,
		client, transaction_pool, keystore,
	) = {
		let sc_service::ServiceParams {
			config, client, transaction_pool, keystore, ..
		} = &params;

		(
			config.role.clone(),
			config.force_authoring,
			config.network.node_name.clone(),
			!config.disable_grandpa,
			config.prometheus_registry().cloned(),

			client.clone(), transaction_pool.clone(), keystore.clone(),
		)
	};

	let ServiceComponents {
		task_manager, network, telemetry_on_connect_sinks, ..
	} = sc_service::build(params)?;

	match consensus_result {
		ConsensusResult::ManualSeal => {
			if role.is_authority() {
				let proposer = sc_basic_authorship::ProposerFactory::new(
					client.clone(),
					transaction_pool.clone(),
					prometheus_registry.as_ref(),
				);

				// Background authorship future
				let authorship_future = manual_seal::run_manual_seal(
					Box::new(client.clone()),
					proposer,
					client.clone(),
					transaction_pool.pool().clone(),
					commands_stream,
					select_chain,
					inherent_data_providers,
				);

				// we spawn the future on a background thread managed by service.
				task_manager.spawn_essential_handle().spawn_blocking("manual-seal", authorship_future);
			}
			log::info!("Manual Seal Ready");
		},
		ConsensusResult::GrandPa((grandpa_block_import, grandpa_link)) => {
			if role.is_authority() {
				let proposer = sc_basic_authorship::ProposerFactory::new(
					client.clone(),
					transaction_pool.clone(),
					prometheus_registry.as_ref(),
				);


				let can_author_with =
					sp_consensus::CanAuthorWithNativeVersion::new(client.executor().clone());
				let aura = sc_consensus_aura::start_aura::<_, _, _, _, _, AuraPair, _, _, _>(
					sc_consensus_aura::slot_duration(&*client)?,
					client.clone(),
					select_chain,
					grandpa_block_import,
					proposer,
					network.clone(),
					inherent_data_providers.clone(),
					force_authoring,
					keystore.clone(),
					can_author_with,
				)?;

				// the AURA authoring task is considered essential, i.e. if it
				// fails we take down the service with it.
				task_manager.spawn_essential_handle().spawn_blocking("aura", aura);


				// if the node isn't actively participating in consensus then it doesn't
				// need a keystore, regardless of which protocol we use below.
				let keystore = if role.is_authority() {
					Some(keystore as sp_core::traits::BareCryptoStorePtr)
				} else {
					None
				};
				let grandpa_config = sc_finality_grandpa::Config {
					// FIXME #1578 make this available through chainspec
					gossip_duration: Duration::from_millis(333),
					justification_period: 512,
					name: Some(name),
					observer_enabled: false,
					keystore,
					is_authority: role.is_network_authority(),
				};

				if enable_grandpa {
					// start the full GRANDPA voter
					// NOTE: non-authorities could run the GRANDPA observer protocol, but at
					// this point the full voter should provide better guarantees of block
					// and vote data availability than the observer. The observer has not
					// been tested extensively yet and having most nodes in a network run it
					// could lead to finality stalls.
					let grandpa_config = sc_finality_grandpa::GrandpaParams {
						config: grandpa_config,
						link: grandpa_link,
						network,
						inherent_data_providers,
						telemetry_on_connect: Some(telemetry_on_connect_sinks.on_connect_stream()),
						voting_rule: sc_finality_grandpa::VotingRulesBuilder::default().build(),
						prometheus_registry,
						shared_voter_state: SharedVoterState::empty(),
					};

					// the GRANDPA voter task is considered infallible, i.e.
					// if it fails we take down the service with it.
					task_manager.spawn_essential_handle().spawn_blocking(
						"grandpa-voter",
						sc_finality_grandpa::run_grandpa_voter(grandpa_config)?
					);
				} else {
					sc_finality_grandpa::setup_disabled_grandpa(
						client,
						&inherent_data_providers,
						network,
					)?;
				}
			}
		}
	}

	Ok(task_manager)
}

/// Builds a new service for a light client.
pub fn new_light(config: Configuration) -> Result<TaskManager, ServiceError> {
	let (client, backend, keystore, task_manager, on_demand) =
		sc_service::new_light_parts::<Block, RuntimeApi, Executor>(&config)?;

	let transaction_pool_api = Arc::new(sc_transaction_pool::LightChainApi::new(
		client.clone(), on_demand.clone(),
	));
	let transaction_pool = Arc::new(sc_transaction_pool::BasicPool::new_light(
		config.transaction_pool.clone(),
		transaction_pool_api,
		config.prometheus_registry(),
		task_manager.spawn_handle(),
	));

	let grandpa_block_import = sc_finality_grandpa::light_block_import(
		client.clone(), backend.clone(), &(client.clone() as Arc<_>),
		Arc::new(on_demand.checker().clone()) as Arc<_>,
	)?;
	let finality_proof_import = grandpa_block_import.clone();
	let finality_proof_request_builder =
		finality_proof_import.create_finality_proof_request_builder();

	let import_queue = sc_consensus_aura::import_queue::<_, _, _, AuraPair, _>(
		sc_consensus_aura::slot_duration(&*client)?,
		grandpa_block_import,
		None,
		Some(Box::new(finality_proof_import)),
		client.clone(),
		InherentDataProviders::new(),
		&task_manager.spawn_handle(),
		config.prometheus_registry(),
	)?;

	let light_deps = crate::rpc::LightDeps {
		remote_blockchain: backend.remote_blockchain(),
		fetcher: on_demand.clone(),
		client: client.clone(),
		pool: transaction_pool.clone(),
	};

	let rpc_extensions = crate::rpc::create_light(light_deps);

	let finality_proof_provider =
		Arc::new(GrandpaFinalityProofProvider::new(backend.clone(), client.clone() as Arc<_>));

	sc_service::build(sc_service::ServiceParams {
		block_announce_validator_builder: None,
		finality_proof_request_builder: Some(finality_proof_request_builder),
		finality_proof_provider: Some(finality_proof_provider),
		on_demand: Some(on_demand),
		remote_blockchain: Some(backend.remote_blockchain()),
		rpc_extensions_builder: Box::new(sc_service::NoopRpcExtensionBuilder(rpc_extensions)),
		transaction_pool: transaction_pool,
		config, client, import_queue, keystore, backend, task_manager
	}).map(|ServiceComponents { task_manager, .. }| task_manager)
}
