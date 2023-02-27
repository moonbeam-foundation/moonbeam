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
		impl_runtime_apis! {
			$($custom)*

			impl sp_api::Core<Block> for Runtime {
				fn version() -> RuntimeVersion {
					VERSION
				}

				fn execute_block(block: Block) {
					Executive::execute_block(block)
				}

				fn initialize_block(header: &<Block as BlockT>::Header) {
					Executive::initialize_block(header)
				}
			}

			impl sp_api::Metadata<Block> for Runtime {
				fn metadata() -> OpaqueMetadata {
					OpaqueMetadata::new(Runtime::metadata().into())
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

			impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index> for Runtime {
				fn account_nonce(account: AccountId) -> Index {
					System::account_nonce(account)
				}
			}

			impl moonbeam_rpc_primitives_debug::DebugRuntimeApi<Block> for Runtime {
				fn trace_transaction(
					extrinsics: Vec<<Block as BlockT>::Extrinsic>,
					traced_transaction: &EthereumTransaction,
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

						// Tell the CallDispatcher we are tracing a specific Transaction.
						unhashed::put::<EthereumXcmTracingStatus>(
							ETHEREUM_XCM_TRACING_STORAGE_KEY,
							&EthereumXcmTracingStatus::Transaction(traced_transaction.hash()),
						);

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
						Err(sp_runtime::DispatchError::Other(
							"Failed to find Ethereum transaction among the extrinsics.",
						))
					}
					#[cfg(not(feature = "evm-tracing"))]
					Err(sp_runtime::DispatchError::Other(
						"Missing `evm-tracing` compile time feature flag.",
					))
				}

				fn trace_block(
					extrinsics: Vec<<Block as BlockT>::Extrinsic>,
					known_transactions: Vec<H256>,
				) -> Result<
					(),
					sp_runtime::DispatchError,
				> {
					#[cfg(feature = "evm-tracing")]
					{
						use moonbeam_evm_tracer::tracer::EvmTracer;
						use xcm_primitives::EthereumXcmTracingStatus;

						// Tell the CallDispatcher we are tracing a full Block.
						frame_support::storage::unhashed::put::<EthereumXcmTracingStatus>(
							xcm_primitives::ETHEREUM_XCM_TRACING_STORAGE_KEY,
							&EthereumXcmTracingStatus::Block,
						);

						let mut config = <Runtime as pallet_evm::Config>::config().clone();
						config.estimate = true;

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
					EVM::account_codes(address)
				}

				fn author() -> H160 {
					<pallet_evm::Pallet<Runtime>>::find_author()
				}

				fn storage_at(address: H160, index: U256) -> H256 {
					let mut tmp = [0u8; 32];
					index.to_big_endian(&mut tmp);
					EVM::account_storages(address, H256::from_slice(&tmp[..]))
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
					<Runtime as pallet_evm::Config>::Runner::call(
						from,
						to,
						data,
						value,
						gas_limit.low_u64(),
						max_fee_per_gas,
						max_priority_fee_per_gas,
						nonce,
						access_list.unwrap_or_default(),
						is_transactional,
						validate,
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
					#[allow(clippy::or_fun_call)] // suggestion not helpful here
					<Runtime as pallet_evm::Config>::Runner::create(
						from,
						data,
						value,
						gas_limit.low_u64(),
						max_fee_per_gas,
						max_priority_fee_per_gas,
						nonce,
						access_list.unwrap_or_default(),
						is_transactional,
						validate,
						config.as_ref().unwrap_or(<Runtime as pallet_evm::Config>::config()),
					).map_err(|err| err.error.into())
				}

				fn current_transaction_statuses() -> Option<Vec<TransactionStatus>> {
					Ethereum::current_transaction_statuses()
				}

				fn current_block() -> Option<pallet_ethereum::Block> {
					Ethereum::current_block()
				}

				fn current_receipts() -> Option<Vec<pallet_ethereum::Receipt>> {
					Ethereum::current_receipts()
				}

				fn current_all() -> (
					Option<pallet_ethereum::Block>,
					Option<Vec<pallet_ethereum::Receipt>>,
					Option<Vec<TransactionStatus>>,
				) {
					(
						Ethereum::current_block(),
						Ethereum::current_receipts(),
						Ethereum::current_transaction_statuses(),
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
			}

			impl nimbus_primitives::NimbusApi<Block> for Runtime {
				fn can_author(
					author: nimbus_primitives::NimbusId,
					slot: u32,
					parent_header: &<Block as BlockT>::Header
				) -> bool {
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
					if pallet_parachain_staking::Pallet::<Self>::round().should_update(block_number) {
						// get author account id
						use nimbus_primitives::AccountLookup;
						let author_account_id = if let Some(account) =
							pallet_author_mapping::Pallet::<Self>::lookup_account(&author) {
							account
						} else {
							// return false if author mapping not registered like in can_author impl
							return false
						};
						// predict eligibility post-selection by computing selection results now
						let (eligible, _) =
							pallet_author_slot_filter::compute_pseudo_random_subset::<Self>(
								pallet_parachain_staking::Pallet::<Self>::compute_top_candidates(),
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

			#[cfg(feature = "runtime-benchmarks")]
			impl frame_benchmarking::Benchmark<Block> for Runtime {

				fn benchmark_metadata(extra: bool) -> (
					Vec<frame_benchmarking::BenchmarkList>,
					Vec<frame_support::traits::StorageInfo>,
				) {
					use frame_benchmarking::{list_benchmark, Benchmarking, BenchmarkList};
					use moonbeam_xcm_benchmarks::generic::benchmarking as MoonbeamXcmBenchmarks;
					use frame_support::traits::StorageInfoTrait;
					use frame_system_benchmarking::Pallet as SystemBench;
					use pallet_crowdloan_rewards::Pallet as PalletCrowdloanRewardsBench;
					use pallet_parachain_staking::Pallet as ParachainStakingBench;
					use pallet_author_mapping::Pallet as PalletAuthorMappingBench;
					use pallet_author_slot_filter::Pallet as PalletAuthorSlotFilter;
					use pallet_moonbeam_orbiters::Pallet as PalletMoonbeamOrbiters;
					use pallet_author_inherent::Pallet as PalletAuthorInherent;
					use pallet_asset_manager::Pallet as PalletAssetManagerBench;
					use pallet_xcm_transactor::Pallet as XcmTransactorBench;
					use pallet_randomness::Pallet as RandomnessBench;
					use pallet_migrations::Pallet as MigrationsBench;
					use MoonbeamXcmBenchmarks::XcmGenericBenchmarks as MoonbeamXcmGenericBench;

					let mut list = Vec::<BenchmarkList>::new();

					list_benchmark!(list, extra, frame_system, SystemBench::<Runtime>);
					list_benchmark!(list, extra, parachain_staking, ParachainStakingBench::<Runtime>);
					list_benchmark!(list, extra, pallet_crowdloan_rewards, PalletCrowdloanRewardsBench::<Runtime>);
					list_benchmark!(list, extra, pallet_author_mapping, PalletAuthorMappingBench::<Runtime>);
					list_benchmark!(list, extra, pallet_author_slot_filter, PalletAuthorSlotFilter::<Runtime>);
					list_benchmark!(list, extra, pallet_moonbeam_orbiters, PalletMoonbeamOrbiters::<Runtime>);
					list_benchmark!(list, extra, pallet_author_inherent, PalletAuthorInherent::<Runtime>);
					list_benchmark!(list, extra, pallet_asset_manager, PalletAssetManagerBench::<Runtime>);
					list_benchmark!(list, extra, xcm_transactor, XcmTransactorBench::<Runtime>);
					list_benchmark!(list, extra, pallet_randomness, RandomnessBench::<Runtime>);
					list_benchmark!(
						list,
						extra,
						moonbeam_xcm_benchmarks_generic,
						MoonbeamXcmGenericBench::<Runtime>
					);
					list_benchmark!(list, extra, pallet_migrations, MigrationsBench::<Runtime>);

					let storage_info = AllPalletsWithSystem::storage_info();

					return (list, storage_info)
				}

				fn dispatch_benchmark(
					config: frame_benchmarking::BenchmarkConfig,
				) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
					use frame_benchmarking::{
						add_benchmark, BenchmarkBatch, Benchmarking, TrackedStorageKey,
					};

					use xcm::latest::prelude::*;
					use frame_benchmarking::BenchmarkError;

					use frame_system_benchmarking::Pallet as SystemBench;
					impl frame_system_benchmarking::Config for Runtime {}

					impl moonbeam_xcm_benchmarks::Config for Runtime {}
					impl moonbeam_xcm_benchmarks::generic::Config for Runtime {}

					use pallet_asset_manager::Config as PalletAssetManagerConfig;

					impl pallet_xcm_benchmarks::Config for Runtime {
						type XcmConfig = xcm_config::XcmExecutorConfig;
						type AccountIdConverter = xcm_config::LocationToAccountId;
						fn valid_destination() -> Result<MultiLocation, BenchmarkError> {
							Ok(MultiLocation::parent())
						}
						fn worst_case_holding() -> MultiAssets {
						// 100 fungibles
							const HOLDING_FUNGIBLES: u32 = 100;
							let fungibles_amount: u128 = 100;
							let assets = (0..HOLDING_FUNGIBLES).map(|i| {
								let location: MultiLocation = GeneralIndex(i as u128).into();
								MultiAsset {
									id: Concrete(location),
									fun: Fungible(fungibles_amount * i as u128),
								}
								.into()
							})
							.chain(
								core::iter::once(
									MultiAsset {
										id: Concrete(MultiLocation::parent()),
										fun: Fungible(u128::MAX)
									}
								)
							)
							.collect::<Vec<_>>();


							for (i, asset) in assets.iter().enumerate() {
								if let MultiAsset {
									id: Concrete(location),
									fun: Fungible(_)
								} = asset {
									<AssetManager as xcm_primitives::AssetTypeGetter<
										<Runtime as PalletAssetManagerConfig>::AssetId,
										<Runtime as PalletAssetManagerConfig>::ForeignAssetType>
									>::set_asset_type_asset_id(
										location.clone().into(),
										i as u128
									);
									// set 1-1
									<AssetManager as xcm_primitives::UnitsToWeightRatio<
										<Runtime as PalletAssetManagerConfig>::ForeignAssetType>
									>::set_units_per_second(
										location.clone().into(),
										1_000_000_000_000u128
									);
								}
							}
							assets.into()
						}
					}

					impl pallet_xcm_benchmarks::generic::Config for Runtime {
						type RuntimeCall = RuntimeCall;

						fn worst_case_response() -> (u64, Response) {
							(0u64, Response::Version(Default::default()))
						}

						fn transact_origin() -> Result<MultiLocation, BenchmarkError> {
							Ok(MultiLocation::parent())
						}

						fn subscribe_origin() -> Result<MultiLocation, BenchmarkError> {
							Ok(MultiLocation::parent())
						}

						fn claimable_asset() -> Result<(MultiLocation, MultiLocation, MultiAssets), BenchmarkError> {
							let origin = MultiLocation::parent();
							let assets: MultiAssets = (Concrete(MultiLocation::parent()), 1_000u128).into();
							let ticket = MultiLocation { parents: 0, interior: Here };
							Ok((origin, ticket, assets))
						}
					}

					use moonbeam_xcm_benchmarks::generic::benchmarking as MoonbeamXcmBenchmarks;

					use pallet_crowdloan_rewards::Pallet as PalletCrowdloanRewardsBench;
					use pallet_parachain_staking::Pallet as ParachainStakingBench;
					use pallet_author_mapping::Pallet as PalletAuthorMappingBench;
					use pallet_author_slot_filter::Pallet as PalletAuthorSlotFilter;
					use pallet_moonbeam_orbiters::Pallet as PalletMoonbeamOrbiters;
					use pallet_author_inherent::Pallet as PalletAuthorInherent;
					use pallet_asset_manager::Pallet as PalletAssetManagerBench;
					use pallet_xcm_transactor::Pallet as XcmTransactorBench;
					use pallet_randomness::Pallet as RandomnessBench;
					use pallet_migrations::Pallet as MigrationsBench;
					use MoonbeamXcmBenchmarks::XcmGenericBenchmarks as MoonbeamXcmGenericBench;

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

					];

					let mut batches = Vec::<BenchmarkBatch>::new();
					let params = (&config, &whitelist);

					add_benchmark!(
						params,
						batches,
						parachain_staking,
						ParachainStakingBench::<Runtime>
					);
					add_benchmark!(
					params,
						batches,
						pallet_crowdloan_rewards,
						PalletCrowdloanRewardsBench::<Runtime>
					);
					add_benchmark!(
						params,
						batches,
						pallet_author_mapping,
						PalletAuthorMappingBench::<Runtime>
					);
					add_benchmark!(params, batches, frame_system, SystemBench::<Runtime>);
					add_benchmark!(
						params,
						batches,
						pallet_author_slot_filter,
						PalletAuthorSlotFilter::<Runtime>
					);
					add_benchmark!(
						params,
						batches,
						pallet_moonbeam_orbiters,
						PalletMoonbeamOrbiters::<Runtime>
					);
					add_benchmark!(
						params,
						batches,
						pallet_author_inherent,
						PalletAuthorInherent::<Runtime>
					);
					add_benchmark!(
						params,
						batches,
						pallet_asset_manager,
						PalletAssetManagerBench::<Runtime>
					);
					add_benchmark!(
						params,
						batches,
						xcm_transactor,
						XcmTransactorBench::<Runtime>
					);

					add_benchmark!(
						params,
						batches,
						pallet_randomness,
						RandomnessBench::<Runtime>
					);

					add_benchmark!(
						params,
						batches,
						pallet_migrations,
						MigrationsBench::<Runtime>
					);

					add_benchmark!(
						params,
						batches,
						moonbeam_xcm_benchmarks_generic,
						MoonbeamXcmGenericBench::<Runtime>
					);

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
