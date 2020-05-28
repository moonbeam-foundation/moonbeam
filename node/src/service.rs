//! Service and ServiceFactory implementation. Specialized wrapper over substrate service.
use futures::prelude::*;
use node_moonbeam_runtime::{self, RuntimeApi};
use node_primitives::{AccountId, Block, Index};
use sc_client_api::ExecutorProvider;
use sc_consensus::LongestChain;
use sc_executor::native_executor_instance;
pub use sc_executor::NativeExecutor;
use sc_finality_grandpa::{
	FinalityProofProvider as GrandpaFinalityProofProvider, SharedVoterState,
	StorageAndProofProvider,
};
use sc_network::Event;
use sc_service::{
	config::Configuration, error::Error as ServiceError, AbstractService, ServiceBuilder,
};
use sp_inherents::InherentDataProviders;
use std::sync::Arc;
use std::time::Duration;

// Our native executor instance.
native_executor_instance!(
	pub Executor,
	node_moonbeam_runtime::api::dispatch,
	node_moonbeam_runtime::native_version,
);

/// Starts a `ServiceBuilder` for a full service.
///
/// Use this macro if you don't actually need the full service, but just the builder in order to
/// be able to perform chain operations.
macro_rules! new_full_start {
	($config:expr) => {{
		type RpcExtension = jsonrpc_core::IoHandler<sc_rpc::Metadata>;

		use std::sync::Arc;

		let mut import_setup = None;
		let inherent_data_providers = sp_inherents::InherentDataProviders::new();

		let builder = sc_service::ServiceBuilder::new_full::<
			node_primitives::Block,
			node_moonbeam_runtime::RuntimeApi,
			crate::service::Executor,
		>($config)?
		.with_select_chain(|_config, backend| Ok(sc_consensus::LongestChain::new(backend.clone())))?
		.with_transaction_pool(|config, client, _fetcher, prometheus_registry| {
			let pool_api = sc_transaction_pool::FullChainApi::new(client.clone());
			Ok(sc_transaction_pool::BasicPool::new(
				config,
				std::sync::Arc::new(pool_api),
				prometheus_registry,
			))
		})?
		.with_import_queue(
			|_config, client, mut select_chain, _transaction_pool, spawn_task_handle| {
				let select_chain = select_chain
					.take()
					.ok_or_else(|| sc_service::Error::SelectChainRequired)?;

				let (grandpa_block_import, grandpa_link) = sc_finality_grandpa::block_import(
					client.clone(),
					&(client.clone() as Arc<_>),
					select_chain,
				)?;

				let justification_import = grandpa_block_import.clone();

				let (block_import, babe_link) = sc_consensus_babe::block_import(
					sc_consensus_babe::Config::get_or_compute(&*client)?,
					grandpa_block_import,
					client.clone(),
				)?;

				let import_queue = sc_consensus_babe::import_queue(
					babe_link.clone(),
					block_import.clone(),
					Some(Box::new(justification_import)),
					None,
					client.clone(),
					inherent_data_providers.clone(),
					spawn_task_handle,
				)?;

				import_setup = Some((block_import, grandpa_link, babe_link));

				Ok(import_queue)
			},
			)?
		.with_rpc_extensions(|builder| -> Result<RpcExtension, _> {
			use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApi};
			use sc_consensus_babe_rpc::BabeRPCHandler;
			use substrate_frame_rpc_system::{FullSystem, SystemApi};

			let mut io = jsonrpc_core::IoHandler::default();

			let babe_link = import_setup
				.as_ref()
				.map(|s| &s.2)
				.expect("BabeLink is present for full services or set up failed; qed.");

			io.extend_with(SystemApi::to_delegate(FullSystem::new(
				builder.client().clone(),
				builder.pool(),
			)));
			io.extend_with(TransactionPaymentApi::to_delegate(TransactionPayment::new(
				builder.client().clone(),
			)));

			io.extend_with(sc_consensus_babe_rpc::BabeApi::to_delegate(
				BabeRPCHandler::new(
					builder.client().clone(),
					sc_consensus_babe::BabeLink::epoch_changes(babe_link).clone(),
					builder.keystore(),
					sc_consensus_babe::BabeLink::config(babe_link).clone(),
					builder
						.select_chain()
						.cloned()
						.expect("SelectChain is present for full services or set up failed; qed."),
				),
			));

			Ok(io)
		})?;

		(builder, import_setup, inherent_data_providers)
		}};
}

/// Builds a new service for a full client.
pub fn new_full(config: Configuration) -> Result<impl AbstractService, ServiceError> {
	let role = config.role.clone();
	let is_authority = role.is_authority();
	let force_authoring = config.force_authoring;
	let name = config.network.node_name.clone();
	let disable_grandpa = config.disable_grandpa;

	// sentry nodes announce themselves as authorities to the network
	// and should run the same protocols authorities do, but it should
	// never actively participate in any consensus process.
	let participates_in_consensus = is_authority;

	let (builder, mut import_setup, inherent_data_providers) = new_full_start!(config);

	let (block_import, grandpa_link, babe_link) = import_setup.take().expect(
		"Link Half and Block Import are present for Full Services or setup failed before. qed",
	);

	let service = builder
		.with_finality_proof_provider(|client, backend| {
			// GenesisAuthoritySetProvider is implemented for StorageAndProofProvider
			let provider = client as Arc<dyn StorageAndProofProvider<_, _>>;
			Ok(Arc::new(GrandpaFinalityProofProvider::new(backend, provider)) as _)
		})?
		.build()?;

	let (sentries, authority_discovery_role) = match role {
		sc_service::config::Role::Authority { ref sentry_nodes } => (
			sentry_nodes.clone(),
			sc_authority_discovery::Role::Authority(service.keystore()),
		),
		sc_service::config::Role::Sentry { .. } => (vec![], sc_authority_discovery::Role::Sentry),
		_ => unreachable!("Due to outer matches! constraint; qed."),
	};

	if participates_in_consensus {
		let proposer =
			sc_basic_authorship::ProposerFactory::new(service.client(), service.transaction_pool());

		let client = service.client();
		let select_chain = service
			.select_chain()
			.ok_or(ServiceError::SelectChainRequired)?;

		let can_author_with =
			sp_consensus::CanAuthorWithNativeVersion::new(client.executor().clone());

		let babe_config = sc_consensus_babe::BabeParams {
			keystore: service.keystore(),
			client,
			select_chain,
			env: proposer,
			block_import,
			sync_oracle: service.network(),
			inherent_data_providers: inherent_data_providers.clone(),
			force_authoring,
			babe_link,
			can_author_with,
		};

		let babe = sc_consensus_babe::start_babe(babe_config)?;
		service.spawn_essential_task("babe-proposer", babe);

		let network = service.network();
		let dht_event_stream = network
			.event_stream("authority-discovery")
			.filter_map(|e| async move {
				match e {
					Event::Dht(e) => Some(e),
					_ => None,
				}
			})
			.boxed();
		let authority_discovery = sc_authority_discovery::AuthorityDiscovery::new(
			service.client(),
			network,
			sentries,
			dht_event_stream,
			authority_discovery_role,
			service.prometheus_registry(),
		);

		service.spawn_task("authority-discovery", authority_discovery);
	}

	// if the node isn't actively participating in consensus then it doesn't
	// need a keystore, regardless of which protocol we use below.
	let keystore = if role.is_authority() {
		Some(service.keystore())
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

	let enable_grandpa = !disable_grandpa;
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
			network: service.network(),
			inherent_data_providers: inherent_data_providers.clone(),
			telemetry_on_connect: Some(service.telemetry_on_connect_stream()),
			voting_rule: sc_finality_grandpa::VotingRulesBuilder::default().build(),
			prometheus_registry: service.prometheus_registry(),
			shared_voter_state: SharedVoterState::empty(),
		};

		// the GRANDPA voter task is considered infallible, i.e.
		// if it fails we take down the service with it.
		service.spawn_essential_task(
			"grandpa-voter",
			sc_finality_grandpa::run_grandpa_voter(grandpa_config)?,
		);
	} else {
		sc_finality_grandpa::setup_disabled_grandpa(
			service.client(),
			&inherent_data_providers,
			service.network(),
		)?;
	}

	Ok(service)
}

/// Builds a new service for a light client.
pub fn new_light(config: Configuration) -> Result<impl AbstractService, ServiceError> {
	type RpcExtension = jsonrpc_core::IoHandler<sc_rpc::Metadata>;
	let inherent_data_providers = InherentDataProviders::new();

	let service = ServiceBuilder::new_light::<Block, RuntimeApi, Executor>(config)?
		.with_select_chain(|_config, backend| Ok(LongestChain::new(backend.clone())))?
		.with_transaction_pool(|config, client, fetcher, prometheus_registry| {
			let fetcher = fetcher
				.ok_or_else(|| "Trying to start light transaction pool without active fetcher")?;

			let pool_api = sc_transaction_pool::LightChainApi::new(client.clone(), fetcher.clone());
			let pool = sc_transaction_pool::BasicPool::with_revalidation_type(
				config,
				Arc::new(pool_api),
				prometheus_registry,
				sc_transaction_pool::RevalidationType::Light,
			);
			Ok(pool)
		})?
		.with_import_queue_and_fprb(
			|_config, client, backend, fetcher, _select_chain, _tx_pool, spawn_task_handle| {
				let fetch_checker = fetcher
					.map(|fetcher| fetcher.checker().clone())
					.ok_or_else(|| {
						"Trying to start light import queue without active fetch checker"
					})?;
				let grandpa_block_import = sc_finality_grandpa::light_block_import(
					client.clone(),
					backend,
					&(client.clone() as Arc<_>),
					Arc::new(fetch_checker),
				)?;
				let finality_proof_import = grandpa_block_import.clone();
				let finality_proof_request_builder =
					finality_proof_import.create_finality_proof_request_builder();

				let (babe_block_import, babe_link) = sc_consensus_babe::block_import(
					sc_consensus_babe::Config::get_or_compute(&*client)?,
					grandpa_block_import,
					client.clone(),
				)?;

				let import_queue = sc_consensus_babe::import_queue(
					babe_link,
					babe_block_import,
					None,
					Some(Box::new(finality_proof_import)),
					client.clone(),
					inherent_data_providers.clone(),
					spawn_task_handle,
				)?;

				Ok((import_queue, finality_proof_request_builder))
			},
		)?
		.with_finality_proof_provider(|client, backend| {
			// GenesisAuthoritySetProvider is implemented for StorageAndProofProvider
			let provider = client as Arc<dyn StorageAndProofProvider<_, _>>;
			Ok(Arc::new(GrandpaFinalityProofProvider::new(backend, provider)) as _)
		})?
		.with_rpc_extensions(|builder| -> Result<RpcExtension, _> {
			use substrate_frame_rpc_system::{LightSystem, SystemApi};

			let fetcher = builder
				.fetcher()
				.ok_or_else(|| "Trying to start node RPC without active fetcher")?;
			let remote_blockchain = builder
				.remote_backend()
				.ok_or_else(|| "Trying to start node RPC without active remote blockchain")?;

			let mut io = jsonrpc_core::IoHandler::default();
			io.extend_with(SystemApi::<AccountId, Index>::to_delegate(
				LightSystem::new(
					builder.client().clone(),
					remote_blockchain,
					fetcher,
					builder.pool(),
				),
			));
			Ok(io)
		})?
		.build()?;

	Ok(service)
}
