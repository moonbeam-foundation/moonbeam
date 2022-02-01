// Copyright 2019-2022 PureStake Inc.
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
// along with Moonbeam. If not, see <http://www.gnu.org/licenses/>.

//! Utilities for creating the neccessary moonbeam client subsystems.

use parachain_inherent::ParachainInherentData;
use sc_cli::{build_runtime, structopt::StructOpt, SubstrateCli};
use sc_client_api::backend::Backend;
use sc_service::TFullBackend;
use sc_tracing::logging::LoggerBuilder;
use sp_api::{ApiExt, ConstructRuntimeApi, Core, Metadata};
use sp_block_builder::BlockBuilder;
use sp_offchain::OffchainWorkerApi;
use sp_runtime::traits::{Block as BlockT, Header};
use sp_session::SessionKeys;
use sp_transaction_pool::runtime_api::TaggedTransactionQueue;
use std::{error::Error, future::Future, str::FromStr, sync::Arc};
use substrate_simnode::{ChainInfo, FullClientFor, Node, SimnodeCli};

/// Set up and run simnode for a moonbeam type runtime.
pub fn moonbeam_node<C, F, Fut>(callback: F) -> Result<(), Box<dyn Error>>
where
	C: ChainInfo<
			BlockImport = Arc<FullClientFor<C>>,
			InherentDataProviders = (sp_timestamp::InherentDataProvider, ParachainInherentData),
		> + 'static,
	//<C as ChainInfo>::RuntimeApi: ApiExt<C::Block>,
	<C::RuntimeApi as ConstructRuntimeApi<C::Block, FullClientFor<C>>>::RuntimeApi:
		Core<C::Block>
			+ nimbus_primitives::NimbusApi<C::Block>
			+ nimbus_primitives::AuthorFilterAPI<C::Block, nimbus_primitives::NimbusId>
			+ Metadata<C::Block>
			+ OffchainWorkerApi<C::Block>
			+ SessionKeys<C::Block>
			+ TaggedTransactionQueue<C::Block>
			+ BlockBuilder<C::Block>
			+ ApiExt<C::Block, StateBackend = <TFullBackend<C::Block> as Backend<C::Block>>::State>,
	<C::Runtime as system::Config>::Call: From<system::Call<C::Runtime>>,
	<<C as ChainInfo>::Block as BlockT>::Hash: FromStr + Unpin,
	<<C as ChainInfo>::Block as BlockT>::Header: Unpin,
	<<<C as ChainInfo>::Block as BlockT>::Header as Header>::Number:
		num_traits::cast::AsPrimitive<usize> + num_traits::cast::AsPrimitive<u32>,
	F: FnOnce(Node<C>) -> Fut,
	Fut: Future<Output = Result<(), Box<dyn Error>>>,
{
	let tokio_runtime = build_runtime()?;
	// parse cli args
	let cli = <<<C as ChainInfo>::Cli as SimnodeCli>::SubstrateCli as StructOpt>::from_args();
	let cli_config = <C as ChainInfo>::Cli::cli_config(&cli);

	// set up logging
	LoggerBuilder::new(<C as ChainInfo>::Cli::log_filters(cli_config)?).init()?;

	// set up the test-runner
	let config = cli.create_configuration(cli_config, tokio_runtime.handle().clone())?;
	sc_cli::print_node_infos::<<<C as ChainInfo>::Cli as SimnodeCli>::SubstrateCli>(&config);

	let node = substrate_simnode::build_node_subsystems::<C, _>(
		config,
		true,
		|client, _sc, keystore, parachain_inherent| {
			let create_inherent_data_providers = Box::new(move |_, _| {
				let parachain_sproof = parachain_inherent.clone().unwrap();

				async move {
					let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

					let parachain_system = parachain_sproof.lock().unwrap().create_inherent(0);

					Ok((timestamp, parachain_system))
				}
			});

			let consensus_data_provider = nimbus_consensus::NimbusManualSealConsensusDataProvider {
				keystore: keystore.sync_keystore(),
				client: client.clone(),
			};
			Ok((
				client,
				Some(Box::new(consensus_data_provider)),
				create_inherent_data_providers,
			))
		},
	)?;

	// hand off node.
	tokio_runtime.block_on(callback(node))?;

	Ok(())
}
