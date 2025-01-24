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

#[macro_export]
macro_rules! impl_runtime_apis_plus_common {
	{$($custom:tt)*} => {

		#[cfg(feature = "evm-tracing")]
		// Helper function to replay the "on_idle" hook for all pallets, we need this for
		// evm-tracing because some ethereum-xcm transactions might be executed at on_idle.
		//
		// We need to make sure that we replay on_idle exactly the same way as the
		// original block execution, but unfortunatly frame executive diosn't provide a function
		// to replay only on_idle, so we need to copy here some code inside frame executive.
		fn replay_on_idle() {
			use frame_system::pallet_prelude::BlockNumberFor;
			use frame_support::traits::OnIdle;

			let weight = <frame_system::Pallet<Runtime>>::block_weight();
			let max_weight = <
					<Runtime as frame_system::Config>::BlockWeights as
					frame_support::traits::Get<_>
				>::get().max_block;
			let remaining_weight = max_weight.saturating_sub(weight.total());
			if remaining_weight.all_gt(Weight::zero()) {
				let _ = <AllPalletsWithSystem as OnIdle<BlockNumberFor<Runtime>>>::on_idle(
					<frame_system::Pallet<Runtime>>::block_number(),
					remaining_weight,
				);
			}
		}

		impl_runtime_apis! {
			$($custom)*

			impl sp_api::Core<Block> for Runtime {
				fn version() -> RuntimeVersion {
					VERSION
				}

				fn execute_block(block: Block) {
					Executive::execute_block(block)
				}

				fn initialize_block(header: &<Block as BlockT>::Header) -> sp_runtime::ExtrinsicInclusionMode {
					Executive::initialize_block(header)
				}
			}

			impl sp_api::Metadata<Block> for Runtime {
				fn metadata() -> OpaqueMetadata {
					OpaqueMetadata::new(Runtime::metadata().into())
				}

				fn metadata_at_version(version: u32) -> Option<OpaqueMetadata> {
					Runtime::metadata_at_version(version)
				}

				fn metadata_versions() -> Vec<u32> {
					Runtime::metadata_versions()
				}
			}

			impl sp_block_builder::BlockBuilder<Block> for Runtime {
				fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
					Executive::apply_extrinsic(extrinsic)
				}

				fn finalize_block() -> <Block as BlockT>::Header {
					Executive::finalize_block()
				}

				fn inherent_extrinsics(
					data: sp_inherents::InherentData,
				) -> Vec<<Block as BlockT>::Extrinsic> {
					data.create_extrinsics()
				}

				fn check_inherents(
					block: Block,
					data: sp_inherents::InherentData,
				) -> sp_inherents::CheckInherentsResult {
					data.check_extrinsics(&block)
				}
			}

			impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
				fn offchain_worker(header: &<Block as BlockT>::Header) {
					Executive::offchain_worker(header)
				}
			}

			impl sp_session::SessionKeys<Block> for Runtime {
				fn decode_session_keys(
					encoded: Vec<u8>,
				) -> Option<Vec<(Vec<u8>, sp_core::crypto::KeyTypeId)>> {
					opaque::SessionKeys::decode_into_raw_public_keys(&encoded)
				}

				fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
					opaque::SessionKeys::generate(seed)
				}
			}

			#[cfg(not(feature = "disable-genesis-builder"))]
			impl sp_genesis_builder::GenesisBuilder<Block> for Runtime {
				fn build_state(config: Vec<u8>) -> sp_genesis_builder::Result {
					frame_support::genesis_builder_helper::build_state::<RuntimeGenesisConfig>(config)
				}

				fn get_preset(id: &Option<sp_genesis_builder::PresetId>) -> Option<Vec<u8>> {
					frame_support::genesis_builder_helper::get_preset::<RuntimeGenesisConfig>(id, genesis_config_preset::get_preset)
				}

				fn preset_names() -> Vec<sp_genesis_builder::PresetId> {
					genesis_config_preset::preset_names()
				}
			}

			impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index> for Runtime {
				fn account_nonce(account: AccountId) -> Index {
					System::account_nonce(account)
				}
			}

			impl moonbeam_rpc_primitives_debug::DebugRuntimeApi<Block> for Runtime {
				fn trace_transaction(
					extrinsics: Vec<<Block as BlockT>::Extrinsic>,
					traced_transaction: &EthereumTransaction,
					header: &<Block as BlockT>::Header,
				) -> Result<
					(),
					sp_runtime::DispatchError,
				> {
					#[cfg(feature = "evm-tracing")]
					{
						use moonbeam_evm_tracer::tracer::EvmTracer;
						use xcm_primitives::{
							ETHEREUM_XCM_TRACING_STORAGE_KEY,
							EthereumXcmTracingStatus
						};
						use frame_support::storage::unhashed;
						use frame_system::pallet_prelude::BlockNumberFor;

						// Tell the CallDispatcher we are tracing a specific Transaction.
						unhashed::put::<EthereumXcmTracingStatus>(
							ETHEREUM_XCM_TRACING_STORAGE_KEY,
							&EthereumXcmTracingStatus::Transaction(traced_transaction.hash()),
						);

						// Initialize block: calls the "on_initialize" hook on every pallet
						// in AllPalletsWithSystem.
						// After pallet message queue was introduced, this must be done only after
						// enabling XCM tracing by setting ETHEREUM_XCM_TRACING_STORAGE_KEY
						// in the storage
						Executive::initialize_block(header);

						// Apply the a subset of extrinsics: all the substrate-specific or ethereum
						// transactions that preceded the requested transaction.
						for ext in extrinsics.into_iter() {
							let _ = match &ext.0.function {
								RuntimeCall::Ethereum(transact { transaction }) => {
									if transaction == traced_transaction {
										EvmTracer::new().trace(|| Executive::apply_extrinsic(ext));
										return Ok(());
									} else {
										Executive::apply_extrinsic(ext)
									}
								}
								_ => Executive::apply_extrinsic(ext),
							};
							if let Some(EthereumXcmTracingStatus::TransactionExited) = unhashed::get(
								ETHEREUM_XCM_TRACING_STORAGE_KEY
							) {
								return Ok(());
							}
						}

						if let Some(EthereumXcmTracingStatus::Transaction(_)) = unhashed::get(
							ETHEREUM_XCM_TRACING_STORAGE_KEY
						) {
							// If the transaction was not found, it might be
							// an eth-xcm transaction that was executed at on_idle
							replay_on_idle();
						}

						if let Some(EthereumXcmTracingStatus::TransactionExited) = unhashed::get(
							ETHEREUM_XCM_TRACING_STORAGE_KEY
						) {
							// The transaction was found
							Ok(())
						} else {
							// The transaction was not-found
							Err(sp_runtime::DispatchError::Other(
								"Failed to find Ethereum transaction among the extrinsics.",
							))
						}
					}
					#[cfg(not(feature = "evm-tracing"))]
					Err(sp_runtime::DispatchError::Other(
						"Missing `evm-tracing` compile time feature flag.",
					))
				}

				fn trace_block(
					extrinsics: Vec<<Block as BlockT>::Extrinsic>,
					known_transactions: Vec<H256>,
					header: &<Block as BlockT>::Header,
				) -> Result<
					(),
					sp_runtime::DispatchError,
				> {
					#[cfg(feature = "evm-tracing")]
					{
						use moonbeam_evm_tracer::tracer::EvmTracer;
						use frame_system::pallet_prelude::BlockNumberFor;
						use xcm_primitives::EthereumXcmTracingStatus;

						// Tell the CallDispatcher we are tracing a full Block.
						frame_support::storage::unhashed::put::<EthereumXcmTracingStatus>(
							xcm_primitives::ETHEREUM_XCM_TRACING_STORAGE_KEY,
							&EthereumXcmTracingStatus::Block,
						);

						let mut config = <Runtime as pallet_evm::Config>::config().clone();
						config.estimate = true;

						// Initialize block: calls the "on_initialize" hook on every pallet
						// in AllPalletsWithSystem.
						// After pallet message queue was introduced, this must be done only after
						// enabling XCM tracing by setting ETHEREUM_XCM_TRACING_STORAGE_KEY
						// in the storage
						Executive::initialize_block(header);

						// Apply all extrinsics. Ethereum extrinsics are traced.
						for ext in extrinsics.into_iter() {
							match &ext.0.function {
								RuntimeCall::Ethereum(transact { transaction }) => {
									if known_transactions.contains(&transaction.hash()) {
										// Each known extrinsic is a new call stack.
										EvmTracer::emit_new();
										EvmTracer::new().trace(|| Executive::apply_extrinsic(ext));
									} else {
										let _ = Executive::apply_extrinsic(ext);
									}
								}
								_ => {
									let _ = Executive::apply_extrinsic(ext);
								}
							};
						}

						// Replay on_idle
						// Some XCM messages with eth-xcm transaction might be executed at on_idle
						replay_on_idle();

						Ok(())
					}
					#[cfg(not(feature = "evm-tracing"))]
					Err(sp_runtime::DispatchError::Other(
						"Missing `evm-tracing` compile time feature flag.",
					))
				}

				fn trace_call(
					header: &<Block as BlockT>::Header,
					from: H160,
					to: H160,
					data: Vec<u8>,
					value: U256,
					gas_limit: U256,
					max_fee_per_gas: Option<U256>,
					max_priority_fee_per_gas: Option<U256>,
					nonce: Option<U256>,
					access_list: Option<Vec<(H160, Vec<H256>)>>,
				) -> Result<(), sp_runtime::DispatchError> {
					#[cfg(feature = "evm-tracing")]
					{
						use moonbeam_evm_tracer::tracer::EvmTracer;

						// Initialize block: calls the "on_initialize" hook on every pallet
						// in AllPalletsWithSystem.
						Executive::initialize_block(header);

						EvmTracer::new().trace(|| {
							let is_transactional = false;
							let validate = true;
							let without_base_extrinsic_weight = true;


							// Estimated encoded transaction size must be based on the heaviest transaction
							// type (EIP1559Transaction) to be compatible with all transaction types.
							let mut estimated_transaction_len = data.len() +
							// pallet ethereum index: 1
							// transact call index: 1
							// Transaction enum variant: 1
							// chain_id 8 bytes
							// nonce: 32
							// max_priority_fee_per_gas: 32
							// max_fee_per_gas: 32
							// gas_limit: 32
							// action: 21 (enum varianrt + call address)
							// value: 32
							// access_list: 1 (empty vec size)
							// 65 bytes signature
							258;

							if access_list.is_some() {
								estimated_transaction_len += access_list.encoded_size();
							}

							let gas_limit = gas_limit.min(u64::MAX.into()).low_u64();

							let (weight_limit, proof_size_base_cost) =
								match <Runtime as pallet_evm::Config>::GasWeightMapping::gas_to_weight(
									gas_limit,
									without_base_extrinsic_weight
								) {
									weight_limit if weight_limit.proof_size() > 0 => {
										(Some(weight_limit), Some(estimated_transaction_len as u64))
									}
									_ => (None, None),
								};

							let _ = <Runtime as pallet_evm::Config>::Runner::call(
								from,
								to,
								data,
								value,
								gas_limit,
								max_fee_per_gas,
								max_priority_fee_per_gas,
								nonce,
								access_list.unwrap_or_default(),
								is_transactional,
								validate,
								weight_limit,
								proof_size_base_cost,
								<Runtime as pallet_evm::Config>::config(),
							);
						});
						Ok(())
					}
					#[cfg(not(feature = "evm-tracing"))]
					Err(sp_runtime::DispatchError::Other(
						"Missing `evm-tracing` compile time feature flag.",
					))
				}
			}

			impl moonbeam_rpc_primitives_txpool::TxPoolRuntimeApi<Block> for Runtime {
				fn extrinsic_filter(
					xts_ready: Vec<<Block as BlockT>::Extrinsic>,
					xts_future: Vec<<Block as BlockT>::Extrinsic>,
				) -> TxPoolResponse {
					TxPoolResponse {
						ready: xts_ready
							.into_iter()
							.filter_map(|xt| match xt.0.function {
								RuntimeCall::Ethereum(transact { transaction }) => Some(transaction),
								_ => None,
							})
							.collect(),
						future: xts_future
							.into_iter()
							.filter_map(|xt| match xt.0.function {
								RuntimeCall::Ethereum(transact { transaction }) => Some(transaction),
								_ => None,
							})
							.collect(),
					}
				}
			}

			impl fp_rpc::EthereumRuntimeRPCApi<Block> for Runtime {
				fn chain_id() -> u64 {
					<Runtime as pallet_evm::Config>::ChainId::get()
				}

				fn account_basic(address: H160) -> EVMAccount {
					let (account, _) = EVM::account_basic(&address);
					account
				}

				fn gas_price() -> U256 {
					let (gas_price, _) = <Runtime as pallet_evm::Config>::FeeCalculator::min_gas_price();
					gas_price
				}

				fn account_code_at(address: H160) -> Vec<u8> {
					pallet_evm::AccountCodes::<Runtime>::get(address)
				}

				fn author() -> H160 {
					<pallet_evm::Pallet<Runtime>>::find_author()
				}

				fn storage_at(address: H160, index: U256) -> H256 {
					let mut tmp = [0u8; 32];
					index.to_big_endian(&mut tmp);
					pallet_evm::AccountStorages::<Runtime>::get(address, H256::from_slice(&tmp[..]))
				}

				fn call(
					from: H160,
					to: H160,
					data: Vec<u8>,
					value: U256,
					gas_limit: U256,
					max_fee_per_gas: Option<U256>,
					max_priority_fee_per_gas: Option<U256>,
					nonce: Option<U256>,
					estimate: bool,
					access_list: Option<Vec<(H160, Vec<H256>)>>,
				) -> Result<pallet_evm::CallInfo, sp_runtime::DispatchError> {
					let config = if estimate {
						let mut config = <Runtime as pallet_evm::Config>::config().clone();
						config.estimate = true;
						Some(config)
					} else {
						None
					};
					let is_transactional = false;
					let validate = true;

					// Estimated encoded transaction size must be based on the heaviest transaction
					// type (EIP1559Transaction) to be compatible with all transaction types.
					let mut estimated_transaction_len = data.len() +
						// pallet ethereum index: 1
						// transact call index: 1
						// Transaction enum variant: 1
						// chain_id 8 bytes
						// nonce: 32
						// max_priority_fee_per_gas: 32
						// max_fee_per_gas: 32
						// gas_limit: 32
						// action: 21 (enum varianrt + call address)
						// value: 32
						// access_list: 1 (empty vec size)
						// 65 bytes signature
						258;

					if access_list.is_some() {
						estimated_transaction_len += access_list.encoded_size();
					}

					let gas_limit = gas_limit.min(u64::MAX.into()).low_u64();
					let without_base_extrinsic_weight = true;

					let (weight_limit, proof_size_base_cost) =
						match <Runtime as pallet_evm::Config>::GasWeightMapping::gas_to_weight(
							gas_limit,
							without_base_extrinsic_weight
						) {
							weight_limit if weight_limit.proof_size() > 0 => {
								(Some(weight_limit), Some(estimated_transaction_len as u64))
							}
							_ => (None, None),
						};

					<Runtime as pallet_evm::Config>::Runner::call(
						from,
						to,
						data,
						value,
						gas_limit,
						max_fee_per_gas,
						max_priority_fee_per_gas,
						nonce,
						access_list.unwrap_or_default(),
						is_transactional,
						validate,
						weight_limit,
						proof_size_base_cost,
						config.as_ref().unwrap_or(<Runtime as pallet_evm::Config>::config()),
					).map_err(|err| err.error.into())
				}

				fn create(
					from: H160,
					data: Vec<u8>,
					value: U256,
					gas_limit: U256,
					max_fee_per_gas: Option<U256>,
					max_priority_fee_per_gas: Option<U256>,
					nonce: Option<U256>,
					estimate: bool,
					access_list: Option<Vec<(H160, Vec<H256>)>>,
				) -> Result<pallet_evm::CreateInfo, sp_runtime::DispatchError> {
					let config = if estimate {
						let mut config = <Runtime as pallet_evm::Config>::config().clone();
						config.estimate = true;
						Some(config)
					} else {
						None
					};
					let is_transactional = false;
					let validate = true;

					let mut estimated_transaction_len = data.len() +
						// from: 20
						// value: 32
						// gas_limit: 32
						// nonce: 32
						// 1 byte transaction action variant
						// chain id 8 bytes
						// 65 bytes signature
						190;

					if max_fee_per_gas.is_some() {
						estimated_transaction_len += 32;
					}
					if max_priority_fee_per_gas.is_some() {
						estimated_transaction_len += 32;
					}
					if access_list.is_some() {
						estimated_transaction_len += access_list.encoded_size();
					}

					let gas_limit = if gas_limit > U256::from(u64::MAX) {
						u64::MAX
					} else {
						gas_limit.low_u64()
					};
					let without_base_extrinsic_weight = true;

					let (weight_limit, proof_size_base_cost) =
						match <Runtime as pallet_evm::Config>::GasWeightMapping::gas_to_weight(
							gas_limit,
							without_base_extrinsic_weight
						) {
							weight_limit if weight_limit.proof_size() > 0 => {
								(Some(weight_limit), Some(estimated_transaction_len as u64))
							}
							_ => (None, None),
						};

					#[allow(clippy::or_fun_call)] // suggestion not helpful here
					<Runtime as pallet_evm::Config>::Runner::create(
						from,
						data,
						value,
						gas_limit,
						max_fee_per_gas,
						max_priority_fee_per_gas,
						nonce,
						access_list.unwrap_or_default(),
						is_transactional,
						validate,
						weight_limit,
						proof_size_base_cost,
						config.as_ref().unwrap_or(<Runtime as pallet_evm::Config>::config()),
					).map_err(|err| err.error.into())
				}

				fn current_transaction_statuses() -> Option<Vec<TransactionStatus>> {
					pallet_ethereum::CurrentTransactionStatuses::<Runtime>::get()
				}

				fn current_block() -> Option<pallet_ethereum::Block> {
					pallet_ethereum::CurrentBlock::<Runtime>::get()
				}

				fn current_receipts() -> Option<Vec<pallet_ethereum::Receipt>> {
					pallet_ethereum::CurrentReceipts::<Runtime>::get()
				}

				fn current_all() -> (
					Option<pallet_ethereum::Block>,
					Option<Vec<pallet_ethereum::Receipt>>,
					Option<Vec<TransactionStatus>>,
				) {
					(
						pallet_ethereum::CurrentBlock::<Runtime>::get(),
						pallet_ethereum::CurrentReceipts::<Runtime>::get(),
						pallet_ethereum::CurrentTransactionStatuses::<Runtime>::get(),
					)
				}

				fn extrinsic_filter(
					xts: Vec<<Block as BlockT>::Extrinsic>,
				) -> Vec<EthereumTransaction> {
					xts.into_iter().filter_map(|xt| match xt.0.function {
						RuntimeCall::Ethereum(transact { transaction }) => Some(transaction),
						_ => None
					}).collect::<Vec<EthereumTransaction>>()
				}

				fn elasticity() -> Option<Permill> {
					None
				}

				fn gas_limit_multiplier_support() {}

				fn pending_block(
					xts: Vec<<Block as sp_runtime::traits::Block>::Extrinsic>
				) -> (
					Option<pallet_ethereum::Block>, Option<sp_std::prelude::Vec<TransactionStatus>>
				) {
					for ext in xts.into_iter() {
						let _ = Executive::apply_extrinsic(ext);
					}

					Ethereum::on_finalize(System::block_number() + 1);

					(
						pallet_ethereum::CurrentBlock::<Runtime>::get(),
						pallet_ethereum::CurrentTransactionStatuses::<Runtime>::get()
					)
				 }

				fn initialize_pending_block(header: &<Block as BlockT>::Header) {
					pallet_randomness::vrf::using_fake_vrf(|| {
						let _ = Executive::initialize_block(header);
					})
				}
			}

			impl fp_rpc::ConvertTransactionRuntimeApi<Block> for Runtime {
				fn convert_transaction(
					transaction: pallet_ethereum::Transaction
				) -> <Block as BlockT>::Extrinsic {
					UncheckedExtrinsic::new_unsigned(
						pallet_ethereum::Call::<Runtime>::transact { transaction }.into(),
					)
				}
			}

			impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance>
			for Runtime {
				fn query_info(
					uxt: <Block as BlockT>::Extrinsic,
					len: u32,
				) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
					TransactionPayment::query_info(uxt, len)
				}

				fn query_fee_details(
					uxt: <Block as BlockT>::Extrinsic,
					len: u32,
				) -> pallet_transaction_payment::FeeDetails<Balance> {
					TransactionPayment::query_fee_details(uxt, len)
				}

				fn query_weight_to_fee(weight: Weight) -> Balance {
					TransactionPayment::weight_to_fee(weight)
				}

				fn query_length_to_fee(length: u32) -> Balance {
					TransactionPayment::length_to_fee(length)
				}
			}

			impl nimbus_primitives::NimbusApi<Block> for Runtime {
				fn can_author(
					author: nimbus_primitives::NimbusId,
					slot: u32,
					parent_header: &<Block as BlockT>::Header
				) -> bool {
					use pallet_parachain_staking::Config as PalletParachainStakingConfig;

					let block_number = parent_header.number + 1;

					// The Moonbeam runtimes use an entropy source that needs to do some accounting
					// work during block initialization. Therefore we initialize it here to match
					// the state it will be in when the next block is being executed.
					use frame_support::traits::OnInitialize;
					System::initialize(
						&block_number,
						&parent_header.hash(),
						&parent_header.digest,
					);

					// Because the staking solution calculates the next staking set at the beginning
					// of the first block in the new round, the only way to accurately predict the
					// authors is to compute the selection during prediction.
					if pallet_parachain_staking::Pallet::<Self>::round()
						.should_update(block_number) {
						// get author account id
						use nimbus_primitives::AccountLookup;
						let author_account_id = if let Some(account) =
							pallet_author_mapping::Pallet::<Self>::lookup_account(&author) {
							account
						} else {
							// return false if author mapping not registered like in can_author impl
							return false
						};
						let candidates = pallet_parachain_staking::Pallet::<Self>::compute_top_candidates();
						if candidates.is_empty() {
							// If there are zero selected candidates, we use the same eligibility
							// as the previous round
							return AuthorInherent::can_author(&author, &slot);
						}

						// predict eligibility post-selection by computing selection results now
						let (eligible, _) =
							pallet_author_slot_filter::compute_pseudo_random_subset::<Self>(
								candidates,
								&slot
							);
						eligible.contains(&author_account_id)
					} else {
						AuthorInherent::can_author(&author, &slot)
					}
				}
			}

			impl cumulus_primitives_core::CollectCollationInfo<Block> for Runtime {
				fn collect_collation_info(
					header: &<Block as BlockT>::Header
				) -> cumulus_primitives_core::CollationInfo {
					ParachainSystem::collect_collation_info(header)
				}
			}

			impl session_keys_primitives::VrfApi<Block> for Runtime {
				fn get_last_vrf_output() -> Option<<Block as BlockT>::Hash> {
					// TODO: remove in future runtime upgrade along with storage item
					if pallet_randomness::Pallet::<Self>::not_first_block().is_none() {
						return None;
					}
					pallet_randomness::Pallet::<Self>::local_vrf_output()
				}
				fn vrf_key_lookup(
					nimbus_id: nimbus_primitives::NimbusId
				) -> Option<session_keys_primitives::VrfId> {
					use session_keys_primitives::KeysLookup;
					AuthorMapping::lookup_keys(&nimbus_id)
				}
			}

			impl xcm_runtime_apis::fees::XcmPaymentApi<Block> for Runtime {
				fn query_acceptable_payment_assets(
					xcm_version: xcm::Version
				) -> Result<Vec<VersionedAssetId>, XcmPaymentApiError> {
					XcmWeightTrader::query_acceptable_payment_assets(xcm_version)
				}

				fn query_weight_to_asset_fee(
					weight: Weight, asset: VersionedAssetId
				) -> Result<u128, XcmPaymentApiError> {
					XcmWeightTrader::query_weight_to_asset_fee(weight, asset)
				}

				fn query_xcm_weight(message: VersionedXcm<()>) -> Result<Weight, XcmPaymentApiError> {
					PolkadotXcm::query_xcm_weight(message)
				}

				fn query_delivery_fees(
					destination: VersionedLocation, message: VersionedXcm<()>
				) -> Result<VersionedAssets, XcmPaymentApiError> {
					PolkadotXcm::query_delivery_fees(destination, message)
				}
			}

			impl xcm_runtime_apis::dry_run::DryRunApi<Block, RuntimeCall, RuntimeEvent, OriginCaller>
				for Runtime {
					fn dry_run_call(
						origin: OriginCaller,
						call: RuntimeCall
					) -> Result<CallDryRunEffects<RuntimeEvent>, XcmDryRunApiError> {
						PolkadotXcm::dry_run_call::<
							Runtime,
							xcm_config::XcmRouter,
							OriginCaller,
							RuntimeCall>(origin, call)
					}

					fn dry_run_xcm(
						origin_location: VersionedLocation,
						xcm: VersionedXcm<RuntimeCall>
					) -> Result<XcmDryRunEffects<RuntimeEvent>, XcmDryRunApiError> {
						PolkadotXcm::dry_run_xcm::<
							Runtime,
							xcm_config::XcmRouter,
							RuntimeCall,
							xcm_config::XcmExecutorConfig>(origin_location, xcm)
					}
				}

			impl xcm_runtime_apis::conversions::LocationToAccountApi<Block, AccountId> for Runtime {
				fn convert_location(location: VersionedLocation) -> Result<
					AccountId,
					xcm_runtime_apis::conversions::Error
				> {
					xcm_runtime_apis::conversions::LocationToAccountHelper::<
						AccountId,
						xcm_config::LocationToAccountId,
					>::convert_location(location)
				}
			}

			#[cfg(feature = "runtime-benchmarks")]
			impl frame_benchmarking::Benchmark<Block> for Runtime {

				fn benchmark_metadata(extra: bool) -> (
					Vec<frame_benchmarking::BenchmarkList>,
					Vec<frame_support::traits::StorageInfo>,
				) {
					use frame_benchmarking::{list_benchmark, Benchmarking, BenchmarkList};
					use moonbeam_xcm_benchmarks::generic::benchmarking as MoonbeamXcmBenchmarks;
					use frame_support::traits::StorageInfoTrait;
					use MoonbeamXcmBenchmarks::XcmGenericBenchmarks as MoonbeamXcmGenericBench;

					use pallet_xcm::benchmarking::Pallet as PalletXcmExtrinsicsBenchmark;

					let mut list = Vec::<BenchmarkList>::new();
					list_benchmarks!(list, extra);

					let storage_info = AllPalletsWithSystem::storage_info();

					return (list, storage_info)
				}

				fn dispatch_benchmark(
					config: frame_benchmarking::BenchmarkConfig,
				) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
					use frame_benchmarking::{add_benchmark, BenchmarkBatch, Benchmarking};
					use frame_support::traits::TrackedStorageKey;
					use cumulus_primitives_core::ParaId;

					use xcm::latest::prelude::{
						GeneralIndex, Junction, Junctions, Location, Response, NetworkId, AssetId,
						Assets as XcmAssets, Fungible, Asset, ParentThen, Parachain, Parent
					};
					use xcm_config::SelfReserve;
					use frame_benchmarking::BenchmarkError;

					use frame_system_benchmarking::Pallet as SystemBench;
					impl frame_system_benchmarking::Config for Runtime {}

					impl moonbeam_xcm_benchmarks::Config for Runtime {}
					impl moonbeam_xcm_benchmarks::generic::Config for Runtime {}

					use pallet_asset_manager::Config as PalletAssetManagerConfig;

					use pallet_xcm::benchmarking::Pallet as PalletXcmExtrinsicsBenchmark;
					parameter_types! {
						pub const RandomParaId: ParaId = ParaId::new(43211234);
					}

					pub struct TestDeliveryHelper;
					impl xcm_builder::EnsureDelivery for TestDeliveryHelper {
						fn ensure_successful_delivery(
							origin_ref: &Location,
							_dest: &Location,
							_fee_reason: xcm_executor::traits::FeeReason,
						) -> (Option<xcm_executor::FeesMode>, Option<XcmAssets>) {
							use xcm_executor::traits::ConvertLocation;
							let account = xcm_config::LocationToH160::convert_location(origin_ref)
								.expect("Invalid location");
							// Give the existential deposit at least
							let balance = ExistentialDeposit::get();
							let _ = <Balances as frame_support::traits::Currency<_>>::
								make_free_balance_be(&account.into(), balance);

							(None, None)
						}
					}

					impl pallet_xcm::benchmarking::Config for Runtime {
				        type DeliveryHelper = TestDeliveryHelper;

						fn get_asset() -> Asset {
							Asset {
								id: AssetId(SelfReserve::get()),
								fun: Fungible(ExistentialDeposit::get()),
							}
						}

						fn reachable_dest() -> Option<Location> {
							Some(Parent.into())
						}

						fn teleportable_asset_and_dest() -> Option<(Asset, Location)> {
							None
						}

						fn reserve_transferable_asset_and_dest() -> Option<(Asset, Location)> {
							use xcm_config::SelfReserve;

							ParachainSystem::open_outbound_hrmp_channel_for_benchmarks_or_tests(
								RandomParaId::get().into()
							);

							Some((
								Asset {
									fun: Fungible(ExistentialDeposit::get()),
									id: AssetId(SelfReserve::get().into())
								},
								// Moonbeam can reserve transfer native token to
								// some random parachain.
								ParentThen(Parachain(RandomParaId::get().into()).into()).into(),
							))
						}

						fn set_up_complex_asset_transfer(
						) -> Option<(XcmAssets, u32, Location, Box<dyn FnOnce()>)> {
							use xcm_config::SelfReserve;

							let destination: xcm::v4::Location = Parent.into();

							let fee_amount: u128 = <Runtime as pallet_balances::Config>::ExistentialDeposit::get();
							let fee_asset: Asset = (SelfReserve::get(), fee_amount).into();

							// Give some multiple of transferred amount
							let balance = fee_amount * 1000;
							let who = frame_benchmarking::whitelisted_caller();
							let _ =
								<Balances as frame_support::traits::Currency<_>>::make_free_balance_be(&who, balance);

							// verify initial balance
							assert_eq!(Balances::free_balance(&who), balance);

							// set up local asset
							let asset_amount: u128 = 10u128;
							let initial_asset_amount: u128 = asset_amount * 10;

							let (asset_id, _, _) = pallet_assets::benchmarking::create_default_minted_asset::<
								Runtime,
								()
							>(true, initial_asset_amount);
							let transfer_asset: Asset = (SelfReserve::get(), asset_amount).into();

							let assets: XcmAssets = vec![fee_asset.clone(), transfer_asset].into();
							let fee_index: u32 = 0;

							let verify: Box<dyn FnOnce()> = Box::new(move || {
								// verify balance after transfer, decreased by
								// transferred amount (and delivery fees)
								assert!(Balances::free_balance(&who) <= balance - fee_amount);
							});

							Some((assets, fee_index, destination, verify))
						}
					}

					impl pallet_xcm_benchmarks::Config for Runtime {
						type XcmConfig = xcm_config::XcmExecutorConfig;
						type AccountIdConverter = xcm_config::LocationToAccountId;
						type DeliveryHelper = ();
						fn valid_destination() -> Result<Location, BenchmarkError> {
							Ok(Location::parent())
						}
						fn worst_case_holding(_depositable_count: u32) -> XcmAssets {
						// 100 fungibles
							const HOLDING_FUNGIBLES: u32 = 100;
							let fungibles_amount: u128 = 100;
							let assets = (0..HOLDING_FUNGIBLES).map(|i| {
								let location: Location = GeneralIndex(i as u128).into();
								Asset {
									id: AssetId(location),
									fun: Fungible(fungibles_amount * i as u128),
								}
								.into()
							})
							.chain(
								core::iter::once(
									Asset {
										id: AssetId(Location::parent()),
										fun: Fungible(u128::MAX)
									}
								)
							)
							.collect::<Vec<_>>();


							for (i, asset) in assets.iter().enumerate() {
								if let Asset {
									id: AssetId(location),
									fun: Fungible(_)
								} = asset {
									EvmForeignAssets::set_asset(
										location.clone(),
										i as u128
									);
									XcmWeightTrader::set_asset_price(
										location.clone(),
										1u128.pow(18)
									);
								}
							}
							assets.into()
						}
					}

					impl pallet_xcm_benchmarks::generic::Config for Runtime {
						type RuntimeCall = RuntimeCall;
						type TransactAsset = Balances;

						fn worst_case_response() -> (u64, Response) {
							(0u64, Response::Version(Default::default()))
						}

						fn worst_case_asset_exchange()
							-> Result<(XcmAssets, XcmAssets), BenchmarkError> {
							Err(BenchmarkError::Skip)
						}

						fn universal_alias() -> Result<(Location, Junction), BenchmarkError> {
							Err(BenchmarkError::Skip)
						}

						fn export_message_origin_and_destination()
							-> Result<(Location, NetworkId, Junctions), BenchmarkError> {
							Err(BenchmarkError::Skip)
						}

						fn transact_origin_and_runtime_call()
							-> Result<(Location, RuntimeCall), BenchmarkError> {
							Ok((Location::parent(), frame_system::Call::remark_with_event {
								remark: vec![]
							}.into()))
						}

						fn subscribe_origin() -> Result<Location, BenchmarkError> {
							Ok(Location::parent())
						}

						fn claimable_asset()
							-> Result<(Location, Location, XcmAssets), BenchmarkError> {
							let origin = Location::parent();
							let assets: XcmAssets = (AssetId(Location::parent()), 1_000u128)
								.into();
							let ticket = Location { parents: 0, interior: [].into() /* Here */ };
							Ok((origin, ticket, assets))
						}

						fn fee_asset() -> Result<Asset, BenchmarkError> {
							Err(BenchmarkError::Skip)
						}

						fn unlockable_asset()
							-> Result<(Location, Location, Asset), BenchmarkError> {
							Err(BenchmarkError::Skip)
						}

						fn alias_origin() -> Result<(Location, Location), BenchmarkError> {
							Err(BenchmarkError::Skip)
						}
					}

					let whitelist: Vec<TrackedStorageKey> = vec![
						// Block Number
						hex_literal::hex!(  "26aa394eea5630e07c48ae0c9558cef7"
											"02a5c1b19ab7a04f536c519aca4983ac")
							.to_vec().into(),
						// Total Issuance
						hex_literal::hex!(  "c2261276cc9d1f8598ea4b6a74b15c2f"
											"57c875e4cff74148e4628f264b974c80")
							.to_vec().into(),
						// Execution Phase
						hex_literal::hex!(  "26aa394eea5630e07c48ae0c9558cef7"
											"ff553b5a9862a516939d82b3d3d8661a")
							.to_vec().into(),
						// Event Count
						hex_literal::hex!(  "26aa394eea5630e07c48ae0c9558cef7"
											"0a98fdbe9ce6c55837576c60c7af3850")
							.to_vec().into(),
						// System Events
						hex_literal::hex!(  "26aa394eea5630e07c48ae0c9558cef7"
											"80d41e5e16056765bc8461851072c9d7")
							.to_vec().into(),
						// System BlockWeight
						hex_literal::hex!(  "26aa394eea5630e07c48ae0c9558cef7"
											"34abf5cb34d6244378cddbf18e849d96")
							.to_vec().into(),
						// ParachainStaking Round
						hex_literal::hex!(  "a686a3043d0adcf2fa655e57bc595a78"
											"13792e785168f725b60e2969c7fc2552")
							.to_vec().into(),
						// Treasury Account (py/trsry)
						hex_literal::hex!(  "26aa394eea5630e07c48ae0c9558cef7"
											"b99d880ec681799c0cf30e8886371da9"
											"7be2919ac397ba499ea5e57132180ec6"
											"6d6f646c70792f747273727900000000"
											"00000000"
						).to_vec().into(),
						// Treasury Account (pc/trsry)
						hex_literal::hex!(  "26aa394eea5630e07c48ae0c9558cef7"
											"b99d880ec681799c0cf30e8886371da9"
											"7be2919ac397ba499ea5e57132180ec6"
											"6d6f646c70632f747273727900000000"
											"00000000"
						).to_vec().into(),
						// ParachainInfo ParachainId
						hex_literal::hex!(  "0d715f2646c8f85767b5d2764bb27826"
											"04a74d81251e398fd8a0a4d55023bb3f")
							.to_vec().into(),
						// Parameters Parameters
						hex_literal::hex!(  "c63bdd4a39095ccf55623a6f2872bf8a" // Pallet: "Parameters"
											"c63bdd4a39095ccf55623a6f2872bf8a" // Storage Prefix: "Parameters"
											// MoonbaseRuntimeRuntimeParamsRuntimeParametersKey(FeesTreasuryProportion)
											"71d0aacb690b61280d0c97c6b6a666640000"
										)
							.to_vec().into(),

					];

					let mut batches = Vec::<BenchmarkBatch>::new();
					let params = (&config, &whitelist);

					add_benchmarks!(params, batches);

					if batches.is_empty() {
						return Err("Benchmark not found for this pallet.".into());
					}
					Ok(batches)
				}
			}

			#[cfg(feature = "try-runtime")]
			impl frame_try_runtime::TryRuntime<Block> for Runtime {
				fn on_runtime_upgrade(checks: frame_try_runtime::UpgradeCheckSelect) -> (Weight, Weight) {
					log::info!("try-runtime::on_runtime_upgrade()");
					// NOTE: intentional expect: we don't want to propagate the error backwards,
					// and want to have a backtrace here. If any of the pre/post migration checks
					// fail, we shall stop right here and right now.
					let weight = Executive::try_runtime_upgrade(checks)
						.expect("runtime upgrade logic *must* be infallible");
					(weight, RuntimeBlockWeights::get().max_block)
				}

				fn execute_block(
					block: Block,
					state_root_check: bool,
					signature_check: bool,
					select: frame_try_runtime::TryStateSelect
				) -> Weight {
					log::info!(
						"try-runtime: executing block {:?} / root checks: {:?} / try-state-select: {:?}",
						block.header.hash(),
						state_root_check,
						select,
					);
					// NOTE: intentional unwrap: we don't want to propagate the error backwards,
					// and want to have a backtrace here.
					Executive::try_execute_block(
						block,
						state_root_check,
						signature_check,
						select,
					).expect("execute-block failed")
				}
			}
		}
	};
}
