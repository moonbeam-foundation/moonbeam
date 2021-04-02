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

//! This module constructs and executes the appropriate service components for the given subcommand

use crate::{
	chain_spec,
	cli::{Cli, RelayChainCli, Subcommand},
};
use cumulus_client_service::genesis::generate_genesis_block;
use cumulus_primitives_core::ParaId;
use log::info;
use moonbeam_runtime::{AccountId, Block};
use parity_scale_codec::Encode;
use polkadot_parachain::primitives::AccountIdConversion;
use polkadot_service::RococoChainSpec;
use sc_cli::{
	ChainSpec, CliConfiguration, DefaultConfigurationValues, ImportParams, KeystoreParams,
	NetworkParams, Result, RuntimeVersion, SharedParams, SubstrateCli,
};
use sc_service::{
	config::{BasePath, PrometheusConfig},
	PartialComponents,
};
use sp_core::hexdisplay::HexDisplay;
use sp_core::H160;
use sp_runtime::traits::Block as _;
use std::{io::Write, net::SocketAddr, str::FromStr};

fn load_spec(
	id: &str,
	para_id: ParaId,
) -> std::result::Result<Box<dyn sc_service::ChainSpec>, String> {
	match id {
		"alphanet" => Ok(Box::new(chain_spec::ChainSpec::from_json_bytes(
			&include_bytes!("../../specs/alphanet/parachain-embedded-specs-v7.json")[..],
		)?)),
		"stagenet" => Ok(Box::new(chain_spec::ChainSpec::from_json_bytes(
			&include_bytes!("../../specs/stagenet/parachain-embedded-specs-v7.json")[..],
		)?)),
		"dev" | "development" => Ok(Box::new(chain_spec::development_chain_spec(None, None))),
		"local" => Ok(Box::new(chain_spec::get_chain_spec(para_id))),
		"" => Err(
			"You have not specified what chain to sync. In the future, this will default to \
				Moonbeam mainnet. Mainnet is not yet live so you must choose a spec."
				.into(),
		),
		path => Ok(Box::new(chain_spec::ChainSpec::from_json_file(
			path.into(),
		)?)),
	}
}

impl SubstrateCli for Cli {
	fn impl_name() -> String {
		"Moonbase Parachain Collator".into()
	}

	fn impl_version() -> String {
		env!("SUBSTRATE_CLI_IMPL_VERSION").into()
	}

	fn description() -> String {
		format!(
			"Moonbase Parachain Collator\n\nThe command-line arguments provided first will be \
		passed to the parachain node, while the arguments provided after -- will be passed \
		to the relaychain node.\n\n\
		{} [parachain-args] -- [relaychain-args]",
			Self::executable_name()
		)
	}

	fn author() -> String {
		env!("CARGO_PKG_AUTHORS").into()
	}

	fn support_url() -> String {
		"https://github.com/PureStake/moonbeam/issues/new".into()
	}

	fn copyright_start_year() -> i32 {
		2019
	}

	fn load_spec(&self, id: &str) -> std::result::Result<Box<dyn sc_service::ChainSpec>, String> {
		load_spec(id, self.run.parachain_id.unwrap_or(1000).into())
	}

	fn native_runtime_version(_: &Box<dyn ChainSpec>) -> &'static RuntimeVersion {
		&moonbeam_runtime::VERSION
	}
}

impl SubstrateCli for RelayChainCli {
	fn impl_name() -> String {
		"Moonbeam Parachain Collator".into()
	}

	fn impl_version() -> String {
		env!("SUBSTRATE_CLI_IMPL_VERSION").into()
	}

	fn description() -> String {
		"Moonbeam Parachain Collator\n\nThe command-line arguments provided first will be \
		passed to the parachain node, while the arguments provided after -- will be passed \
		to the relaychain node.\n\n\
		parachain-collator [parachain-args] -- [relaychain-args]"
			.into()
	}

	fn author() -> String {
		env!("CARGO_PKG_AUTHORS").into()
	}

	fn support_url() -> String {
		"https://github.com/PureStake/moonbeam/issues/new".into()
	}

	fn copyright_start_year() -> i32 {
		2019
	}

	fn load_spec(&self, id: &str) -> std::result::Result<Box<dyn sc_service::ChainSpec>, String> {
		match id {
			"moonbase_alpha_relay" => Ok(Box::new(RococoChainSpec::from_json_bytes(
				&include_bytes!("../../specs/alphanet/rococo-embedded-specs-v7.json")[..],
			)?)),
			"moonbase_stage_relay" => Ok(Box::new(RococoChainSpec::from_json_bytes(
				&include_bytes!("../../specs/stagenet/rococo-embedded-specs-v7.json")[..],
			)?)),
			// If we are not using a moonbeam-centric pre-baked relay spec, then fall back to the
			// Polkadot service to interpret the id.
			_ => polkadot_cli::Cli::from_iter([RelayChainCli::executable_name()].iter())
				.load_spec(id),
		}
	}

	fn native_runtime_version(chain_spec: &Box<dyn ChainSpec>) -> &'static RuntimeVersion {
		polkadot_cli::Cli::native_runtime_version(chain_spec)
	}
}

#[allow(clippy::borrowed_box)]
fn extract_genesis_wasm(chain_spec: &Box<dyn sc_service::ChainSpec>) -> Result<Vec<u8>> {
	let mut storage = chain_spec.build_storage()?;

	storage
		.top
		.remove(sp_core::storage::well_known_keys::CODE)
		.ok_or_else(|| "Could not find wasm file in genesis state!".into())
}

/// Parse command line arguments into service configuration.
pub fn run() -> Result<()> {
	let cli = Cli::from_args();
	match &cli.subcommand {
		Some(Subcommand::BuildSpec(params)) => {
			let runner = cli.create_runner(&params.base)?;
			runner.sync_run(|config| {
				if params.mnemonic.is_some() || params.accounts.is_some() {
					params.base.run(
						Box::new(chain_spec::development_chain_spec(
							params.mnemonic.clone(),
							params.accounts,
						)),
						config.network,
					)
				} else {
					params.base.run(config.chain_spec, config.network)
				}
			})
		}
		Some(Subcommand::CheckBlock(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|config| {
				// maybe these three lines could be a helper function is_dev(config) -> bool
				let extension = chain_spec::Extensions::try_get(&*config.chain_spec);
				let relay_chain_id = extension.map(|e| e.relay_chain.clone());
				let dev_service =
					cli.run.dev_service || relay_chain_id == Some("dev-service".to_string());

				let PartialComponents {
					client,
					task_manager,
					import_queue,
					..
				} = crate::service::new_partial(&config, None, dev_service)?;
				Ok((cmd.run(client, import_queue), task_manager))
			})
		}
		Some(Subcommand::ExportBlocks(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|config| {
				let PartialComponents {
					client,
					task_manager,
					..
				} = crate::service::new_partial(&config, None, false)?;
				Ok((cmd.run(client, config.database), task_manager))
			})
		}
		Some(Subcommand::ExportState(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|config| {
				let PartialComponents {
					client,
					task_manager,
					..
				} = crate::service::new_partial(&config, None, false)?;
				Ok((cmd.run(client, config.chain_spec), task_manager))
			})
		}
		Some(Subcommand::ImportBlocks(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|config| {
				let extension = chain_spec::Extensions::try_get(&*config.chain_spec);
				let relay_chain_id = extension.map(|e| e.relay_chain.clone());
				let dev_service =
					cli.run.dev_service || relay_chain_id == Some("dev-service".to_string());

				let PartialComponents {
					client,
					task_manager,
					import_queue,
					..
				} = crate::service::new_partial(&config, None, dev_service)?;
				Ok((cmd.run(client, import_queue), task_manager))
			})
		}
		Some(Subcommand::PurgeChain(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|config| cmd.run(config.database))
		}
		Some(Subcommand::Revert(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|config| {
				let PartialComponents {
					client,
					task_manager,
					backend,
					..
				} = crate::service::new_partial(&config, None, false)?;
				Ok((cmd.run(client, backend), task_manager))
			})
		}
		Some(Subcommand::ExportGenesisState(params)) => {
			let mut builder = sc_cli::LoggerBuilder::new("");
			builder.with_profiling(sc_tracing::TracingReceiver::Log, "");
			let _ = builder.init();

			let block: Block = generate_genesis_block(&load_spec(
				&params.chain.clone().unwrap_or_default(),
				params.parachain_id.into(),
			)?)?;
			let raw_header = block.header().encode();
			let output_buf = if params.raw {
				raw_header
			} else {
				format!("0x{:?}", HexDisplay::from(&block.header().encode())).into_bytes()
			};

			if let Some(output) = &params.output {
				std::fs::write(output, output_buf)?;
			} else {
				std::io::stdout().write_all(&output_buf)?;
			}

			Ok(())
		}
		Some(Subcommand::ExportGenesisWasm(params)) => {
			let mut builder = sc_cli::LoggerBuilder::new("");
			builder.with_profiling(sc_tracing::TracingReceiver::Log, "");
			let _ = builder.init();

			let raw_wasm_blob =
				extract_genesis_wasm(&cli.load_spec(&params.chain.clone().unwrap_or_default())?)?;
			let output_buf = if params.raw {
				raw_wasm_blob
			} else {
				format!("0x{:?}", HexDisplay::from(&raw_wasm_blob)).into_bytes()
			};

			if let Some(output) = &params.output {
				std::fs::write(output, output_buf)?;
			} else {
				std::io::stdout().write_all(&output_buf)?;
			}

			Ok(())
		}
		None => {
			let runner = cli.create_runner(&*cli.run)?;
			let collator = cli.run.base.validator || cli.collator;
			let author_id: Option<H160> = cli.run.author_id;
			if collator && author_id.is_none() {
				return Err("Collator nodes must specify an author account id".into());
			}

			runner
				.run_node_until_exit(|config| async move {
					let key = sp_core::Pair::generate().0;

					let extension = chain_spec::Extensions::try_get(&*config.chain_spec);
					let relay_chain_id = extension.map(|e| e.relay_chain.clone());
					let dev_service =
						cli.run.dev_service || relay_chain_id == Some("dev-service".to_string());
					let para_id = extension.map(|e| e.para_id);

					// If dev service was requested, start up manual or instant seal.
					// Otherwise continue with the normal parachain node.
					// Dev service can be requested in two ways.
					// 1. by providing the --dev-service flag to the CLI
					// 2. by specifying "dev-service" in the chain spec's "relay-chain" field.
					// NOTE: the --dev flag triggers the dev service by way of number 2
					if dev_service {
						// --dev implies --collator
						let collator = collator || cli.run.shared_params.dev;

						// If no author id was supplied, use the one that is staked at genesis
						// in the default development spec.
						let author_id = author_id.or_else(|| {
							Some(
								AccountId::from_str("6Be02d1d3665660d22FF9624b7BE0551ee1Ac91b")
									.expect("Gerald is a valid account"),
							)
						});

						return crate::service::new_dev(
							config,
							cli.run.sealing,
							author_id,
							collator,
							cli.run.ethapi,
						);
					}

					let polkadot_cli = RelayChainCli::new(
						config.base_path.as_ref().map(|x| x.path().join("polkadot")),
						relay_chain_id,
						[RelayChainCli::executable_name()]
							.iter()
							.chain(cli.relaychain_args.iter()),
					);

					let id = ParaId::from(cli.run.parachain_id.or(para_id).unwrap_or(1000));

					let parachain_account =
						AccountIdConversion::<polkadot_primitives::v0::AccountId>::into_account(
							&id,
						);

					let block: Block = generate_genesis_block(&config.chain_spec)
						.map_err(|e| format!("{:?}", e))?;
					let genesis_state =
						format!("0x{:?}", HexDisplay::from(&block.header().encode()));

					let task_executor = config.task_executor.clone();
					let polkadot_config = SubstrateCli::create_configuration(
						&polkadot_cli,
						&polkadot_cli,
						task_executor,
					)
					.map_err(|err| format!("Relay chain argument error: {}", err))?;

					info!("Parachain id: {:?}", id);
					info!("Parachain Account: {}", parachain_account);
					info!("Parachain genesis state: {}", genesis_state);
					info!("Is collating: {}", if collator { "yes" } else { "no" });

					crate::service::start_node(
						config,
						key,
						author_id,
						polkadot_config,
						id,
						collator,
						cli.run.ethapi,
					)
					.await
					.map(|r| r.0)
					.map_err(Into::into)
				})
				.map_err(Into::into)
		}
	}
}

impl DefaultConfigurationValues for RelayChainCli {
	fn p2p_listen_port() -> u16 {
		30334
	}

	fn rpc_ws_listen_port() -> u16 {
		9945
	}

	fn rpc_http_listen_port() -> u16 {
		9934
	}

	fn prometheus_listen_port() -> u16 {
		9616
	}
}

impl CliConfiguration<Self> for RelayChainCli {
	fn shared_params(&self) -> &SharedParams {
		self.base.base.shared_params()
	}

	fn import_params(&self) -> Option<&ImportParams> {
		self.base.base.import_params()
	}

	fn network_params(&self) -> Option<&NetworkParams> {
		self.base.base.network_params()
	}

	fn keystore_params(&self) -> Option<&KeystoreParams> {
		self.base.base.keystore_params()
	}

	fn base_path(&self) -> Result<Option<BasePath>> {
		Ok(self
			.shared_params()
			.base_path()
			.or_else(|| self.base_path.clone().map(Into::into)))
	}

	fn rpc_http(&self, default_listen_port: u16) -> Result<Option<SocketAddr>> {
		self.base.base.rpc_http(default_listen_port)
	}

	fn rpc_ipc(&self) -> Result<Option<String>> {
		self.base.base.rpc_ipc()
	}

	fn rpc_ws(&self, default_listen_port: u16) -> Result<Option<SocketAddr>> {
		self.base.base.rpc_ws(default_listen_port)
	}

	fn prometheus_config(&self, default_listen_port: u16) -> Result<Option<PrometheusConfig>> {
		self.base.base.prometheus_config(default_listen_port)
	}

	fn init<C: SubstrateCli>(&self) -> Result<()> {
		unreachable!("PolkadotCli is never initialized; qed");
	}

	fn chain_id(&self, is_dev: bool) -> Result<String> {
		let chain_id = self.base.base.chain_id(is_dev)?;

		Ok(if chain_id.is_empty() {
			self.chain_id.clone().unwrap_or_default()
		} else {
			chain_id
		})
	}

	fn role(&self, is_dev: bool) -> Result<sc_service::Role> {
		self.base.base.role(is_dev)
	}

	fn transaction_pool(&self) -> Result<sc_service::config::TransactionPoolOptions> {
		self.base.base.transaction_pool()
	}

	fn state_cache_child_ratio(&self) -> Result<Option<usize>> {
		self.base.base.state_cache_child_ratio()
	}

	fn rpc_methods(&self) -> Result<sc_service::config::RpcMethods> {
		self.base.base.rpc_methods()
	}

	fn rpc_ws_max_connections(&self) -> Result<Option<usize>> {
		self.base.base.rpc_ws_max_connections()
	}

	fn rpc_cors(&self, is_dev: bool) -> Result<Option<Vec<String>>> {
		self.base.base.rpc_cors(is_dev)
	}

	fn telemetry_external_transport(&self) -> Result<Option<sc_service::config::ExtTransport>> {
		self.base.base.telemetry_external_transport()
	}

	fn default_heap_pages(&self) -> Result<Option<u64>> {
		self.base.base.default_heap_pages()
	}

	fn force_authoring(&self) -> Result<bool> {
		self.base.base.force_authoring()
	}

	fn disable_grandpa(&self) -> Result<bool> {
		self.base.base.disable_grandpa()
	}

	fn max_runtime_instances(&self) -> Result<Option<usize>> {
		self.base.base.max_runtime_instances()
	}

	fn announce_block(&self) -> Result<bool> {
		self.base.base.announce_block()
	}
}
