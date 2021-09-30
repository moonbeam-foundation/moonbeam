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

use crate::{PerfCmd, txn_signer::UnsignedTransaction};

use sp_runtime::transaction_validity::TransactionSource;
use sc_service::{
	Configuration, NativeExecutionDispatch, TFullClient, TFullBackend, TaskManager, TransactionPool,
};
use sc_cli::{
	CliConfiguration, Result as CliResult, SharedParams,
};
use sp_core::{H160, H256, U256};
use sc_client_api::HeaderBackend;
use sp_api::{ConstructRuntimeApi, ProvideRuntimeApi, BlockId};
use std::{sync::Arc, marker::PhantomData};
use fp_rpc::{EthereumRuntimeRPCApi, ConvertTransaction};
use nimbus_primitives::NimbusId;
use cumulus_primitives_parachain_inherent::MockValidationDataInherentDataProvider;
use sc_consensus_manual_seal::{run_manual_seal, EngineCommand, ManualSealParams, CreatedBlock};
use ethereum::TransactionAction;

use futures::{
	Stream, SinkExt,
	channel::{
		oneshot,
		mpsc,
	},
};

use service::{chain_spec, RuntimeApiCollection, Block};
use sha3::{Digest, Keccak256};

type FullClient<RuntimeApi, Executor> = TFullClient<Block, RuntimeApi, Executor>;
type FullBackend = TFullBackend<Block>;

const EXTRINSIC_GAS_LIMIT: u64 = 12_995_000;
const MIN_GAS_PRICE: u64 = 1_000_000_000;

struct PerfTestRunner<RuntimeApi, Executor>
	where
		RuntimeApi:
			ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>> + Send + Sync + 'static,
		RuntimeApi::RuntimeApi:
			RuntimeApiCollection<StateBackend = sc_client_api::StateBackendFor<FullBackend, Block>>,
		Executor: NativeExecutionDispatch + 'static,
{
	task_manager: TaskManager,
	client: Arc<TFullClient<Block, RuntimeApi, Executor>>,
	manual_seal_command_sink: mpsc::Sender<EngineCommand<H256>>,
	pool: Arc<sc_transaction_pool::FullPool<Block, FullClient<RuntimeApi, Executor>>>,

	_marker1: PhantomData<RuntimeApi>,
	_marker2: PhantomData<Executor>,
}

// TODO: am I abusing the name "runner"?
impl<RuntimeApi, Executor> PerfTestRunner<RuntimeApi, Executor>
	where
		RuntimeApi:
			ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>> + Send + Sync + 'static,
		RuntimeApi::RuntimeApi:
			RuntimeApiCollection<StateBackend = sc_client_api::StateBackendFor<FullBackend, Block>>,
		Executor: NativeExecutionDispatch + 'static,
{
	fn from_cmd(config: &Configuration, cmd: &PerfCmd) -> CliResult<Self> {
		let sc_service::PartialComponents {
			client,
			backend,
			mut task_manager,
			import_queue,
			keystore_container,
			select_chain: maybe_select_chain,
			transaction_pool,
			other:
				(
					block_import,
					pending_transactions,
					filter_pool,
					telemetry,
					_telemetry_worker_handle,
					frontier_backend,
				),
		} = service::new_partial::<RuntimeApi, Executor>(&config, true)?;

		// TODO: review -- we don't need any actual networking
		let (_network, _system_rpc_tx, network_starter) =
			sc_service::build_network(sc_service::BuildNetworkParams {
				config: &config,
				client: client.clone(),
				transaction_pool: transaction_pool.clone(),
				spawn_handle: task_manager.spawn_handle(),
				import_queue,
				on_demand: None,
				block_announce_validator_builder: None,
				warp_sync: None,
			})?;

		// TODO: maybe offchain worker needed?

		let author_id = chain_spec::get_from_seed::<NimbusId>("Alice");

		// TODO: no need for prometheus here...
		let prometheus_registry = config.prometheus_registry().cloned();

		let env = sc_basic_authorship::ProposerFactory::new(
			task_manager.spawn_handle(),
			client.clone(),
			transaction_pool.clone(),
			prometheus_registry.as_ref(),
			telemetry.as_ref().map(|x| x.handle()),
		);

		let mut command_sink = None;
		let command_stream: Box<dyn Stream<Item = EngineCommand<H256>> + Send + Sync + Unpin> = {
			let (sink, stream) = mpsc::channel(1000);
			command_sink = Some(sink);
			Box::new(stream)
		};

		let select_chain = maybe_select_chain.expect(
			"`new_partial` builds a `LongestChainRule` when building dev service.\
				We specified the dev service when calling `new_partial`.\
				Therefore, a `LongestChainRule` is present. qed.",
		);

		let client_set_aside_for_cidp = client.clone();


		log::debug!("spawning authorship task...");
		task_manager.spawn_essential_handle().spawn_blocking(
			"authorship_task",
			run_manual_seal(ManualSealParams {
				block_import,
				env,
				client: client.clone(),
				pool: transaction_pool.clone(),
				commands_stream: command_stream,
				select_chain,
				consensus_data_provider: None,
				create_inherent_data_providers: move |block: H256, ()| {
					let current_para_block = client_set_aside_for_cidp
						.number(block)
						.expect("Header lookup should succeed")
						.expect("Header passed in as parent should be present in backend.");
					let author_id = author_id.clone();

					async move {
						let time = sp_timestamp::InherentDataProvider::from_system_time();

						let mocked_parachain = MockValidationDataInherentDataProvider {
							current_para_block,
							relay_offset: 1000,
							relay_blocks_per_para_block: 2,
						};

						let author = nimbus_primitives::InherentDataProvider::<NimbusId>(author_id);

						Ok((time, mocked_parachain, author))
					}
				},
			}),
		);

		service::rpc::spawn_essential_tasks(service::rpc::SpawnTasksParams {
			task_manager: &task_manager,
			client: client.clone(),
			substrate_backend: backend.clone(),
			frontier_backend: frontier_backend.clone(),
			pending_transactions: pending_transactions.clone(),
			filter_pool: filter_pool.clone(),
		});

		network_starter.start_network();

		Ok(PerfTestRunner {
			task_manager,
			client: client.clone(),
			manual_seal_command_sink: command_sink.unwrap(),
			pool: transaction_pool,
			_marker1: Default::default(),
			_marker2: Default::default(),
		})
	}

	fn evm_call(
		&mut self,
		from: H160,
		to: H160,
		data: Vec<u8>,
		value: U256,
		gas_limit: U256,
		gas_price: Option<U256>,
		nonce: Option<U256>,
		estimate: bool,
	) -> Result<fp_evm::CallInfo, sp_runtime::DispatchError> {
		let hash = self.client.info().best_hash;
		log::info!("evm_call best_hash: {:?}", hash);

		let result = self.client.runtime_api().call(
			&BlockId::Hash(hash),
			from,
			to,
			data,
			value,
			gas_limit,
			gas_price,
			nonce,
			false,
		);

		result.expect("why is this a Result<Result<...>>???") // TODO
	}

	fn evm_create(
		&mut self,
		from: H160,
		data: Vec<u8>,
		value: U256,
		gas_limit: U256,
		gas_price: Option<U256>,
		nonce: Option<U256>,
		estimate: bool,
	) -> Result<fp_evm::CreateInfo, sp_runtime::DispatchError> {
		let hash = self.client.info().best_hash;
		log::info!("evm_create best_hash: {:?}", hash);

		let result = self.client.runtime_api().create(
			&BlockId::Hash(hash),
			from,
			data,
			value,
			gas_limit,
			gas_price,
			nonce,
			false,
		);

		result.expect("why is this a Result<Result<...>>???") // TODO
	}

	/// Creates a transaction out of the given call/create arguments, signs it, and sends it
	fn eth_sign_and_send_transaction(
		&mut self,
		signing_key: &H256,
		to: Option<H160>,
		data: Vec<u8>,
		value: U256,
		gas_limit: U256,
		gas_price: U256,
		nonce: U256,
	) -> Result<H256, sp_runtime::DispatchError> {

		const CHAIN_ID: u64 = 1281; // TODO: derive from CLI or from Moonbase

		let action = match to {
			Some(addr) => TransactionAction::Call(addr),
			None => TransactionAction::Create,
		};

		let unsigned = UnsignedTransaction {
			chain_id: CHAIN_ID,
			nonce,
			gas_price,
			gas_limit,
			action,
			value,
			input: data,
		};
		let signed = unsigned.sign(signing_key);

		let transaction_hash =
			H256::from_slice(Keccak256::digest(&rlp::encode(&signed)).as_slice());

		let transaction_converter = moonbase_runtime::TransactionConverter;
		let unchecked_extrinsic = transaction_converter.convert_transaction(signed);

		let hash = self.client.info().best_hash;
		log::debug!("eth_sign_and_send_transaction best_hash: {:?}", hash);
		let future = self.pool.submit_one(
			&BlockId::hash(hash),
			TransactionSource::Local,
			unchecked_extrinsic
		);

		futures::executor::block_on(future);

		Ok(transaction_hash)

	}

	/// Author a block through manual sealing
	fn create_block(&mut self, create_empty: bool) ->
		CreatedBlock<H256>
	{
		log::debug!("Issuing seal command...");
		let hash = self.client.info().best_hash;

		let mut sink = self.manual_seal_command_sink.clone();
		let future = async move {
			let (sender, receiver) = oneshot::channel();
			let command = EngineCommand::SealNewBlock {
				create_empty,
				finalize: true,
				parent_hash: Some(hash),
				sender: Some(sender),
			};
			sink.send(command).await;
			receiver.await
		};

		log::trace!("waiting for SealNewBlock command to resolve...");
		futures::executor::block_on(future)
			.expect("Failed to receive SealNewBlock response")
			.expect("we have two layers of results, apparently")
	}
}

impl CliConfiguration for PerfCmd {
	fn shared_params(&self) -> &SharedParams {
		&self.shared_params
	}

	// copied from BenchmarkCmd, might be useful
	/*
	fn chain_id(&self, _is_dev: bool) -> Result<String> {
		Ok(match self.shared_params.chain {
			Some(ref chain) => chain.clone(),
			None => "dev".into(),
		})
	}
	*/
}

impl PerfCmd {

	// taking a different approach and starting a full dev service
	pub fn run<RuntimeApi, Executor>(&self, config: Configuration, ) -> CliResult<()>
	where
		RuntimeApi:
			ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>> + Send + Sync + 'static,
		RuntimeApi::RuntimeApi:
			RuntimeApiCollection<StateBackend = sc_client_api::StateBackendFor<FullBackend, Block>>,
		Executor: NativeExecutionDispatch + 'static,
	{
		let alice_hex = "f24FF3a9CF04c71Dbc94D0b566f7A27B94566cac";
		let alice_bytes = hex::decode(alice_hex)
			.expect("alice_hex is valid hex; qed");
		let alice = H160::from_slice(&alice_bytes[..]);

		let alice_priv_hex = "5fb92d6e98884f76de468fa3f6278f8807c48bebc13595d45af5bdc4da702133";
		let alice_priv_bytes = hex::decode(alice_priv_hex)
			.expect("alice_priv_hex is valid hex; qed");
		let alice_priv = H256::from_slice(&alice_priv_bytes[..]);

		log::debug!("alice: {:?}", alice);

		let mut runner = PerfTestRunner::<RuntimeApi, Executor>::from_cmd(&config, &self)?;

		let mut alice_nonce: U256 = 0.into();


		// Fibonacci contract:
		/*
		pragma solidity>= 0.8.0;
		contract Fibonacci {
			function fib2(uint n) public returns(uint b) {
				if (n == 0) {
					return 0;
				}
				uint a = 1;
				b = 1;
				for (uint i = 2; i < n; i++) {
					uint c = a + b;
					a = b;
					b = c;
				}
				return b;
			}
		}
		*/

		// start by deploying a contract...
		let fibonacci_hex =
			"608060405234801561001057600080fd5b5061024b806100206000396000f3fe608060405234801561001057600080fd5b506004361061002b5760003560e01c80633a9bbfcd14610030575b600080fd5b61004a600480360381019061004591906100d3565b610060565b604051610057919061010b565b60405180910390f35b60008082141561007357600090506100b9565b600060019050600191506000600290505b838110156100b6576000838361009a9190610126565b90508392508093505080806100ae90610186565b915050610084565b50505b919050565b6000813590506100cd816101fe565b92915050565b6000602082840312156100e557600080fd5b60006100f3848285016100be565b91505092915050565b6101058161017c565b82525050565b600060208201905061012060008301846100fc565b92915050565b60006101318261017c565b915061013c8361017c565b9250827fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff03821115610171576101706101cf565b5b828201905092915050565b6000819050919050565b60006101918261017c565b91507fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff8214156101c4576101c36101cf565b5b600182019050919050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052601160045260246000fd5b6102078161017c565b811461021257600080fd5b5056fea264697066735822122046f6b28b441f2f3caa263c58109a043ec2c03d4823bf1d25b1937cc1c684efab64736f6c63430008040033";
		let fibonacci_bytecode = hex::decode(fibonacci_hex)
			.expect("fibonacci_hex is valid hex; qed");

		// do a create() call (which doesn't persist) to see what our expected contract address
		// will be. afterward we create a txn and produce a block so it will persist.
		// TODO: better way to calculate new contract address
		let create_info = runner.evm_create(
			alice,
			fibonacci_bytecode.clone(),
			0.into(),
			EXTRINSIC_GAS_LIMIT.into(),
			Some(MIN_GAS_PRICE.into()),
			Some(alice_nonce),
			false
		).expect("EVM create failed while estimating contract address");
		let fibonacci_address = create_info.value;
		log::debug!("Fibonacci fibonacci_address expected to be {:?}", fibonacci_address);

		log::trace!("Issuing EVM create txn...");
		let txn_hash = runner.eth_sign_and_send_transaction(
			&alice_priv,
			None,
			fibonacci_bytecode,
			0.into(),
			EXTRINSIC_GAS_LIMIT.into(),
			MIN_GAS_PRICE.into(),
			alice_nonce,
		).expect("EVM create failed while trying to deploy Fibonacci contract");

		log::trace!("Creating block...");
		runner.create_block(true);

		// TODO: get txn results

		alice_nonce = alice_nonce.saturating_add(1.into());
		let calldata_hex = "3a9bbfcd0000000000000000000000000000000000000000000000000000000000000400";
		let calldata = hex::decode(calldata_hex)
			.expect("calldata is valid hex; qed");

		let call_results = runner.evm_call(
			alice,
			fibonacci_address,
			calldata,
			0.into(),
			EXTRINSIC_GAS_LIMIT.into(),
			Some(MIN_GAS_PRICE.into()),
			Some(alice_nonce),
			false
		).expect("EVM call failed while trying to invoke Fibonacci contract");

		log::debug!("EVM call returned {:?}", call_results);

		Ok(())
	}
}

