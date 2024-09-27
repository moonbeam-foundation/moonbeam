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
// along with Moonbeam.  If not, see <http://www.gnu.org/licenses/>.

//! This module constructs and executes the appropriate service components for the given subcommand

use crate::cli::{Cli, RelayChainCli, RunCmd, Subcommand};
use cumulus_client_cli::extract_genesis_wasm;
use cumulus_primitives_core::ParaId;
use frame_benchmarking_cli::BenchmarkCmd;
use log::{info, warn};
use moonbeam_cli_opt::EthApi;
use moonbeam_service::{
	chain_spec, frontier_database_dir, moonbase_runtime, moonbeam_runtime, moonriver_runtime,
	HostFunctions, IdentifyVariant,
};
use parity_scale_codec::Encode;
#[cfg(feature = "westend-native")]
use polkadot_service::WestendChainSpec;
use sc_cli::{
	ChainSpec, CliConfiguration, DefaultConfigurationValues, ImportParams, KeystoreParams,
	NetworkParams, Result, RuntimeVersion, SharedParams, SubstrateCli,
};
use sc_service::{
	config::{BasePath, PrometheusConfig},
	DatabaseSource, PartialComponents,
};
use sp_core::hexdisplay::HexDisplay;
use sp_runtime::{
	traits::{
		AccountIdConversion, Block as BlockT, Hash as HashT, HashingFor, Header as HeaderT, Zero,
	},
	StateVersion,
};
use std::{io::Write, net::SocketAddr};

fn load_spec(
	id: &str,
	para_id: ParaId,
	run_cmd: &RunCmd,
) -> std::result::Result<Box<dyn sc_service::ChainSpec>, String> {
	Ok(match id {
		// Moonbase networks
		"moonbase-alpha" | "alphanet" => Box::new(chain_spec::RawChainSpec::from_json_bytes(
			&include_bytes!("../../../specs/alphanet/parachain-embedded-specs-v8.json")[..],
		)?),
		#[cfg(feature = "moonbase-native")]
		"moonbase-local" => Box::new(chain_spec::moonbase::get_chain_spec(para_id)),
		#[cfg(feature = "moonbase-native")]
		"moonbase-dev" | "dev" | "development" => {
			Box::new(chain_spec::moonbase::development_chain_spec(None, None))
		}
		#[cfg(all(feature = "test-spec", feature = "moonbeam-native"))]
		"staking" => Box::new(chain_spec::test_spec::staking_spec(para_id)),
		// Moonriver networks
		"moonriver" => Box::new(chain_spec::RawChainSpec::from_json_bytes(
			&include_bytes!("../../../specs/moonriver/parachain-embedded-specs.json")[..],
		)?),
		#[cfg(feature = "moonriver-native")]
		"moonriver-dev" => Box::new(chain_spec::moonriver::development_chain_spec(None, None)),
		#[cfg(feature = "moonriver-native")]
		"moonriver-local" => Box::new(chain_spec::moonriver::get_chain_spec(para_id)),

		// Moonbeam networks
		"moonbeam" | "" => Box::new(chain_spec::RawChainSpec::from_json_bytes(
			&include_bytes!("../../../specs/moonbeam/parachain-embedded-specs.json")[..],
		)?),
		#[cfg(feature = "moonbeam-native")]
		"moonbeam-dev" => Box::new(chain_spec::moonbeam::development_chain_spec(None, None)),
		#[cfg(feature = "moonbeam-native")]
		"moonbeam-local" => Box::new(chain_spec::moonbeam::get_chain_spec(para_id)),

		// Specs provided as json specify which runtime to use in their file name. For example,
		// `moonbeam-custom.json` uses the moonbeam runtime.
		// `moonbase-dev-workshop.json` uses the moonbase runtime.
		// If no magic strings match, then the moonbase runtime is used by default.
		// TODO explore CLI options to make this nicer. eg `--force-moonriver-runtime`
		path => {
			let path = std::path::PathBuf::from(path);

			let starts_with = |prefix: &str| {
				path.file_name()
					.and_then(|f| f.to_str().map(|s| s.starts_with(&prefix)))
					.unwrap_or(false)
			};

			if run_cmd.force_moonbase || starts_with("moonbase") {
				Box::new(chain_spec::moonbase::ChainSpec::from_json_file(path)?)
			} else if run_cmd.force_moonriver || starts_with("moonriver") {
				Box::new(chain_spec::moonriver::ChainSpec::from_json_file(path)?)
			} else {
				Box::new(chain_spec::moonbeam::ChainSpec::from_json_file(path)?)
			}
		}
	})
}

impl SubstrateCli for Cli {
	fn impl_name() -> String {
		"Moonbeam Parachain Collator".into()
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
		"https://github.com/moonbeam-foundation/moonbeam/issues/new".into()
	}

	fn copyright_start_year() -> i32 {
		2019
	}

	fn load_spec(&self, id: &str) -> std::result::Result<Box<dyn sc_service::ChainSpec>, String> {
		load_spec(id, self.run.parachain_id.unwrap_or(1000).into(), &self.run)
	}
}

impl Cli {
	fn runtime_version(spec: &Box<dyn sc_service::ChainSpec>) -> &'static RuntimeVersion {
		match spec {
			#[cfg(feature = "moonriver-native")]
			spec if spec.is_moonriver() => return &moonbeam_service::moonriver_runtime::VERSION,
			#[cfg(feature = "moonbeam-native")]
			spec if spec.is_moonbeam() => return &moonbeam_service::moonbeam_runtime::VERSION,
			#[cfg(feature = "moonbase-native")]
			_ => return &moonbeam_service::moonbase_runtime::VERSION,
			#[cfg(not(feature = "moonbase-native"))]
			_ => panic!("invalid chain spec"),
		}
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
		"https://github.com/moonbeam-foundation/moonbeam/issues/new".into()
	}

	fn copyright_start_year() -> i32 {
		2019
	}

	fn load_spec(&self, id: &str) -> std::result::Result<Box<dyn sc_service::ChainSpec>, String> {
		match id {
			#[cfg(feature = "westend-native")]
			"westend_moonbase_relay_testnet" => Ok(Box::new(WestendChainSpec::from_json_bytes(
				&include_bytes!("../../../specs/alphanet/westend-embedded-specs-v8.json")[..],
			)?)),
			// If we are not using a moonbeam-centric pre-baked relay spec, then fall back to the
			// Polkadot service to interpret the id.
			_ => polkadot_cli::Cli::from_iter([RelayChainCli::executable_name()].iter())
				.load_spec(id),
		}
	}
}

fn validate_trace_environment(cli: &Cli) -> Result<()> {
	if (cli.run.ethapi.contains(&EthApi::Debug) || cli.run.ethapi.contains(&EthApi::Trace))
		&& cli
			.run
			.base
			.base
			.import_params
			.wasm_runtime_overrides
			.is_none()
	{
		return Err(
			"`debug` or `trace` namespaces requires `--wasm-runtime-overrides /path/to/overrides`."
				.into(),
		);
	}
	Ok(())
}

/// Parse command line arguments into service configuration.
pub fn run() -> Result<()> {
	let mut cli = Cli::from_args();
	let _ = validate_trace_environment(&cli)?;
	// Set --execution wasm as default
	let execution_strategies = cli.run.base.base.import_params.execution_strategies.clone();
	if execution_strategies.execution.is_none() {
		cli.run
			.base
			.base
			.import_params
			.execution_strategies
			.execution = Some(sc_cli::ExecutionStrategy::Wasm);
	}

	match &cli.subcommand {
		Some(Subcommand::BuildSpec(params)) => {
			let runner = cli.create_runner(&params.base)?;
			runner.sync_run(|config| {
				if params.mnemonic.is_some() || params.accounts.is_some() {
					if config.chain_spec.is_moonbeam() {
						params.base.run(
							Box::new(chain_spec::moonbeam::development_chain_spec(
								params.mnemonic.clone(),
								params.accounts,
							)),
							config.network,
						)
					} else if config.chain_spec.is_moonriver() {
						params.base.run(
							Box::new(chain_spec::moonriver::development_chain_spec(
								params.mnemonic.clone(),
								params.accounts,
							)),
							config.network,
						)
					} else {
						params.base.run(
							Box::new(chain_spec::moonbase::development_chain_spec(
								params.mnemonic.clone(),
								params.accounts,
							)),
							config.network,
						)
					}
				} else {
					params.base.run(config.chain_spec, config.network)
				}
			})
		}
		Some(Subcommand::CheckBlock(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			let rpc_config = cli.run.new_rpc_config();
			runner.async_run(|mut config| {
				let (client, _, import_queue, task_manager) =
					moonbeam_service::new_chain_ops(&mut config, &rpc_config)?;
				Ok((cmd.run(client, import_queue), task_manager))
			})
		}
		Some(Subcommand::ExportBlocks(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			let rpc_config = cli.run.new_rpc_config();
			runner.async_run(|mut config| {
				let (client, _, _, task_manager) =
					moonbeam_service::new_chain_ops(&mut config, &rpc_config)?;
				Ok((cmd.run(client, config.database), task_manager))
			})
		}
		Some(Subcommand::ExportState(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			let rpc_config = cli.run.new_rpc_config();
			runner.async_run(|mut config| {
				let (client, _, _, task_manager) =
					moonbeam_service::new_chain_ops(&mut config, &rpc_config)?;
				Ok((cmd.run(client, config.chain_spec), task_manager))
			})
		}
		Some(Subcommand::ImportBlocks(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			let rpc_config = cli.run.new_rpc_config();
			runner.async_run(|mut config| {
				let (client, _, import_queue, task_manager) =
					moonbeam_service::new_chain_ops(&mut config, &rpc_config)?;
				Ok((cmd.run(client, import_queue), task_manager))
			})
		}
		Some(Subcommand::PurgeChain(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|config| {
				// Although the cumulus_client_cli::PurgeCommand will extract the relay chain id,
				// we need to extract it here to determine whether we are running the dev service.
				let extension = chain_spec::Extensions::try_get(&*config.chain_spec);
				let relay_chain_id = extension.map(|e| e.relay_chain.as_str());
				let dev_service = cli.run.dev_service || relay_chain_id == Some("dev-service");

				// Remove Frontier offchain db
				let frontier_database_config = match config.database {
					DatabaseSource::RocksDb { .. } => DatabaseSource::RocksDb {
						path: frontier_database_dir(&config, "db"),
						cache_size: 0,
					},
					DatabaseSource::ParityDb { .. } => DatabaseSource::ParityDb {
						path: frontier_database_dir(&config, "paritydb"),
					},
					_ => {
						return Err(format!("Cannot purge `{:?}` database", config.database).into())
					}
				};
				cmd.base.run(frontier_database_config)?;

				if dev_service {
					// base refers to the encapsulated "regular" sc_cli::PurgeChain command
					return cmd.base.run(config.database);
				}

				let polkadot_cli = RelayChainCli::new(
					&config,
					[RelayChainCli::executable_name().to_string()]
						.iter()
						.chain(cli.relaychain_args.iter()),
				);

				let polkadot_config = SubstrateCli::create_configuration(
					&polkadot_cli,
					&polkadot_cli,
					config.tokio_handle.clone(),
				)
				.map_err(|err| format!("Relay chain argument error: {}", err))?;

				cmd.run(config, polkadot_config)
			})
		}
		Some(Subcommand::Revert(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			let chain_spec = &runner.config().chain_spec;
			let rpc_config = cli.run.new_rpc_config();
			match chain_spec {
				#[cfg(feature = "moonriver-native")]
				spec if spec.is_moonriver() => runner.async_run(|mut config| {
					let params = moonbeam_service::new_partial::<
						moonbeam_service::moonriver_runtime::RuntimeApi,
						moonbeam_service::MoonriverCustomizations,
					>(&mut config, &rpc_config, false)?;

					Ok((
						cmd.run(params.client, params.backend, None),
						params.task_manager,
					))
				}),
				#[cfg(feature = "moonbeam-native")]
				spec if spec.is_moonbeam() => runner.async_run(|mut config| {
					let params = moonbeam_service::new_partial::<
						moonbeam_service::moonbeam_runtime::RuntimeApi,
						moonbeam_service::MoonbeamCustomizations,
					>(&mut config, &rpc_config, false)?;

					Ok((
						cmd.run(params.client, params.backend, None),
						params.task_manager,
					))
				}),
				#[cfg(feature = "moonbase-native")]
				_ => runner.async_run(|mut config| {
					let params = moonbeam_service::new_partial::<
						moonbeam_service::moonbase_runtime::RuntimeApi,
						moonbeam_service::MoonbaseCustomizations,
					>(&mut config, &rpc_config, false)?;

					Ok((
						cmd.run(params.client, params.backend, None),
						params.task_manager,
					))
				}),
				#[cfg(not(feature = "moonbase-native"))]
				_ => panic!("invalid chain spec"),
			}
		}
		Some(Subcommand::ExportGenesisHead(params)) => {
			let mut builder = sc_cli::LoggerBuilder::new("");
			builder.with_profiling(sc_tracing::TracingReceiver::Log, "");
			let _ = builder.init();

			// Cumulus approach here, we directly call the generic load_spec func
			let chain_spec = load_spec(
				params.chain.as_deref().unwrap_or_default(),
				params.parachain_id.unwrap_or(1000).into(),
				&cli.run,
			)?;
			let state_version = Cli::runtime_version(&chain_spec).state_version();

			let output_buf = match chain_spec {
				#[cfg(feature = "moonriver-native")]
				chain_spec if chain_spec.is_moonriver() => {
					let block: moonbeam_service::moonriver_runtime::Block =
						generate_genesis_block(&*chain_spec, state_version)?;
					let raw_header = block.header().encode();
					let output_buf = if params.raw {
						raw_header
					} else {
						format!("0x{:?}", HexDisplay::from(&block.header().encode())).into_bytes()
					};
					output_buf
				}
				#[cfg(feature = "moonbeam-native")]
				chain_spec if chain_spec.is_moonbeam() => {
					let block: moonbeam_service::moonbeam_runtime::Block =
						generate_genesis_block(&*chain_spec, state_version)?;
					let raw_header = block.header().encode();
					let output_buf = if params.raw {
						raw_header
					} else {
						format!("0x{:?}", HexDisplay::from(&block.header().encode())).into_bytes()
					};
					output_buf
				}
				#[cfg(feature = "moonbase-native")]
				_ => {
					let block: moonbeam_service::moonbase_runtime::Block =
						generate_genesis_block(&*chain_spec, state_version)?;
					let raw_header = block.header().encode();
					let output_buf = if params.raw {
						raw_header
					} else {
						format!("0x{:?}", HexDisplay::from(&block.header().encode())).into_bytes()
					};
					output_buf
				}
				#[cfg(not(feature = "moonbase-native"))]
				_ => panic!("invalid chain spec"),
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

			let raw_wasm_blob = extract_genesis_wasm(
				&*cli.load_spec(params.chain.as_deref().unwrap_or_default())?,
			)?;
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
		Some(Subcommand::Benchmark(cmd)) => {
			let runner = cli.create_runner(cmd)?;

			// Switch on the concrete benchmark sub-command
			match cmd {
				BenchmarkCmd::Pallet(cmd) => {
					if cfg!(feature = "runtime-benchmarks") {
						let chain_spec = &runner.config().chain_spec;
						match chain_spec {
							#[cfg(feature = "moonriver-native")]
							spec if spec.is_moonriver() => {
								return runner.sync_run(|config| {
									cmd.run_with_spec::<HashingFor<moonriver_runtime::Block>, HostFunctions>(
										Some(config.chain_spec),
									)
								})
							}
							#[cfg(feature = "moonbeam-native")]
							spec if spec.is_moonbeam() => {
								return runner.sync_run(|config| {
									cmd.run_with_spec::<HashingFor<moonbeam_runtime::Block>, HostFunctions>(
										Some(config.chain_spec),
									)
								})
							}
							#[cfg(feature = "moonbase-native")]
							_ => {
								return runner.sync_run(|config| {
									cmd.run_with_spec::<HashingFor<moonbase_runtime::Block>, HostFunctions>(
										Some(config.chain_spec),
									)
								})
							}
							#[cfg(not(feature = "moonbase-native"))]
							_ => panic!("invalid chain spec"),
						}
					} else if cfg!(feature = "moonbase-runtime-benchmarks") {
						return runner.sync_run(|config| {
							cmd.run_with_spec::<HashingFor<moonbeam_service::moonbase_runtime::Block>, HostFunctions>(
								Some(config.chain_spec),
							)
						});
					} else {
						Err("Benchmarking wasn't enabled when building the node. \
					You can enable it with `--features runtime-benchmarks`."
							.into())
					}
				}
				BenchmarkCmd::Block(cmd) => {
					let chain_spec = &runner.config().chain_spec;
					let rpc_config = cli.run.new_rpc_config();
					match chain_spec {
						#[cfg(feature = "moonriver-native")]
						spec if spec.is_moonriver() => {
							return runner.sync_run(|mut config| {
								let params = moonbeam_service::new_partial::<
									moonbeam_service::moonriver_runtime::RuntimeApi,
									moonbeam_service::MoonriverCustomizations,
								>(&mut config, &rpc_config, false)?;

								cmd.run(params.client)
							})
						}
						#[cfg(feature = "moonbeam-native")]
						spec if spec.is_moonbeam() => {
							return runner.sync_run(|mut config| {
								let params = moonbeam_service::new_partial::<
									moonbeam_service::moonbeam_runtime::RuntimeApi,
									moonbeam_service::MoonbeamCustomizations,
								>(&mut config, &rpc_config, false)?;

								cmd.run(params.client)
							})
						}
						#[cfg(feature = "moonbase-native")]
						_ => {
							return runner.sync_run(|mut config| {
								let params = moonbeam_service::new_partial::<
									moonbeam_service::moonbase_runtime::RuntimeApi,
									moonbeam_service::MoonbaseCustomizations,
								>(&mut config, &rpc_config, false)?;

								cmd.run(params.client)
							})
						}
						#[cfg(not(feature = "moonbase-native"))]
						_ => panic!("invalid chain spec"),
					}
				}
				#[cfg(not(feature = "runtime-benchmarks"))]
				BenchmarkCmd::Storage(_) => Err(
					"Storage benchmarking can be enabled with `--features runtime-benchmarks`."
						.into(),
				),
				#[cfg(feature = "runtime-benchmarks")]
				BenchmarkCmd::Storage(cmd) => {
					let chain_spec = &runner.config().chain_spec;
					let rpc_config = cli.run.new_rpc_config();
					match chain_spec {
						#[cfg(feature = "moonriver-native")]
						spec if spec.is_moonriver() => {
							return runner.sync_run(|mut config| {
								let params = moonbeam_service::new_partial::<
									moonbeam_service::moonriver_runtime::RuntimeApi,
									moonbeam_service::MoonriverCustomizations,
								>(&mut config, &rpc_config, false)?;

								let db = params.backend.expose_db();
								let storage = params.backend.expose_storage();

								cmd.run(config, params.client, db, storage)
							})
						}
						#[cfg(feature = "moonbeam-native")]
						spec if spec.is_moonbeam() => {
							return runner.sync_run(|mut config| {
								let params = moonbeam_service::new_partial::<
									moonbeam_service::moonbeam_runtime::RuntimeApi,
									moonbeam_service::MoonbeamCustomizations,
								>(&mut config, &rpc_config, false)?;

								let db = params.backend.expose_db();
								let storage = params.backend.expose_storage();

								cmd.run(config, params.client, db, storage)
							})
						}
						#[cfg(feature = "moonbase-native")]
						_ => {
							return runner.sync_run(|mut config| {
								let params = moonbeam_service::new_partial::<
									moonbeam_service::moonbase_runtime::RuntimeApi,
									moonbeam_service::MoonbaseCustomizations,
								>(&mut config, &rpc_config, false)?;

								let db = params.backend.expose_db();
								let storage = params.backend.expose_storage();

								cmd.run(config, params.client, db, storage)
							})
						}
						#[cfg(not(feature = "moonbase-native"))]
						_ => panic!("invalid chain spec"),
					}
				}
				BenchmarkCmd::Overhead(_) => Err("Unsupported benchmarking command".into()),
				BenchmarkCmd::Extrinsic(_) => Err("Unsupported benchmarking command".into()),
				BenchmarkCmd::Machine(cmd) => {
					return runner.sync_run(|config| {
						cmd.run(
							&config,
							frame_benchmarking_cli::SUBSTRATE_REFERENCE_HARDWARE.clone(),
						)
					});
				}
			}
		}
		Some(Subcommand::TryRuntime) => Err("The `try-runtime` subcommand has been migrated to a \
			standalone CLI (https://github.com/paritytech/try-runtime-cli). It is no longer \
			being maintained here and will be removed entirely some time after January 2024. \
			Please remove this subcommand from your runtime and use the standalone CLI."
			.into()),
		Some(Subcommand::Key(cmd)) => Ok(cmd.run(&cli)?),
		Some(Subcommand::PrecompileWasm(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			let rpc_config = cli.run.new_rpc_config();
			runner.async_run(|mut config| match &config.chain_spec {
				#[cfg(feature = "moonriver-native")]
				spec if spec.is_moonriver() => {
					let PartialComponents {
						task_manager,
						backend,
						..
					} = moonbeam_service::new_partial::<
						moonbeam_service::moonriver_runtime::RuntimeApi,
						moonbeam_service::MoonriverCustomizations,
					>(&mut config, &rpc_config, false)?;

					Ok((cmd.run(backend, config.chain_spec), task_manager))
				}
				#[cfg(feature = "moonbeam-native")]
				spec if spec.is_moonbeam() => {
					let PartialComponents {
						task_manager,
						backend,
						..
					} = moonbeam_service::new_partial::<
						moonbeam_service::moonbeam_runtime::RuntimeApi,
						moonbeam_service::MoonbeamCustomizations,
					>(&mut config, &rpc_config, false)?;

					Ok((cmd.run(backend, config.chain_spec), task_manager))
				}
				#[cfg(feature = "moonbase-native")]
				_ => {
					let PartialComponents {
						task_manager,
						backend,
						..
					} = moonbeam_service::new_partial::<
						moonbeam_service::moonbase_runtime::RuntimeApi,
						moonbeam_service::MoonbaseCustomizations,
					>(&mut config, &rpc_config, false)?;

					Ok((cmd.run(backend, config.chain_spec), task_manager))
				}
				#[cfg(not(feature = "moonbase-native"))]
				_ => panic!("invalid chain spec"),
			})
		}
		None => {
			let runner = cli.create_runner(&(*cli.run).normalize())?;
			let collator_options = cli.run.collator_options();

			// It is used when feature "lazy-loading" is enabled
			#[allow(unused_mut)]
			runner.run_node_until_exit(|mut config| async move {
				let hwbench = if !cli.run.no_hardware_benchmarks {
					config.database.path().map(|database_path| {
						let _ = std::fs::create_dir_all(&database_path);
						sc_sysinfo::gather_hwbench(Some(database_path))
					})
				} else {
					None
				};

				let extension = chain_spec::Extensions::try_get(&*config.chain_spec);

				let rpc_config = cli.run.new_rpc_config();

				#[cfg(feature = "lazy-loading")]
				if let Some(fork_chain_from_rpc) = cli.run.fork_chain_from_rpc {
					// When running the dev service, just use Alice's author inherent
					//TODO maybe make the --alice etc flags work here, and consider bringing back
					// the author-id flag. For now, this will work.
					let author_id = Some(chain_spec::get_from_seed::<nimbus_primitives::NimbusId>(
						"Alice",
					));

					let lazy_loading_config = moonbeam_cli_opt::LazyLoadingConfig {
						state_rpc: fork_chain_from_rpc,
						from_block: cli.run.block,
						state_overrides_path: cli.run.fork_state_overrides,
						runtime_override: cli.run.runtime_override,
					};

					let spec_builder =
						chain_spec::test_spec::lazy_loading_spec_builder(Default::default());
					config.chain_spec = Box::new(spec_builder.build());

					// TODO: create a tokio runtime inside offchain_worker thread (otherwise it will panic)
					// We just disable it for now, since it is not needed
					config.offchain_worker.enabled = false;

					return moonbeam_service::lazy_loading::new_lazy_loading_service::<
						moonbeam_runtime::RuntimeApi,
						moonbeam_service::MoonbeamCustomizations,
						sc_network::NetworkWorker<_, _>,
					>(
						config,
						author_id,
						cli.run.sealing,
						rpc_config,
						lazy_loading_config,
						hwbench,
					)
					.await
					.map_err(Into::into);
				}
				#[cfg(not(feature = "lazy-loading"))]
				{
					// If dev service was requested, start up manual or instant seal.
					// Otherwise continue with the normal parachain node.
					// Dev service can be requested in two ways.
					// 1. by providing the --dev-service flag to the CLI
					// 2. by specifying "dev-service" in the chain spec's "relay-chain" field.
					// NOTE: the --dev flag triggers the dev service by way of number 2
					let relay_chain_id = extension.map(|e| e.relay_chain.as_str());
					let dev_service = cli.run.dev_service
						|| config.chain_spec.is_dev()
						|| relay_chain_id == Some("dev-service");
					if dev_service {
						// When running the dev service, just use Alice's author inherent
						//TODO maybe make the --alice etc flags work here, and consider bringing back
						// the author-id flag. For now, this will work.
						let author_id = Some(chain_spec::get_from_seed::<
							nimbus_primitives::NimbusId,
						>("Alice"));

						return match &config.chain_spec {
							#[cfg(feature = "moonriver-native")]
							spec if spec.is_moonriver() => moonbeam_service::new_dev::<
								moonbeam_service::moonriver_runtime::RuntimeApi,
								moonbeam_service::MoonriverCustomizations,
								sc_network::NetworkWorker<_, _>,
							>(config, author_id, cli.run.sealing, rpc_config, hwbench)
							.await
							.map_err(Into::into),
							#[cfg(feature = "moonbeam-native")]
							spec if spec.is_moonbeam() => moonbeam_service::new_dev::<
								moonbeam_service::moonbeam_runtime::RuntimeApi,
								moonbeam_service::MoonbeamCustomizations,
								sc_network::NetworkWorker<_, _>,
							>(config, author_id, cli.run.sealing, rpc_config, hwbench)
							.await
							.map_err(Into::into),
							#[cfg(feature = "moonbase-native")]
							_ => moonbeam_service::new_dev::<
								moonbeam_service::moonbase_runtime::RuntimeApi,
								moonbeam_service::MoonbaseCustomizations,
								sc_network::NetworkWorker<_, _>,
							>(config, author_id, cli.run.sealing, rpc_config, hwbench)
							.await
							.map_err(Into::into),
							#[cfg(not(feature = "moonbase-native"))]
							_ => panic!("invalid chain spec"),
						};
					}
				}

				let polkadot_cli = RelayChainCli::new(
					&config,
					[RelayChainCli::executable_name().to_string()]
						.iter()
						.chain(cli.relaychain_args.iter()),
				);

				let para_id = extension.map(|e| e.para_id);
				let id = ParaId::from(cli.run.parachain_id.clone().or(para_id).unwrap_or(1000));

				let parachain_account =
					AccountIdConversion::<polkadot_primitives::v7::AccountId>::into_account_truncating(&id);

				let tokio_handle = config.tokio_handle.clone();
				let polkadot_config =
					SubstrateCli::create_configuration(&polkadot_cli, &polkadot_cli, tokio_handle)
						.map_err(|err| format!("Relay chain argument error: {}", err))?;

				info!("Parachain Account: {}", parachain_account);
				info!(
					"Is collating: {}",
					if config.role.is_authority() {
						"yes"
					} else {
						"no"
					}
				);

				if !rpc_config.relay_chain_rpc_urls.is_empty() && cli.relaychain_args.len() > 0 {
					warn!(
						"Detected relay chain node arguments together with \
					--relay-chain-rpc-url. This command starts a minimal Polkadot node that only \
					uses a network-related subset of all relay chain CLI options."
					);
				}

				match &config.chain_spec {
					#[cfg(feature = "moonriver-native")]
					spec if spec.is_moonriver() => moonbeam_service::start_node::<
						moonbeam_service::moonriver_runtime::RuntimeApi,
						moonbeam_service::MoonriverCustomizations,
					>(
						config,
						polkadot_config,
						collator_options,
						id,
						rpc_config,
						true,
						cli.run.block_authoring_duration,
						hwbench,
					)
					.await
					.map(|r| r.0)
					.map_err(Into::into),
					#[cfg(feature = "moonbeam-native")]
					spec if spec.is_moonbeam() => moonbeam_service::start_node::<
						moonbeam_service::moonbeam_runtime::RuntimeApi,
						moonbeam_service::MoonbeamCustomizations,
					>(
						config,
						polkadot_config,
						collator_options,
						id,
						rpc_config,
						true,
						cli.run.block_authoring_duration,
						hwbench,
					)
					.await
					.map(|r| r.0)
					.map_err(Into::into),
					#[cfg(feature = "moonbase-native")]
					_ => moonbeam_service::start_node::<
						moonbeam_service::moonbase_runtime::RuntimeApi,
						moonbeam_service::MoonbaseCustomizations,
					>(
						config,
						polkadot_config,
						collator_options,
						id,
						rpc_config,
						true,
						cli.run.block_authoring_duration,
						hwbench,
					)
					.await
					.map(|r| r.0)
					.map_err(Into::into),
					#[cfg(not(feature = "moonbase-native"))]
					_ => panic!("invalid chain spec"),
				}
			})
		}
	}
}

impl DefaultConfigurationValues for RelayChainCli {
	fn p2p_listen_port() -> u16 {
		30334
	}

	fn rpc_listen_port() -> u16 {
		9945
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
			.base_path()?
			.or_else(|| Some(self.base_path.clone().into())))
	}

	fn rpc_addr(&self, default_listen_port: u16) -> Result<Option<SocketAddr>> {
		self.base.base.rpc_addr(default_listen_port)
	}

	fn prometheus_config(
		&self,
		default_listen_port: u16,
		chain_spec: &Box<dyn ChainSpec>,
	) -> Result<Option<PrometheusConfig>> {
		self.base
			.base
			.prometheus_config(default_listen_port, chain_spec)
	}

	fn init<F>(
		&self,
		_support_url: &String,
		_impl_version: &String,
		_logger_hook: F,
		_config: &sc_service::Configuration,
	) -> Result<()>
	where
		F: FnOnce(&mut sc_cli::LoggerBuilder, &sc_service::Configuration),
	{
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

	fn transaction_pool(&self, is_dev: bool) -> Result<sc_service::config::TransactionPoolOptions> {
		self.base.base.transaction_pool(is_dev)
	}

	fn rpc_methods(&self) -> Result<sc_service::config::RpcMethods> {
		self.base.base.rpc_methods()
	}

	fn rpc_max_connections(&self) -> Result<u32> {
		self.base.base.rpc_max_connections()
	}

	fn rpc_cors(&self, is_dev: bool) -> Result<Option<Vec<String>>> {
		self.base.base.rpc_cors(is_dev)
	}

	// fn telemetry_external_transport(&self) -> Result<Option<sc_service::config::ExtTransport>> {
	// 	self.base.base.telemetry_external_transport()
	// }

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

/// Generate the genesis block from a given ChainSpec.
pub fn generate_genesis_block<Block: BlockT>(
	chain_spec: &dyn ChainSpec,
	genesis_state_version: StateVersion,
) -> std::result::Result<Block, String> {
	let storage = chain_spec.build_storage()?;

	let child_roots = storage.children_default.iter().map(|(sk, child_content)| {
		let state_root = <<<Block as BlockT>::Header as HeaderT>::Hashing as HashT>::trie_root(
			child_content.data.clone().into_iter().collect(),
			genesis_state_version,
		);
		(sk.clone(), state_root.encode())
	});
	let state_root = <<<Block as BlockT>::Header as HeaderT>::Hashing as HashT>::trie_root(
		storage.top.clone().into_iter().chain(child_roots).collect(),
		genesis_state_version,
	);

	let extrinsics_root = <<<Block as BlockT>::Header as HeaderT>::Hashing as HashT>::trie_root(
		Vec::new(),
		genesis_state_version,
	);

	Ok(Block::new(
		<<Block as BlockT>::Header as HeaderT>::new(
			Zero::zero(),
			extrinsics_root,
			state_root,
			Default::default(),
			Default::default(),
		),
		Default::default(),
	))
}
