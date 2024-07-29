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

//! Moonbase Runtime Integration Tests

mod common;
use common::*;

use precompile_utils::{
	precompile_set::{is_precompile_or_fail, IsActivePrecompile},
	prelude::*,
	testing::*,
};

use fp_evm::{Context, IsPrecompileResult};
use frame_support::{
	assert_noop, assert_ok,
	dispatch::DispatchClass,
	traits::{
		fungible::Inspect, Currency as CurrencyT, EnsureOrigin, PalletInfo, StorageInfo,
		StorageInfoTrait,
	},
	weights::{constants::WEIGHT_REF_TIME_PER_SECOND, Weight},
	StorageHasher, Twox128,
};
use moonbase_runtime::{
	//asset_config::ForeignAssetInstance,
	xcm_config::SelfReserve,
	AccountId,
	AssetId,
	Balances,
	CrowdloanRewards,
	EvmForeignAssets,
	Executive,
	OpenTechCommitteeCollective,
	ParachainStaking,
	PolkadotXcm,
	Precompiles,
	Runtime,
	RuntimeBlockWeights,
	RuntimeCall,
	RuntimeEvent,
	System,
	TransactionPayment,
	TransactionPaymentAsGasPrice,
	TreasuryCouncilCollective,
	XTokens,
	XcmTransactor,
	FOREIGN_ASSET_PRECOMPILE_ADDRESS_PREFIX,
	WEEKS,
};
use polkadot_parachain::primitives::Sibling;
use precompile_utils::testing::MockHandle;
use sp_runtime::{
	traits::{Convert as XcmConvert, Dispatchable},
	BuildStorage,
};
use std::str::from_utf8;
use xcm_builder::{ParentIsPreset, SiblingParachainConvertsVia};
use xcm_executor::traits::ConvertLocation;

use moonbase_runtime::currency::{GIGAWEI, WEI};
use moonbeam_xcm_benchmarks::weights::XcmWeight;
use moonkit_xcm_primitives::AccountIdAssetIdConversion;
use nimbus_primitives::NimbusId;
use pallet_evm::PrecompileSet;
//use pallet_evm_precompileset_assets_erc20::{SELECTOR_LOG_APPROVAL, SELECTOR_LOG_TRANSFER};
use pallet_moonbeam_foreign_assets::AssetStatus;
use pallet_transaction_payment::Multiplier;
use pallet_xcm_transactor::{Currency, CurrencyPayment, HrmpOperation, TransactWeights};
use parity_scale_codec::Encode;
use sha3::{Digest, Keccak256};
use sp_core::{crypto::UncheckedFrom, ByteArray, Pair, H160, H256, U256};
use sp_runtime::{bounded_vec, DispatchError, ModuleError};
use std::cell::Cell;
use std::rc::Rc;
use xcm::latest::prelude::*;

type AuthorMappingPCall =
	pallet_evm_precompile_author_mapping::AuthorMappingPrecompileCall<Runtime>;
type BatchPCall = pallet_evm_precompile_batch::BatchPrecompileCall<Runtime>;
type CrowdloanRewardsPCall =
	pallet_evm_precompile_crowdloan_rewards::CrowdloanRewardsPrecompileCall<Runtime>;
type XcmUtilsPCall = pallet_evm_precompile_xcm_utils::XcmUtilsPrecompileCall<
	Runtime,
	moonbase_runtime::xcm_config::XcmExecutorConfig,
>;
type XtokensPCall = pallet_evm_precompile_xtokens::XtokensPrecompileCall<Runtime>;
/*type ForeignAssetsPCall = pallet_evm_precompileset_assets_erc20::Erc20AssetsPrecompileSetCall<
	Runtime,
	ForeignAssetInstance,
>;*/
type XcmTransactorV1PCall =
	pallet_evm_precompile_xcm_transactor::v1::XcmTransactorPrecompileV1Call<Runtime>;
type XcmTransactorV2PCall =
	pallet_evm_precompile_xcm_transactor::v2::XcmTransactorPrecompileV2Call<Runtime>;

// TODO: can we construct a const U256...?
const BASE_FEE_GENISIS: u128 = 10 * GIGAWEI;

#[test]
fn xcmp_queue_controller_origin_is_root() {
	// important for the XcmExecutionManager impl of PauseExecution which uses root origin
	// to suspend/resume XCM execution in xcmp_queue::on_idle
	assert_ok!(
		<moonbase_runtime::Runtime as cumulus_pallet_xcmp_queue::Config
		>::ControllerOrigin::ensure_origin(root_origin())
	);
}

#[test]
fn verify_pallet_prefixes() {
	fn is_pallet_prefix<P: 'static>(name: &str) {
		// Compares the unhashed pallet prefix in the `StorageInstance` implementation by every
		// storage item in the pallet P. This pallet prefix is used in conjunction with the
		// item name to get the unique storage key: hash(PalletPrefix) + hash(StorageName)
		// https://github.com/paritytech/substrate/blob/master/frame/support/procedural/src/pallet/
		// expand/storage.rs#L389-L401
		assert_eq!(
			<moonbase_runtime::Runtime as frame_system::Config>::PalletInfo::name::<P>(),
			Some(name)
		);
	}
	// TODO: use StorageInfoTrait from https://github.com/paritytech/substrate/pull/9246
	// This is now available with polkadot-v0.9.9 dependencies
	is_pallet_prefix::<moonbase_runtime::System>("System");
	is_pallet_prefix::<moonbase_runtime::Utility>("Utility");
	is_pallet_prefix::<moonbase_runtime::ParachainSystem>("ParachainSystem");
	is_pallet_prefix::<moonbase_runtime::TransactionPayment>("TransactionPayment");
	is_pallet_prefix::<moonbase_runtime::ParachainInfo>("ParachainInfo");
	is_pallet_prefix::<moonbase_runtime::EthereumChainId>("EthereumChainId");
	is_pallet_prefix::<moonbase_runtime::EVM>("EVM");
	is_pallet_prefix::<moonbase_runtime::Ethereum>("Ethereum");
	is_pallet_prefix::<moonbase_runtime::ParachainStaking>("ParachainStaking");
	is_pallet_prefix::<moonbase_runtime::Scheduler>("Scheduler");
	is_pallet_prefix::<moonbase_runtime::Treasury>("Treasury");
	is_pallet_prefix::<moonbase_runtime::OpenTechCommitteeCollective>(
		"OpenTechCommitteeCollective",
	);
	is_pallet_prefix::<moonbase_runtime::AuthorInherent>("AuthorInherent");
	is_pallet_prefix::<moonbase_runtime::AuthorFilter>("AuthorFilter");
	is_pallet_prefix::<moonbase_runtime::CrowdloanRewards>("CrowdloanRewards");
	is_pallet_prefix::<moonbase_runtime::AuthorMapping>("AuthorMapping");
	is_pallet_prefix::<moonbase_runtime::MaintenanceMode>("MaintenanceMode");
	is_pallet_prefix::<moonbase_runtime::Identity>("Identity");
	is_pallet_prefix::<moonbase_runtime::XcmpQueue>("XcmpQueue");
	is_pallet_prefix::<moonbase_runtime::CumulusXcm>("CumulusXcm");
	is_pallet_prefix::<moonbase_runtime::DmpQueue>("DmpQueue");
	is_pallet_prefix::<moonbase_runtime::PolkadotXcm>("PolkadotXcm");
	is_pallet_prefix::<moonbase_runtime::Assets>("Assets");
	is_pallet_prefix::<moonbase_runtime::XTokens>("XTokens");
	is_pallet_prefix::<moonbase_runtime::AssetManager>("AssetManager");
	is_pallet_prefix::<moonbase_runtime::Migrations>("Migrations");
	is_pallet_prefix::<moonbase_runtime::XcmTransactor>("XcmTransactor");
	is_pallet_prefix::<moonbase_runtime::ProxyGenesisCompanion>("ProxyGenesisCompanion");
	is_pallet_prefix::<moonbase_runtime::MoonbeamOrbiters>("MoonbeamOrbiters");
	is_pallet_prefix::<moonbase_runtime::EthereumXcm>("EthereumXcm");
	is_pallet_prefix::<moonbase_runtime::Randomness>("Randomness");
	is_pallet_prefix::<moonbase_runtime::TreasuryCouncilCollective>("TreasuryCouncilCollective");
	is_pallet_prefix::<moonbase_runtime::MoonbeamLazyMigrations>("MoonbeamLazyMigrations");
	is_pallet_prefix::<moonbase_runtime::RelayStorageRoots>("RelayStorageRoots");

	let prefix = |pallet_name, storage_name| {
		let mut res = [0u8; 32];
		res[0..16].copy_from_slice(&Twox128::hash(pallet_name));
		res[16..32].copy_from_slice(&Twox128::hash(storage_name));
		res.to_vec()
	};
	assert_eq!(
		<moonbase_runtime::Balances as StorageInfoTrait>::storage_info(),
		vec![
			StorageInfo {
				pallet_name: b"Balances".to_vec(),
				storage_name: b"TotalIssuance".to_vec(),
				prefix: prefix(b"Balances", b"TotalIssuance"),
				max_values: Some(1),
				max_size: Some(16),
			},
			StorageInfo {
				pallet_name: b"Balances".to_vec(),
				storage_name: b"InactiveIssuance".to_vec(),
				prefix: prefix(b"Balances", b"InactiveIssuance"),
				max_values: Some(1),
				max_size: Some(16),
			},
			StorageInfo {
				pallet_name: b"Balances".to_vec(),
				storage_name: b"Account".to_vec(),
				prefix: prefix(b"Balances", b"Account"),
				max_values: None,
				max_size: Some(100),
			},
			StorageInfo {
				pallet_name: b"Balances".to_vec(),
				storage_name: b"Locks".to_vec(),
				prefix: prefix(b"Balances", b"Locks"),
				max_values: None,
				max_size: Some(1287),
			},
			StorageInfo {
				pallet_name: b"Balances".to_vec(),
				storage_name: b"Reserves".to_vec(),
				prefix: prefix(b"Balances", b"Reserves"),
				max_values: None,
				max_size: Some(1037),
			},
			StorageInfo {
				pallet_name: b"Balances".to_vec(),
				storage_name: b"Holds".to_vec(),
				prefix: prefix(b"Balances", b"Holds"),
				max_values: None,
				max_size: Some(55),
			},
			StorageInfo {
				pallet_name: b"Balances".to_vec(),
				storage_name: b"Freezes".to_vec(),
				prefix: prefix(b"Balances", b"Freezes"),
				max_values: None,
				max_size: Some(37),
			},
		]
	);
	assert_eq!(
		<moonbase_runtime::Sudo as StorageInfoTrait>::storage_info(),
		vec![StorageInfo {
			pallet_name: b"Sudo".to_vec(),
			storage_name: b"Key".to_vec(),
			prefix: prefix(b"Sudo", b"Key"),
			max_values: Some(1),
			max_size: Some(20),
		}]
	);
	assert_eq!(
		<moonbase_runtime::Proxy as StorageInfoTrait>::storage_info(),
		vec![
			StorageInfo {
				pallet_name: b"Proxy".to_vec(),
				storage_name: b"Proxies".to_vec(),
				prefix: prefix(b"Proxy", b"Proxies"),
				max_values: None,
				max_size: Some(845),
			},
			StorageInfo {
				pallet_name: b"Proxy".to_vec(),
				storage_name: b"Announcements".to_vec(),
				prefix: prefix(b"Proxy", b"Announcements"),
				max_values: None,
				max_size: Some(1837),
			}
		]
	);
	assert_eq!(
		<moonbase_runtime::MaintenanceMode as StorageInfoTrait>::storage_info(),
		vec![StorageInfo {
			pallet_name: b"MaintenanceMode".to_vec(),
			storage_name: b"MaintenanceMode".to_vec(),
			prefix: prefix(b"MaintenanceMode", b"MaintenanceMode"),
			max_values: Some(1),
			max_size: None,
		},]
	);

	assert_eq!(
		<moonbase_runtime::RelayStorageRoots as StorageInfoTrait>::storage_info(),
		vec![
			StorageInfo {
				pallet_name: b"RelayStorageRoots".to_vec(),
				storage_name: b"RelayStorageRoot".to_vec(),
				prefix: prefix(b"RelayStorageRoots", b"RelayStorageRoot"),
				max_values: None,
				max_size: Some(44),
			},
			StorageInfo {
				pallet_name: b"RelayStorageRoots".to_vec(),
				storage_name: b"RelayStorageRootKeys".to_vec(),
				prefix: prefix(b"RelayStorageRoots", b"RelayStorageRootKeys"),
				max_values: Some(1),
				max_size: Some(121),
			},
		]
	);
}

#[test]
fn test_collectives_storage_item_prefixes() {
	for StorageInfo { pallet_name, .. } in
		<moonbase_runtime::TreasuryCouncilCollective as StorageInfoTrait>::storage_info()
	{
		assert_eq!(pallet_name, b"TreasuryCouncilCollective".to_vec());
	}

	for StorageInfo { pallet_name, .. } in
		<moonbase_runtime::OpenTechCommitteeCollective as StorageInfoTrait>::storage_info()
	{
		assert_eq!(pallet_name, b"OpenTechCommitteeCollective".to_vec());
	}
}

#[test]
fn collective_set_members_root_origin_works() {
	ExtBuilder::default().build().execute_with(|| {
		// TreasuryCouncilCollective
		assert_ok!(TreasuryCouncilCollective::set_members(
			<Runtime as frame_system::Config>::RuntimeOrigin::root(),
			vec![AccountId::from(ALICE), AccountId::from(BOB)],
			Some(AccountId::from(ALICE)),
			2
		));
		// OpenTechCommitteeCollective
		assert_ok!(OpenTechCommitteeCollective::set_members(
			<Runtime as frame_system::Config>::RuntimeOrigin::root(),
			vec![AccountId::from(ALICE), AccountId::from(BOB)],
			Some(AccountId::from(ALICE)),
			2
		));
	});
}

#[test]
fn collective_set_members_general_admin_origin_works() {
	use moonbase_runtime::{
		governance::custom_origins::Origin as CustomOrigin, OriginCaller, Utility,
	};

	ExtBuilder::default().build().execute_with(|| {
		let root_caller = <Runtime as frame_system::Config>::RuntimeOrigin::root();
		let alice = AccountId::from(ALICE);

		// TreasuryCouncilCollective
		let _ = Utility::dispatch_as(
			root_caller.clone(),
			Box::new(OriginCaller::Origins(CustomOrigin::GeneralAdmin)),
			Box::new(
				pallet_collective::Call::<Runtime, pallet_collective::Instance3>::set_members {
					new_members: vec![alice, AccountId::from(BOB)],
					prime: Some(alice),
					old_count: 2,
				}
				.into(),
			),
		);
		// OpenTechCommitteeCollective
		let _ = Utility::dispatch_as(
			root_caller,
			Box::new(OriginCaller::Origins(CustomOrigin::GeneralAdmin)),
			Box::new(
				pallet_collective::Call::<Runtime, pallet_collective::Instance4>::set_members {
					new_members: vec![alice, AccountId::from(BOB)],
					prime: Some(alice),
					old_count: 2,
				}
				.into(),
			),
		);

		assert_eq!(
			System::events()
				.into_iter()
				.filter_map(|r| {
					match r.event {
						RuntimeEvent::Utility(pallet_utility::Event::DispatchedAs { result })
							if result.is_ok() =>
						{
							Some(true)
						}
						_ => None,
					}
				})
				.collect::<Vec<_>>()
				.len(),
			2
		)
	});
}

#[test]
fn collective_set_members_signed_origin_does_not_work() {
	let alice = AccountId::from(ALICE);
	ExtBuilder::default().build().execute_with(|| {
		// TreasuryCouncilCollective
		assert!(TreasuryCouncilCollective::set_members(
			<Runtime as frame_system::Config>::RuntimeOrigin::signed(alice),
			vec![AccountId::from(ALICE), AccountId::from(BOB)],
			Some(AccountId::from(ALICE)),
			2
		)
		.is_err());
		// OpenTechCommitteeCollective
		assert!(OpenTechCommitteeCollective::set_members(
			<Runtime as frame_system::Config>::RuntimeOrigin::signed(alice),
			vec![AccountId::from(ALICE), AccountId::from(BOB)],
			Some(AccountId::from(ALICE)),
			2
		)
		.is_err());
	});
}

#[test]
fn verify_pallet_indices() {
	fn is_pallet_index<P: 'static>(index: usize) {
		assert_eq!(
			<moonbase_runtime::Runtime as frame_system::Config>::PalletInfo::index::<P>(),
			Some(index)
		);
	}
	is_pallet_index::<moonbase_runtime::System>(0);
	is_pallet_index::<moonbase_runtime::Utility>(1);
	is_pallet_index::<moonbase_runtime::Balances>(3);
	is_pallet_index::<moonbase_runtime::Sudo>(4);
	is_pallet_index::<moonbase_runtime::ParachainSystem>(6);
	is_pallet_index::<moonbase_runtime::TransactionPayment>(7);
	is_pallet_index::<moonbase_runtime::ParachainInfo>(8);
	is_pallet_index::<moonbase_runtime::EthereumChainId>(9);
	is_pallet_index::<moonbase_runtime::EVM>(10);
	is_pallet_index::<moonbase_runtime::Ethereum>(11);
	is_pallet_index::<moonbase_runtime::ParachainStaking>(12);
	is_pallet_index::<moonbase_runtime::Scheduler>(13);
	//is_pallet_index::<moonbase_runtime::Democracy>(14); Removed
	is_pallet_index::<moonbase_runtime::Treasury>(17);
	is_pallet_index::<moonbase_runtime::AuthorInherent>(18);
	is_pallet_index::<moonbase_runtime::AuthorFilter>(19);
	is_pallet_index::<moonbase_runtime::CrowdloanRewards>(20);
	is_pallet_index::<moonbase_runtime::AuthorMapping>(21);
	is_pallet_index::<moonbase_runtime::Proxy>(22);
	is_pallet_index::<moonbase_runtime::MaintenanceMode>(23);
	is_pallet_index::<moonbase_runtime::Identity>(24);
	is_pallet_index::<moonbase_runtime::XcmpQueue>(25);
	is_pallet_index::<moonbase_runtime::CumulusXcm>(26);
	is_pallet_index::<moonbase_runtime::DmpQueue>(27);
	is_pallet_index::<moonbase_runtime::PolkadotXcm>(28);
	is_pallet_index::<moonbase_runtime::Assets>(29);
	is_pallet_index::<moonbase_runtime::XTokens>(30);
	is_pallet_index::<moonbase_runtime::AssetManager>(31);
	is_pallet_index::<moonbase_runtime::Migrations>(32);
	is_pallet_index::<moonbase_runtime::XcmTransactor>(33);
	is_pallet_index::<moonbase_runtime::ProxyGenesisCompanion>(34);
	is_pallet_index::<moonbase_runtime::MoonbeamOrbiters>(37);
	is_pallet_index::<moonbase_runtime::EthereumXcm>(38);
	is_pallet_index::<moonbase_runtime::Randomness>(39);
	is_pallet_index::<moonbase_runtime::TreasuryCouncilCollective>(40);
	is_pallet_index::<moonbase_runtime::OpenTechCommitteeCollective>(46);
	is_pallet_index::<moonbase_runtime::MoonbeamLazyMigrations>(51);
}

#[test]
fn verify_reserved_indices() {
	use frame_metadata::*;

	let mut t: sp_io::TestExternalities = frame_system::GenesisConfig::<Runtime>::default()
		.build_storage()
		.unwrap()
		.into();

	t.execute_with(|| {
		let metadata = moonbase_runtime::Runtime::metadata();
		let metadata = match metadata.1 {
			RuntimeMetadata::V14(metadata) => metadata,
			_ => panic!("metadata has been bumped, test needs to be updated"),
		};
		// 35: BaseFee
		// 36: pallet_assets::<Instance1>
		let reserved = vec![35, 36];
		let existing = metadata
			.pallets
			.iter()
			.map(|p| p.index)
			.collect::<Vec<u8>>();
		assert!(reserved.iter().all(|index| !existing.contains(index)));
	});
}

#[test]
fn verify_proxy_type_indices() {
	assert_eq!(moonbase_runtime::ProxyType::Any as u8, 0);
	assert_eq!(moonbase_runtime::ProxyType::NonTransfer as u8, 1);
	assert_eq!(moonbase_runtime::ProxyType::Governance as u8, 2);
	assert_eq!(moonbase_runtime::ProxyType::Staking as u8, 3);
	assert_eq!(moonbase_runtime::ProxyType::CancelProxy as u8, 4);
	assert_eq!(moonbase_runtime::ProxyType::Balances as u8, 5);
	assert_eq!(moonbase_runtime::ProxyType::AuthorMapping as u8, 6);
	assert_eq!(moonbase_runtime::ProxyType::IdentityJudgement as u8, 7);
}

#[test]
fn join_collator_candidates() {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * UNIT),
			(AccountId::from(BOB), 2_000 * UNIT),
			(AccountId::from(CHARLIE), 1_100 * UNIT),
			(AccountId::from(DAVE), 1_000 * UNIT),
		])
		.with_collators(vec![
			(AccountId::from(ALICE), 1_000 * UNIT),
			(AccountId::from(BOB), 1_000 * UNIT),
		])
		.with_delegations(vec![
			(AccountId::from(CHARLIE), AccountId::from(ALICE), 50 * UNIT),
			(AccountId::from(CHARLIE), AccountId::from(BOB), 50 * UNIT),
		])
		.build()
		.execute_with(|| {
			assert_noop!(
				ParachainStaking::join_candidates(
					origin_of(AccountId::from(ALICE)),
					1_000 * UNIT,
					2u32
				),
				pallet_parachain_staking::Error::<Runtime>::CandidateExists
			);
			assert_noop!(
				ParachainStaking::join_candidates(
					origin_of(AccountId::from(CHARLIE)),
					1_000 * UNIT,
					2u32
				),
				pallet_parachain_staking::Error::<Runtime>::DelegatorExists
			);
			assert!(System::events().is_empty());
			assert_ok!(ParachainStaking::join_candidates(
				origin_of(AccountId::from(DAVE)),
				1_000 * UNIT,
				2u32
			));
			assert_eq!(
				last_event(),
				RuntimeEvent::ParachainStaking(
					pallet_parachain_staking::Event::JoinedCollatorCandidates {
						account: AccountId::from(DAVE),
						amount_locked: 1_000 * UNIT,
						new_total_amt_locked: 3_100 * UNIT
					}
				)
			);
			let candidates = ParachainStaking::candidate_pool();
			assert_eq!(candidates.0[0].owner, AccountId::from(ALICE));
			assert_eq!(candidates.0[0].amount, 1_050 * UNIT);
			assert_eq!(candidates.0[1].owner, AccountId::from(BOB));
			assert_eq!(candidates.0[1].amount, 1_050 * UNIT);
			assert_eq!(candidates.0[2].owner, AccountId::from(DAVE));
			assert_eq!(candidates.0[2].amount, 1_000 * UNIT);
		});
}

#[test]
fn transfer_through_evm_to_stake() {
	ExtBuilder::default()
		.with_balances(vec![(AccountId::from(ALICE), 2_000 * UNIT)])
		.build()
		.execute_with(|| {
			// Charlie has no balance => fails to stake
			assert_noop!(
				ParachainStaking::join_candidates(
					origin_of(AccountId::from(CHARLIE)),
					1_000 * UNIT,
					0u32
				),
				DispatchError::Module(ModuleError {
					index: 12,
					error: [8, 0, 0, 0],
					message: Some("InsufficientBalance")
				})
			);

			// Alice transfer from free balance 2000 UNIT to Bob
			assert_ok!(Balances::transfer_allow_death(
				origin_of(AccountId::from(ALICE)),
				AccountId::from(BOB),
				2_000 * UNIT,
			));
			assert_eq!(Balances::free_balance(AccountId::from(BOB)), 2_000 * UNIT);

			let gas_limit = 100000u64;
			// Bob transfers 1000 UNIT to Charlie via EVM
			assert_ok!(RuntimeCall::EVM(pallet_evm::Call::<Runtime>::call {
				source: H160::from(BOB),
				target: H160::from(CHARLIE),
				input: Vec::new(),
				value: (1_000 * UNIT).into(),
				gas_limit,
				max_fee_per_gas: U256::from(BASE_FEE_GENISIS),
				max_priority_fee_per_gas: None,
				nonce: None,
				access_list: Vec::new(),
			})
			.dispatch(<Runtime as frame_system::Config>::RuntimeOrigin::root()));
			assert_eq!(
				Balances::free_balance(AccountId::from(CHARLIE)),
				1_000 * UNIT,
			);

			// Charlie can stake now
			assert_ok!(ParachainStaking::join_candidates(
				origin_of(AccountId::from(CHARLIE)),
				1_000 * UNIT,
				0u32,
			),);
			let candidates = ParachainStaking::candidate_pool();
			assert_eq!(candidates.0[0].owner, AccountId::from(CHARLIE));
			assert_eq!(candidates.0[0].amount, 1_000 * UNIT);
		});
}

#[test]
fn reward_block_authors() {
	ExtBuilder::default()
		.with_balances(vec![
			// Alice gets 100 extra tokens for her mapping deposit
			(AccountId::from(ALICE), 2_100 * UNIT),
			(AccountId::from(BOB), 1_000 * UNIT),
		])
		.with_collators(vec![(AccountId::from(ALICE), 1_000 * UNIT)])
		.with_delegations(vec![(
			AccountId::from(BOB),
			AccountId::from(ALICE),
			500 * UNIT,
		)])
		.with_mappings(vec![(
			NimbusId::from_slice(&ALICE_NIMBUS).unwrap(),
			AccountId::from(ALICE),
		)])
		.build()
		.execute_with(|| {
			increase_last_relay_slot_number(1);
			// Just before round 3
			run_to_block(2399, Some(NimbusId::from_slice(&ALICE_NIMBUS).unwrap()));
			// no rewards doled out yet
			assert_eq!(
				Balances::usable_balance(AccountId::from(ALICE)),
				1_100 * UNIT,
			);
			assert_eq!(Balances::usable_balance(AccountId::from(BOB)), 500 * UNIT,);
			run_to_block(2401, Some(NimbusId::from_slice(&ALICE_NIMBUS).unwrap()));
			// rewards minted and distributed
			assert_eq!(
				Balances::usable_balance(AccountId::from(ALICE)),
				1213666666584000000000,
			);
			assert_eq!(
				Balances::usable_balance(AccountId::from(BOB)),
				541333333292000000000,
			);
		});
}

#[test]
fn reward_block_authors_with_parachain_bond_reserved() {
	ExtBuilder::default()
		.with_balances(vec![
			// Alice gets 100 extra tokens for her mapping deposit
			(AccountId::from(ALICE), 2_100 * UNIT),
			(AccountId::from(BOB), 1_000 * UNIT),
			(AccountId::from(CHARLIE), UNIT),
		])
		.with_collators(vec![(AccountId::from(ALICE), 1_000 * UNIT)])
		.with_delegations(vec![(
			AccountId::from(BOB),
			AccountId::from(ALICE),
			500 * UNIT,
		)])
		.with_mappings(vec![(
			NimbusId::from_slice(&ALICE_NIMBUS).unwrap(),
			AccountId::from(ALICE),
		)])
		.build()
		.execute_with(|| {
			increase_last_relay_slot_number(1);
			assert_ok!(ParachainStaking::set_parachain_bond_account(
				root_origin(),
				AccountId::from(CHARLIE),
			),);

			// Stop just before round 2
			run_to_block(1199, Some(NimbusId::from_slice(&ALICE_NIMBUS).unwrap()));

			// no rewards doled out yet
			assert_eq!(
				Balances::usable_balance(AccountId::from(ALICE)),
				1_100 * UNIT,
			);
			assert_eq!(Balances::usable_balance(AccountId::from(BOB)), 500 * UNIT,);
			assert_eq!(Balances::usable_balance(AccountId::from(CHARLIE)), UNIT,);

			// Go to round 2
			run_to_block(1201, Some(NimbusId::from_slice(&ALICE_NIMBUS).unwrap()));

			// 30% reserved for parachain bond
			assert_eq!(
				Balances::usable_balance(AccountId::from(CHARLIE)),
				47515000000000000000,
			);

			// Go to round 3
			run_to_block(2401, Some(NimbusId::from_slice(&ALICE_NIMBUS).unwrap()));
			// rewards minted and distributed
			assert_eq!(
				Balances::usable_balance(AccountId::from(ALICE)),
				1182693333281650000000,
			);
			assert_eq!(
				Balances::usable_balance(AccountId::from(BOB)),
				525841666640825000000,
			);
			// 30% again reserved for parachain bond
			assert_eq!(
				Balances::usable_balance(AccountId::from(CHARLIE)),
				94727725000000000000,
			);
		});
}

#[test]
fn initialize_crowdloan_addresses_with_batch_and_pay() {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * UNIT),
			(AccountId::from(BOB), 1_000 * UNIT),
		])
		.with_collators(vec![(AccountId::from(ALICE), 1_000 * UNIT)])
		.with_mappings(vec![(
			NimbusId::from_slice(&ALICE_NIMBUS).unwrap(),
			AccountId::from(ALICE),
		)])
		.with_crowdloan_fund(3_000_000 * UNIT)
		.build()
		.execute_with(|| {
			// set parachain inherent data
			set_parachain_inherent_data();
			let init_block = CrowdloanRewards::init_vesting_block();
			// This matches the previous vesting
			let end_block = init_block + 4 * WEEKS;
			// Batch calls always succeed. We just need to check the inner event
			assert_ok!(
				RuntimeCall::Utility(pallet_utility::Call::<Runtime>::batch_all {
					calls: vec![
						RuntimeCall::CrowdloanRewards(
							pallet_crowdloan_rewards::Call::<Runtime>::initialize_reward_vec {
								rewards: vec![(
									[4u8; 32].into(),
									Some(AccountId::from(CHARLIE)),
									1_500_000 * UNIT
								)]
							}
						),
						RuntimeCall::CrowdloanRewards(
							pallet_crowdloan_rewards::Call::<Runtime>::initialize_reward_vec {
								rewards: vec![(
									[5u8; 32].into(),
									Some(AccountId::from(DAVE)),
									1_500_000 * UNIT
								)]
							}
						),
						RuntimeCall::CrowdloanRewards(
							pallet_crowdloan_rewards::Call::<Runtime>::complete_initialization {
								lease_ending_block: end_block
							}
						)
					]
				})
				.dispatch(root_origin())
			);
			// 30 percent initial payout
			assert_eq!(Balances::balance(&AccountId::from(CHARLIE)), 450_000 * UNIT);
			// 30 percent initial payout
			assert_eq!(Balances::balance(&AccountId::from(DAVE)), 450_000 * UNIT);
			let expected = RuntimeEvent::Utility(pallet_utility::Event::BatchCompleted);
			assert_eq!(last_event(), expected);
			// This one should fail, as we already filled our data
			assert_ok!(
				RuntimeCall::Utility(pallet_utility::Call::<Runtime>::batch {
					calls: vec![RuntimeCall::CrowdloanRewards(
						pallet_crowdloan_rewards::Call::<Runtime>::initialize_reward_vec {
							rewards: vec![([4u8; 32].into(), Some(AccountId::from(ALICE)), 432000)]
						}
					)]
				})
				.dispatch(root_origin())
			);
			let expected_fail = RuntimeEvent::Utility(pallet_utility::Event::BatchInterrupted {
				index: 0,
				error: DispatchError::Module(ModuleError {
					index: 20,
					error: [8, 0, 0, 0],
					message: None,
				}),
			});
			assert_eq!(last_event(), expected_fail);
			// Claim 1 block.
			assert_ok!(CrowdloanRewards::claim(origin_of(AccountId::from(CHARLIE))));
			assert_ok!(CrowdloanRewards::claim(origin_of(AccountId::from(DAVE))));

			let vesting_period = 4 * WEEKS as u128;
			let per_block = (1_050_000 * UNIT) / vesting_period;

			assert_eq!(
				CrowdloanRewards::accounts_payable(&AccountId::from(CHARLIE))
					.unwrap()
					.claimed_reward,
				(450_000 * UNIT) + per_block
			);
			assert_eq!(
				CrowdloanRewards::accounts_payable(&AccountId::from(DAVE))
					.unwrap()
					.claimed_reward,
				(450_000 * UNIT) + per_block
			);
			// The total claimed reward should be equal to the account balance at this point.
			assert_eq!(
				Balances::balance(&AccountId::from(CHARLIE)),
				(450_000 * UNIT) + per_block
			);
			assert_eq!(
				Balances::balance(&AccountId::from(DAVE)),
				(450_000 * UNIT) + per_block
			);
			assert_noop!(
				CrowdloanRewards::claim(origin_of(AccountId::from(ALICE))),
				pallet_crowdloan_rewards::Error::<Runtime>::NoAssociatedClaim
			);
		});
}

#[test]
fn initialize_crowdloan_address_and_change_with_relay_key_sig() {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * UNIT),
			(AccountId::from(BOB), 1_000 * UNIT),
		])
		.with_collators(vec![(AccountId::from(ALICE), 1_000 * UNIT)])
		.with_mappings(vec![(
			NimbusId::from_slice(&ALICE_NIMBUS).unwrap(),
			AccountId::from(ALICE),
		)])
		.with_crowdloan_fund(3_000_000 * UNIT)
		.build()
		.execute_with(|| {
			// set parachain inherent data
			let init_block = CrowdloanRewards::init_vesting_block();
			// This matches the previous vesting
			let end_block = init_block + 4 * WEEKS;

			let (pair1, _) = sp_core::sr25519::Pair::generate();
			let (pair2, _) = sp_core::sr25519::Pair::generate();

			let public1 = pair1.public();
			let public2 = pair2.public();

			// signature:
			// WRAP_BYTES|| NetworkIdentifier|| new_account || previous_account || WRAP_BYTES
			let mut message = pallet_crowdloan_rewards::WRAPPED_BYTES_PREFIX.to_vec();
			message.append(&mut b"moonbase-".to_vec());
			message.append(&mut AccountId::from(DAVE).encode());
			message.append(&mut AccountId::from(CHARLIE).encode());
			message.append(&mut pallet_crowdloan_rewards::WRAPPED_BYTES_POSTFIX.to_vec());

			let signature1 = pair1.sign(&message);
			let signature2 = pair2.sign(&message);

			// Batch calls always succeed. We just need to check the inner event
			assert_ok!(
				// two relay accounts pointing at the same reward account
				RuntimeCall::Utility(pallet_utility::Call::<Runtime>::batch_all {
					calls: vec![
						RuntimeCall::CrowdloanRewards(
							pallet_crowdloan_rewards::Call::<Runtime>::initialize_reward_vec {
								rewards: vec![(
									public1.into(),
									Some(AccountId::from(CHARLIE)),
									1_500_000 * UNIT
								)]
							}
						),
						RuntimeCall::CrowdloanRewards(
							pallet_crowdloan_rewards::Call::<Runtime>::initialize_reward_vec {
								rewards: vec![(
									public2.into(),
									Some(AccountId::from(CHARLIE)),
									1_500_000 * UNIT
								)]
							}
						),
						RuntimeCall::CrowdloanRewards(
							pallet_crowdloan_rewards::Call::<Runtime>::complete_initialization {
								lease_ending_block: end_block
							}
						)
					]
				})
				.dispatch(root_origin())
			);
			// 30 percent initial payout
			assert_eq!(Balances::balance(&AccountId::from(CHARLIE)), 900_000 * UNIT);

			// this should fail, as we are only providing one signature
			assert_noop!(
				CrowdloanRewards::change_association_with_relay_keys(
					origin_of(AccountId::from(CHARLIE)),
					AccountId::from(DAVE),
					AccountId::from(CHARLIE),
					vec![(public1.into(), signature1.clone().into())]
				),
				pallet_crowdloan_rewards::Error::<Runtime>::InsufficientNumberOfValidProofs
			);

			// this should be valid
			assert_ok!(CrowdloanRewards::change_association_with_relay_keys(
				origin_of(AccountId::from(CHARLIE)),
				AccountId::from(DAVE),
				AccountId::from(CHARLIE),
				vec![
					(public1.into(), signature1.into()),
					(public2.into(), signature2.into())
				]
			));

			assert_eq!(
				CrowdloanRewards::accounts_payable(&AccountId::from(DAVE))
					.unwrap()
					.claimed_reward,
				(900_000 * UNIT)
			);
		});
}

#[test]
fn claim_via_precompile() {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * UNIT),
			(AccountId::from(BOB), 1_000 * UNIT),
		])
		.with_collators(vec![(AccountId::from(ALICE), 1_000 * UNIT)])
		.with_mappings(vec![(
			NimbusId::from_slice(&ALICE_NIMBUS).unwrap(),
			AccountId::from(ALICE),
		)])
		.with_crowdloan_fund(3_000_000 * UNIT)
		.build()
		.execute_with(|| {
			// set parachain inherent data
			set_parachain_inherent_data();
			let init_block = CrowdloanRewards::init_vesting_block();
			// This matches the previous vesting
			let end_block = init_block + 4 * WEEKS;
			// Batch calls always succeed. We just need to check the inner event
			assert_ok!(
				RuntimeCall::Utility(pallet_utility::Call::<Runtime>::batch_all {
					calls: vec![
						RuntimeCall::CrowdloanRewards(
							pallet_crowdloan_rewards::Call::<Runtime>::initialize_reward_vec {
								rewards: vec![(
									[4u8; 32].into(),
									Some(AccountId::from(CHARLIE)),
									1_500_000 * UNIT
								)]
							}
						),
						RuntimeCall::CrowdloanRewards(
							pallet_crowdloan_rewards::Call::<Runtime>::initialize_reward_vec {
								rewards: vec![(
									[5u8; 32].into(),
									Some(AccountId::from(DAVE)),
									1_500_000 * UNIT
								)]
							}
						),
						RuntimeCall::CrowdloanRewards(
							pallet_crowdloan_rewards::Call::<Runtime>::complete_initialization {
								lease_ending_block: end_block
							}
						)
					]
				})
				.dispatch(root_origin())
			);

			// 30 percent initial payout
			assert_eq!(Balances::balance(&AccountId::from(CHARLIE)), 450_000 * UNIT);
			// 30 percent initial payout
			assert_eq!(Balances::balance(&AccountId::from(DAVE)), 450_000 * UNIT);

			let crowdloan_precompile_address = H160::from_low_u64_be(2049);

			// Alice uses the crowdloan precompile to claim through the EVM
			let gas_limit = 100000u64;
			let gas_price: U256 = BASE_FEE_GENISIS.into();

			// Construct the call data (selector, amount)
			let mut call_data = Vec::<u8>::from([0u8; 4]);
			call_data[0..4].copy_from_slice(&Keccak256::digest(b"claim()")[0..4]);

			assert_ok!(RuntimeCall::EVM(pallet_evm::Call::<Runtime>::call {
				source: H160::from(CHARLIE),
				target: crowdloan_precompile_address,
				input: call_data,
				value: U256::zero(), // No value sent in EVM
				gas_limit,
				max_fee_per_gas: gas_price,
				max_priority_fee_per_gas: None,
				nonce: None, // Use the next nonce
				access_list: Vec::new(),
			})
			.dispatch(<Runtime as frame_system::Config>::RuntimeOrigin::root()));

			let vesting_period = 4 * WEEKS as u128;
			let per_block = (1_050_000 * UNIT) / vesting_period;

			assert_eq!(
				CrowdloanRewards::accounts_payable(&AccountId::from(CHARLIE))
					.unwrap()
					.claimed_reward,
				(450_000 * UNIT) + per_block
			);
		})
}

#[test]
fn is_contributor_via_precompile() {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * UNIT),
			(AccountId::from(BOB), 1_000 * UNIT),
		])
		.with_collators(vec![(AccountId::from(ALICE), 1_000 * UNIT)])
		.with_mappings(vec![(
			NimbusId::from_slice(&ALICE_NIMBUS).unwrap(),
			AccountId::from(ALICE),
		)])
		.with_crowdloan_fund(3_000_000 * UNIT)
		.build()
		.execute_with(|| {
			// set parachain inherent data
			let init_block = CrowdloanRewards::init_vesting_block();
			// This matches the previous vesting
			let end_block = init_block + 4 * WEEKS;
			// Batch calls always succeed. We just need to check the inner event
			assert_ok!(
				RuntimeCall::Utility(pallet_utility::Call::<Runtime>::batch_all {
					calls: vec![
						RuntimeCall::CrowdloanRewards(
							pallet_crowdloan_rewards::Call::<Runtime>::initialize_reward_vec {
								rewards: vec![(
									[4u8; 32].into(),
									Some(AccountId::from(CHARLIE)),
									1_500_000 * UNIT
								)]
							}
						),
						RuntimeCall::CrowdloanRewards(
							pallet_crowdloan_rewards::Call::<Runtime>::initialize_reward_vec {
								rewards: vec![(
									[5u8; 32].into(),
									Some(AccountId::from(DAVE)),
									1_500_000 * UNIT
								)]
							}
						),
						RuntimeCall::CrowdloanRewards(
							pallet_crowdloan_rewards::Call::<Runtime>::complete_initialization {
								lease_ending_block: end_block
							}
						)
					]
				})
				.dispatch(root_origin())
			);

			let crowdloan_precompile_address = H160::from_low_u64_be(2049);

			// Assert precompile reports Bob is not a contributor
			Precompiles::new()
				.prepare_test(
					ALICE,
					crowdloan_precompile_address,
					CrowdloanRewardsPCall::is_contributor {
						contributor: Address(BOB.into()),
					},
				)
				.expect_cost(1000)
				.expect_no_logs()
				.execute_returns(false);

			// Assert precompile reports Charlie is a nominator
			Precompiles::new()
				.prepare_test(
					ALICE,
					crowdloan_precompile_address,
					CrowdloanRewardsPCall::is_contributor {
						contributor: Address(CHARLIE.into()),
					},
				)
				.expect_cost(1000)
				.expect_no_logs()
				.execute_returns(true);
		})
}

#[test]
fn reward_info_via_precompile() {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * UNIT),
			(AccountId::from(BOB), 1_000 * UNIT),
		])
		.with_collators(vec![(AccountId::from(ALICE), 1_000 * UNIT)])
		.with_mappings(vec![(
			NimbusId::from_slice(&ALICE_NIMBUS).unwrap(),
			AccountId::from(ALICE),
		)])
		.with_crowdloan_fund(3_000_000 * UNIT)
		.build()
		.execute_with(|| {
			// set parachain inherent data
			let init_block = CrowdloanRewards::init_vesting_block();
			// This matches the previous vesting
			let end_block = init_block + 4 * WEEKS;
			// Batch calls always succeed. We just need to check the inner event
			assert_ok!(
				RuntimeCall::Utility(pallet_utility::Call::<Runtime>::batch_all {
					calls: vec![
						RuntimeCall::CrowdloanRewards(
							pallet_crowdloan_rewards::Call::<Runtime>::initialize_reward_vec {
								rewards: vec![(
									[4u8; 32].into(),
									Some(AccountId::from(CHARLIE)),
									1_500_000 * UNIT
								)]
							}
						),
						RuntimeCall::CrowdloanRewards(
							pallet_crowdloan_rewards::Call::<Runtime>::initialize_reward_vec {
								rewards: vec![(
									[5u8; 32].into(),
									Some(AccountId::from(DAVE)),
									1_500_000 * UNIT
								)]
							}
						),
						RuntimeCall::CrowdloanRewards(
							pallet_crowdloan_rewards::Call::<Runtime>::complete_initialization {
								lease_ending_block: end_block
							}
						)
					]
				})
				.dispatch(root_origin())
			);

			let crowdloan_precompile_address = H160::from_low_u64_be(2049);

			let expected_total: U256 = (1_500_000 * UNIT).into();
			let expected_claimed: U256 = (450_000 * UNIT).into();

			// Assert precompile reports correct Charlie reward info.
			Precompiles::new()
				.prepare_test(
					ALICE,
					crowdloan_precompile_address,
					CrowdloanRewardsPCall::reward_info {
						contributor: Address(AccountId::from(CHARLIE).into()),
					},
				)
				.expect_cost(1000)
				.expect_no_logs()
				.execute_returns((expected_total, expected_claimed));
		})
}

#[test]
fn update_reward_address_via_precompile() {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * UNIT),
			(AccountId::from(BOB), 1_000 * UNIT),
		])
		.with_collators(vec![(AccountId::from(ALICE), 1_000 * UNIT)])
		.with_mappings(vec![(
			NimbusId::from_slice(&ALICE_NIMBUS).unwrap(),
			AccountId::from(ALICE),
		)])
		.with_crowdloan_fund(3_000_000 * UNIT)
		.build()
		.execute_with(|| {
			// set parachain inherent data
			let init_block = CrowdloanRewards::init_vesting_block();
			// This matches the previous vesting
			let end_block = init_block + 4 * WEEKS;
			// Batch calls always succeed. We just need to check the inner event
			assert_ok!(
				RuntimeCall::Utility(pallet_utility::Call::<Runtime>::batch_all {
					calls: vec![
						RuntimeCall::CrowdloanRewards(
							pallet_crowdloan_rewards::Call::<Runtime>::initialize_reward_vec {
								rewards: vec![(
									[4u8; 32].into(),
									Some(AccountId::from(CHARLIE)),
									1_500_000 * UNIT
								)]
							}
						),
						RuntimeCall::CrowdloanRewards(
							pallet_crowdloan_rewards::Call::<Runtime>::initialize_reward_vec {
								rewards: vec![(
									[5u8; 32].into(),
									Some(AccountId::from(DAVE)),
									1_500_000 * UNIT
								)]
							}
						),
						RuntimeCall::CrowdloanRewards(
							pallet_crowdloan_rewards::Call::<Runtime>::complete_initialization {
								lease_ending_block: end_block
							}
						)
					]
				})
				.dispatch(root_origin())
			);

			let crowdloan_precompile_address = H160::from_low_u64_be(2049);

			// Charlie uses the crowdloan precompile to update address through the EVM
			let gas_limit = 100000u64;
			let gas_price: U256 = BASE_FEE_GENISIS.into();

			// Construct the input data to check if Bob is a contributor
			let mut call_data = Vec::<u8>::from([0u8; 36]);
			call_data[0..4]
				.copy_from_slice(&Keccak256::digest(b"update_reward_address(address)")[0..4]);
			call_data[16..36].copy_from_slice(&ALICE);

			assert_ok!(RuntimeCall::EVM(pallet_evm::Call::<Runtime>::call {
				source: H160::from(CHARLIE),
				target: crowdloan_precompile_address,
				input: call_data,
				value: U256::zero(), // No value sent in EVM
				gas_limit,
				max_fee_per_gas: gas_price,
				max_priority_fee_per_gas: None,
				nonce: None, // Use the next nonce
				access_list: Vec::new(),
			})
			.dispatch(<Runtime as frame_system::Config>::RuntimeOrigin::root()));

			assert!(CrowdloanRewards::accounts_payable(&AccountId::from(CHARLIE)).is_none());
			assert_eq!(
				CrowdloanRewards::accounts_payable(&AccountId::from(ALICE))
					.unwrap()
					.claimed_reward,
				(450_000 * UNIT)
			);
		})
}

#[test]
fn create_and_manipulate_foreign_asset() {
	ExtBuilder::default().build().execute_with(|| {
		let source_location = xcm::v4::Location::parent();

		// Create foreign asset
		assert_ok!(EvmForeignAssets::create_foreign_asset(
			moonbase_runtime::RuntimeOrigin::root(),
			1,
			source_location.clone(),
			12,
			bounded_vec![b'M', b'T'],
			bounded_vec![b'M', b'y', b'T', b'o', b'k'],
		));
		assert_eq!(
			EvmForeignAssets::assets_by_id(1),
			Some(source_location.clone())
		);
		assert_eq!(
			EvmForeignAssets::assets_by_location(&source_location),
			Some((1, AssetStatus::Active))
		);

		// Freeze foreign asset
		assert_ok!(EvmForeignAssets::freeze_foreign_asset(
			moonbase_runtime::RuntimeOrigin::root(),
			1,
			true
		));
		assert_eq!(
			EvmForeignAssets::assets_by_location(&source_location),
			Some((1, AssetStatus::FrozenXcmDepositAllowed))
		);

		// Unfreeze foreign asset
		assert_ok!(EvmForeignAssets::unfreeze_foreign_asset(
			moonbase_runtime::RuntimeOrigin::root(),
			1,
		));
		assert_eq!(
			EvmForeignAssets::assets_by_location(&source_location),
			Some((1, AssetStatus::Active))
		);
	});
}

// The precoompile asset-erc20 is deprecated and not used anymore for new evm foreign assets
// We don't have testing tools in rust test to call real evm smart contract, so we rely on ts tests.
/*
#[test]
fn xcm_asset_erc20_precompiles_supply_and_balance() {
	ExtBuilder::default()
		.with_xcm_assets(vec![XcmAssetInitialization {
			asset_id: 1,
			xcm_location: xcm::v4::Location::parent(),
			name: "RelayToken",
			symbol: "Relay",
			decimals: 12,
			balances: vec![(AccountId::from(ALICE), 1_000 * UNIT)],
		}])
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * UNIT),
			(AccountId::from(BOB), 1_000 * UNIT),
		])
		.build()
		.execute_with(|| {
			// We have the assetId that corresponds to the relay chain registered
			let relay_asset_id: AssetId = AssetType::Xcm(xcm::v3::Location::parent()).into();

			// Its address is
			let asset_precompile_address = Runtime::asset_id_to_account(
				FOREIGN_ASSET_PRECOMPILE_ADDRESS_PREFIX,
				relay_asset_id,
			);

			// Assert the asset has been created with the correct supply
			assert_eq!(Assets::total_supply(relay_asset_id), 1_000 * UNIT);

			// Access totalSupply through precompile. Important that the context is correct
			Precompiles::new()
				.prepare_test(
					ALICE,
					asset_precompile_address,
					ForeignAssetsPCall::total_supply {},
				)
				.expect_cost(2000)
				.expect_no_logs()
				.execute_returns(U256::from(1000 * UNIT));

			// Access balanceOf through precompile
			Precompiles::new()
				.prepare_test(
					ALICE,
					asset_precompile_address,
					ForeignAssetsPCall::balance_of {
						who: Address(ALICE.into()),
					},
				)
				.expect_cost(2000)
				.expect_no_logs()
				.execute_returns(U256::from(1000 * UNIT));
		});
}

#[test]
fn xcm_asset_erc20_precompiles_transfer() {
	ExtBuilder::default()
		.with_xcm_assets(vec![XcmAssetInitialization {
			asset_id: 1,
			xcm_location: xcm::v4::Location::parent(),
			name: "RelayToken",
			symbol: "Relay",
			decimals: 12,
			balances: vec![(AccountId::from(ALICE), 1_000 * UNIT)],
		}])
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * UNIT),
			(AccountId::from(BOB), 1_000 * UNIT),
		])
		.build()
		.execute_with(|| {
			// We have the assetId that corresponds to the relay chain registered
			let relay_asset_id: AssetId = AssetType::Xcm(xcm::v3::Location::parent()).into();

			// Its address is
			let asset_precompile_address = Runtime::asset_id_to_account(
				FOREIGN_ASSET_PRECOMPILE_ADDRESS_PREFIX,
				relay_asset_id,
			);

			// Transfer tokens from Alice to Bob, 400 UNIT.
			Precompiles::new()
				.prepare_test(
					ALICE,
					asset_precompile_address,
					ForeignAssetsPCall::transfer {
						to: Address(BOB.into()),
						value: { 400 * UNIT }.into(),
					},
				)
				.expect_cost(24342)
				.expect_log(log3(
					asset_precompile_address,
					SELECTOR_LOG_TRANSFER,
					H160::from(ALICE),
					H160::from(BOB),
					solidity::encode_event_data(U256::from(400 * UNIT)),
				))
				.execute_returns(true);

			// Make sure BOB has 400 UNIT
			Precompiles::new()
				.prepare_test(
					BOB,
					asset_precompile_address,
					ForeignAssetsPCall::balance_of {
						who: Address(BOB.into()),
					},
				)
				.expect_cost(2000)
				.expect_no_logs()
				.execute_returns(U256::from(400 * UNIT));
		});
}

#[test]
fn xcm_asset_erc20_precompiles_approve() {
	ExtBuilder::default()
		.with_xcm_assets(vec![XcmAssetInitialization {
			asset_id: 1,
			xcm_location: xcm::v4::Location::parent(),
			name: "RelayToken",
			symbol: "Relay",
			decimals: 12,
			balances: vec![(AccountId::from(ALICE), 1_000 * UNIT)],
		}])
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * UNIT),
			(AccountId::from(BOB), 1_000 * UNIT),
		])
		.build()
		.execute_with(|| {
			// We have the assetId that corresponds to the relay chain registered
			let relay_asset_id: AssetId = AssetType::Xcm(xcm::v3::Location::parent()).into();

			// Its address is
			let asset_precompile_address = Runtime::asset_id_to_account(
				FOREIGN_ASSET_PRECOMPILE_ADDRESS_PREFIX,
				relay_asset_id,
			);

			// Aprove Bob for spending 400 UNIT from Alice
			Precompiles::new()
				.prepare_test(
					ALICE,
					asset_precompile_address,
					ForeignAssetsPCall::approve {
						spender: Address(BOB.into()),
						value: { 400 * UNIT }.into(),
					},
				)
				.expect_cost(14424)
				.expect_log(log3(
					asset_precompile_address,
					SELECTOR_LOG_APPROVAL,
					H160::from(ALICE),
					H160::from(BOB),
					solidity::encode_event_data(U256::from(400 * UNIT)),
				))
				.execute_returns(true);

			// Transfer tokens from Alice to Charlie by using BOB as origin
			Precompiles::new()
				.prepare_test(
					BOB,
					asset_precompile_address,
					ForeignAssetsPCall::transfer_from {
						from: Address(ALICE.into()),
						to: Address(CHARLIE.into()),
						value: { 400 * UNIT }.into(),
					},
				)
				.expect_cost(29686)
				.expect_log(log3(
					asset_precompile_address,
					SELECTOR_LOG_TRANSFER,
					H160::from(ALICE),
					H160::from(CHARLIE),
					solidity::encode_event_data(U256::from(400 * UNIT)),
				))
				.execute_returns(true);

			// Make sure CHARLIE has 400 UNIT
			Precompiles::new()
				.prepare_test(
					CHARLIE,
					asset_precompile_address,
					ForeignAssetsPCall::balance_of {
						who: Address(CHARLIE.into()),
					},
				)
				.expect_cost(2000)
				.expect_no_logs()
				.execute_returns(U256::from(400 * UNIT));
		});
}*/

#[test]
fn xtokens_precompiles_transfer() {
	ExtBuilder::default()
		.with_xcm_assets(vec![XcmAssetInitialization {
			asset_id: 1,
			xcm_location: xcm::v4::Location::parent(),
			name: "RelayToken",
			symbol: "Relay",
			decimals: 12,
			balances: vec![(AccountId::from(ALICE), 1_000 * UNIT)],
		}])
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * UNIT),
			(AccountId::from(BOB), 1_000 * UNIT),
		])
		.with_safe_xcm_version(2)
		.build()
		.execute_with(|| {
			let xtokens_precompile_address = H160::from_low_u64_be(2052);

			// We have the assetId that corresponds to the relay chain registered
			let relay_asset_id: AssetId = 1;

			// Its address is
			let asset_precompile_address = Runtime::asset_id_to_account(
				FOREIGN_ASSET_PRECOMPILE_ADDRESS_PREFIX,
				relay_asset_id,
			);

			// Alice has 1000 tokens. She should be able to send through precompile
			let destination = Location::new(
				1,
				[Junction::AccountId32 {
					network: None,
					id: [1u8; 32],
				}],
			);

			let inside = Rc::new(Cell::new(false));
			let inside2 = inside.clone();

			// We use the address of the asset as an identifier of the asset we want to transfer
			Precompiles::new()
				.prepare_test(
					ALICE,
					xtokens_precompile_address,
					XtokensPCall::transfer {
						currency_address: Address(asset_precompile_address.into()),
						amount: 500_000_000_000_000u128.into(),
						destination,
						weight: 4_000_000,
					},
				)
				.expect_cost(346239)
				.expect_no_logs()
				// We expect an evm subcall ERC20.burnFrom
				.with_subcall_handle(move |subcall| {
					let Subcall {
						address,
						transfer,
						input,
						target_gas: _,
						is_static,
						context,
					} = subcall;

					assert_eq!(context.caller, EvmForeignAssets::account_id().into());
					assert_eq!(
						address,
						[255, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1].into()
					);
					assert_eq!(is_static, false);

					assert!(transfer.is_none());

					assert_eq!(
						context.address,
						[255, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1].into()
					);
					assert_eq!(context.apparent_value, 0u8.into());

					assert_eq!(&input[..4], &keccak256!("burnFrom(address,uint256)")[..4]);
					assert_eq!(&input[4..16], &[0u8; 12]);
					assert_eq!(&input[16..36], ALICE);

					inside2.set(true);

					SubcallOutput {
						output: Default::default(),
						cost: 149_000,
						logs: vec![],
						..SubcallOutput::succeed()
					}
				})
				.execute_returns(())
		})
}

#[test]
fn xtokens_precompiles_transfer_multiasset() {
	ExtBuilder::default()
		.with_xcm_assets(vec![XcmAssetInitialization {
			asset_id: 1,
			xcm_location: xcm::v4::Location::parent(),
			name: "RelayToken",
			symbol: "Relay",
			decimals: 12,
			balances: vec![(AccountId::from(ALICE), 1_000 * UNIT)],
		}])
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * UNIT),
			(AccountId::from(BOB), 1_000 * UNIT),
		])
		.with_safe_xcm_version(2)
		.build()
		.execute_with(|| {
			let xtokens_precompile_address = H160::from_low_u64_be(2052);

			// Alice has 1000 tokens. She should be able to send through precompile
			let destination = Location::new(
				1,
				[Junction::AccountId32 {
					network: None,
					id: [1u8; 32],
				}],
			);

			let inside = Rc::new(Cell::new(false));
			let inside2 = inside.clone();

			// This time we transfer it through TransferMultiAsset
			// Instead of the address, we encode directly the multilocation referencing the asset
			Precompiles::new()
				.prepare_test(
					ALICE,
					xtokens_precompile_address,
					XtokensPCall::transfer_multiasset {
						// We want to transfer the relay token
						asset: Location::parent(),
						amount: 500_000_000_000_000u128.into(),
						destination,
						weight: 4_000_000,
					},
				)
				.expect_cost(346239)
				.expect_no_logs()
				// We expect an evm subcall ERC20.burnFrom
				.with_subcall_handle(move |subcall| {
					let Subcall {
						address,
						transfer,
						input,
						target_gas: _,
						is_static,
						context,
					} = subcall;

					assert_eq!(context.caller, EvmForeignAssets::account_id().into());
					assert_eq!(
						address,
						[255, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1].into()
					);
					assert_eq!(is_static, false);

					assert!(transfer.is_none());

					assert_eq!(
						context.address,
						[255, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1].into()
					);
					assert_eq!(context.apparent_value, 0u8.into());

					assert_eq!(&input[..4], &keccak256!("burnFrom(address,uint256)")[..4]);
					assert_eq!(&input[4..16], &[0u8; 12]);
					assert_eq!(&input[16..36], ALICE);

					inside2.set(true);

					SubcallOutput {
						output: Default::default(),
						cost: 149_000,
						logs: vec![],
						..SubcallOutput::succeed()
					}
				})
				.execute_returns(());

			// Ensure that the subcall was actually called.
			assert!(inside.get(), "subcall not called");
		})
}

#[test]
fn xtokens_precompiles_transfer_native() {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * UNIT),
			(AccountId::from(BOB), 1_000 * UNIT),
		])
		.with_safe_xcm_version(2)
		.build()
		.execute_with(|| {
			let xtokens_precompile_address = H160::from_low_u64_be(2052);

			// Its address is
			let asset_precompile_address = H160::from_low_u64_be(2050);

			// Alice has 1000 tokens. She should be able to send through precompile
			let destination = Location::new(
				1,
				[Junction::AccountId32 {
					network: None,
					id: [1u8; 32],
				}],
			);

			// We use the address of the asset as an identifier of the asset we want to transfer
			Precompiles::new()
				.prepare_test(
					ALICE,
					xtokens_precompile_address,
					XtokensPCall::transfer {
						currency_address: Address(asset_precompile_address),
						amount: { 500 * UNIT }.into(),
						destination,
						weight: 4_000_000,
					},
				)
				.expect_cost(16000)
				.expect_no_logs()
				.execute_returns(());
		})
}

fn run_with_system_weight<F>(w: Weight, mut assertions: F)
where
	F: FnMut() -> (),
{
	let mut t: sp_io::TestExternalities = frame_system::GenesisConfig::<Runtime>::default()
		.build_storage()
		.unwrap()
		.into();
	t.execute_with(|| {
		System::set_block_consumed_resources(w, 0);
		assertions()
	});
}

#[test]
#[rustfmt::skip]
fn length_fee_is_sensible() {
	use sp_runtime::testing::TestXt;

	// tests that length fee is sensible for a few hypothetical transactions
	ExtBuilder::default().build().execute_with(|| {
		let call = frame_system::Call::remark::<Runtime> { remark: vec![] };
		let uxt: TestXt<_, ()> = TestXt::new(call, Some((1u64, ())));

		let calc_fee = |len: u32| -> Balance {
			moonbase_runtime::TransactionPayment::query_fee_details(uxt.clone(), len)
				.inclusion_fee
				.expect("fee should be calculated")
				.len_fee
		};

		// editorconfig-checker-disable
		//                  left: cost of length fee, right: size in bytes
		//                             /------------- proportional component: O(N * 1B)
		//                             |           /- exponential component: O(N ** 3)
		//                             |           |
		assert_eq!(                    1_000_000_001, calc_fee(1));
		assert_eq!(                   10_000_001_000, calc_fee(10));
		assert_eq!(                  100_001_000_000, calc_fee(100));
		assert_eq!(                1_001_000_000_000, calc_fee(1_000));
		assert_eq!(               11_000_000_000_000, calc_fee(10_000)); // inflection point
		assert_eq!(            1_100_000_000_000_000, calc_fee(100_000));
		assert_eq!(        1_001_000_000_000_000_000, calc_fee(1_000_000)); // one UNIT, ~ 1MB
		assert_eq!(    1_000_010_000_000_000_000_000, calc_fee(10_000_000));
		assert_eq!(1_000_000_100_000_000_000_000_000, calc_fee(100_000_000));
		// editorconfig-checker-enable
	});
}

#[test]
fn multiplier_can_grow_from_zero() {
	use frame_support::traits::Get;

	let minimum_multiplier = moonbase_runtime::MinimumMultiplier::get();
	let target = moonbase_runtime::TargetBlockFullness::get()
		* RuntimeBlockWeights::get()
			.get(DispatchClass::Normal)
			.max_total
			.unwrap();
	// if the min is too small, then this will not change, and we are doomed forever.
	// the weight is 1/100th bigger than target.
	run_with_system_weight(target * 101 / 100, || {
		let next = moonbase_runtime::FastAdjustingFeeUpdate::<Runtime>::convert(minimum_multiplier);
		assert!(
			next > minimum_multiplier,
			"{:?} !>= {:?}",
			next,
			minimum_multiplier
		);
	})
}

#[test]
fn ethereum_invalid_transaction() {
	ExtBuilder::default().build().execute_with(|| {
		// Ensure an extrinsic not containing enough gas limit to store the transaction
		// on chain is rejected.
		assert_eq!(
			Executive::apply_extrinsic(unchecked_eth_tx(INVALID_ETH_TX)),
			Err(
				sp_runtime::transaction_validity::TransactionValidityError::Invalid(
					sp_runtime::transaction_validity::InvalidTransaction::Custom(0u8)
				)
			)
		);
	});
}

#[test]
fn transfer_ed_0_substrate() {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), (1 * UNIT) + (1 * WEI)),
			(AccountId::from(BOB), existential_deposit()),
		])
		.build()
		.execute_with(|| {
			// Substrate transfer
			assert_ok!(Balances::transfer_allow_death(
				origin_of(AccountId::from(ALICE)),
				AccountId::from(BOB),
				1 * UNIT,
			));
			// 1 WEI is left in the account
			assert_eq!(Balances::free_balance(AccountId::from(ALICE)), 1 * WEI);
		});
}

#[test]
fn initial_gas_fee_is_correct() {
	use fp_evm::FeeCalculator;

	ExtBuilder::default().build().execute_with(|| {
		let multiplier = TransactionPayment::next_fee_multiplier();
		assert_eq!(multiplier, Multiplier::from(8u128));

		assert_eq!(
			TransactionPaymentAsGasPrice::min_gas_price(),
			(
				10_000_000_000u128.into(),
				Weight::from_parts(25_000_000u64, 0)
			)
		);
	});
}

#[test]
fn transfer_ed_0_evm() {
	ExtBuilder::default()
		.with_balances(vec![
			(
				AccountId::from(ALICE),
				((1 * UNIT) + (21_000 * BASE_FEE_GENISIS)) + (1 * WEI),
			),
			(AccountId::from(BOB), existential_deposit()),
		])
		.build()
		.execute_with(|| {
			// EVM transfer
			assert_ok!(RuntimeCall::EVM(pallet_evm::Call::<Runtime>::call {
				source: H160::from(ALICE),
				target: H160::from(BOB),
				input: Vec::new(),
				value: (1 * UNIT).into(),
				gas_limit: 21_000u64,
				max_fee_per_gas: U256::from(BASE_FEE_GENISIS),
				max_priority_fee_per_gas: None,
				nonce: Some(U256::from(0)),
				access_list: Vec::new(),
			})
			.dispatch(<Runtime as frame_system::Config>::RuntimeOrigin::root()));
			// 1 WEI is left in the account
			assert_eq!(Balances::free_balance(AccountId::from(ALICE)), 1 * WEI,);
		});
}

#[test]
fn refund_ed_0_evm() {
	ExtBuilder::default()
		.with_balances(vec![
			(
				AccountId::from(ALICE),
				((1 * UNIT) + (21_777 * BASE_FEE_GENISIS) + existential_deposit()),
			),
			(AccountId::from(BOB), existential_deposit()),
		])
		.build()
		.execute_with(|| {
			// EVM transfer that zeroes ALICE
			assert_ok!(RuntimeCall::EVM(pallet_evm::Call::<Runtime>::call {
				source: H160::from(ALICE),
				target: H160::from(BOB),
				input: Vec::new(),
				value: (1 * UNIT).into(),
				gas_limit: 21_777u64,
				max_fee_per_gas: U256::from(BASE_FEE_GENISIS),
				max_priority_fee_per_gas: None,
				nonce: Some(U256::from(0)),
				access_list: Vec::new(),
			})
			.dispatch(<Runtime as frame_system::Config>::RuntimeOrigin::root()));
			// ALICE is refunded
			assert_eq!(
				Balances::free_balance(AccountId::from(ALICE)),
				777 * BASE_FEE_GENISIS + existential_deposit(),
			);
		});
}

#[test]
fn author_does_not_receive_priority_fee() {
	ExtBuilder::default()
		.with_balances(vec![(
			AccountId::from(BOB),
			(1 * UNIT) + (21_000 * (500 * GIGAWEI)),
		)])
		.build()
		.execute_with(|| {
			// Some block author as seen by pallet-evm.
			let author = AccountId::from(<pallet_evm::Pallet<Runtime>>::find_author());
			// Currently the default impl of the evm uses `deposit_into_existing`.
			// If we were to use this implementation, and for an author to receive eventual tips,
			// the account needs to be somehow initialized, otherwise the deposit would fail.
			Balances::make_free_balance_be(&author, 100 * UNIT);

			// EVM transfer.
			assert_ok!(RuntimeCall::EVM(pallet_evm::Call::<Runtime>::call {
				source: H160::from(BOB),
				target: H160::from(ALICE),
				input: Vec::new(),
				value: (1 * UNIT).into(),
				gas_limit: 21_000u64,
				max_fee_per_gas: U256::from(300 * GIGAWEI),
				max_priority_fee_per_gas: Some(U256::from(200 * GIGAWEI)),
				nonce: Some(U256::from(0)),
				access_list: Vec::new(),
			})
			.dispatch(<Runtime as frame_system::Config>::RuntimeOrigin::root()));
			// Author free balance didn't change.
			assert_eq!(Balances::free_balance(author), 100 * UNIT,);
		});
}

#[test]
fn total_issuance_after_evm_transaction_with_priority_fee() {
	ExtBuilder::default()
		.with_balances(vec![
			(
				AccountId::from(BOB),
				(1 * UNIT) + (21_000 * (2 * BASE_FEE_GENISIS) + existential_deposit()),
			),
			(AccountId::from(ALICE), existential_deposit()),
			(
				<pallet_treasury::TreasuryAccountId<Runtime> as sp_core::TypedGet>::get(),
				existential_deposit(),
			),
		])
		.build()
		.execute_with(|| {
			let issuance_before = <Runtime as pallet_evm::Config>::Currency::total_issuance();
			// EVM transfer.
			assert_ok!(RuntimeCall::EVM(pallet_evm::Call::<Runtime>::call {
				source: H160::from(BOB),
				target: H160::from(ALICE),
				input: Vec::new(),
				value: (1 * UNIT).into(),
				gas_limit: 21_000u64,
				max_fee_per_gas: U256::from(2 * BASE_FEE_GENISIS),
				max_priority_fee_per_gas: Some(U256::from(BASE_FEE_GENISIS)),
				nonce: Some(U256::from(0)),
				access_list: Vec::new(),
			})
			.dispatch(<Runtime as frame_system::Config>::RuntimeOrigin::root()));

			let issuance_after = <Runtime as pallet_evm::Config>::Currency::total_issuance();
			// Fee is 1 * base_fee + tip.
			let fee = ((2 * BASE_FEE_GENISIS) * 21_000) as f64;
			// 80% was burned.
			let expected_burn = (fee * 0.8) as u128;
			assert_eq!(issuance_after, issuance_before - expected_burn,);
			// 20% was sent to treasury.
			let expected_treasury = (fee * 0.2) as u128;
			assert_eq!(moonbase_runtime::Treasury::pot(), expected_treasury);
		});
}

#[test]
fn total_issuance_after_evm_transaction_without_priority_fee() {
	use fp_evm::FeeCalculator;
	ExtBuilder::default()
		.with_balances(vec![
			(
				AccountId::from(BOB),
				(1 * UNIT) + (21_000 * (2 * BASE_FEE_GENISIS)),
			),
			(AccountId::from(ALICE), existential_deposit()),
			(
				<pallet_treasury::TreasuryAccountId<Runtime> as sp_core::TypedGet>::get(),
				existential_deposit(),
			),
		])
		.build()
		.execute_with(|| {
			let issuance_before = <Runtime as pallet_evm::Config>::Currency::total_issuance();
			// EVM transfer.
			assert_ok!(RuntimeCall::EVM(pallet_evm::Call::<Runtime>::call {
				source: H160::from(BOB),
				target: H160::from(ALICE),
				input: Vec::new(),
				value: (1 * UNIT).into(),
				gas_limit: 21_000u64,
				max_fee_per_gas: U256::from(BASE_FEE_GENISIS),
				max_priority_fee_per_gas: None,
				nonce: Some(U256::from(0)),
				access_list: Vec::new(),
			})
			.dispatch(<Runtime as frame_system::Config>::RuntimeOrigin::root()));

			let issuance_after = <Runtime as pallet_evm::Config>::Currency::total_issuance();
			// Fee is 1 GWEI base fee.
			let base_fee = TransactionPaymentAsGasPrice::min_gas_price().0;
			assert_eq!(base_fee.as_u128(), BASE_FEE_GENISIS); // hint in case following asserts fail
			let fee = (base_fee.as_u128() * 21_000u128) as f64;
			// 80% was burned.
			let expected_burn = (fee * 0.8) as u128;
			assert_eq!(issuance_after, issuance_before - expected_burn,);
			// 20% was sent to treasury.
			let expected_treasury = (fee * 0.2) as u128;
			assert_eq!(moonbase_runtime::Treasury::pot(), expected_treasury);
		});
}

#[test]
fn root_can_change_default_xcm_vers() {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * UNIT),
			(AccountId::from(BOB), 1_000 * UNIT),
		])
		.with_xcm_assets(vec![XcmAssetInitialization {
			asset_id: 1,
			xcm_location: xcm::v4::Location::parent(),
			name: "RelayToken",
			symbol: "Relay",
			decimals: 12,
			balances: vec![(AccountId::from(ALICE), 1_000_000_000_000_000)],
		}])
		.build()
		.execute_with(|| {
			let dest = Location {
				parents: 1,
				interior: [AccountId32 {
					network: None,
					id: [1u8; 32],
				}]
				.into(),
			};
			let source_id: moonbase_runtime::AssetId = 1;
			// Default XCM version is not set yet, so xtokens should fail because it does not
			// know with which version to send
			assert_noop!(
				XTokens::transfer(
					origin_of(AccountId::from(ALICE)),
					moonbase_runtime::xcm_config::CurrencyId::ForeignAsset(source_id),
					100_000_000_000_000,
					Box::new(xcm::VersionedLocation::V4(dest.clone())),
					WeightLimit::Unlimited
				),
				orml_xtokens::Error::<Runtime>::XcmExecutionFailed
			);

			// Root sets the defaultXcm
			assert_ok!(PolkadotXcm::force_default_xcm_version(
				root_origin(),
				Some(4)
			));

			// Now transferring does not fail
			assert_ok!(XTokens::transfer(
				origin_of(AccountId::from(ALICE)),
				moonbase_runtime::xcm_config::CurrencyId::ForeignAsset(source_id),
				100_000_000_000_000,
				Box::new(xcm::VersionedLocation::V4(dest)),
				WeightLimit::Unlimited
			));
		})
}

#[test]
fn transactor_cannot_use_more_than_max_weight() {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * UNIT),
			(AccountId::from(BOB), 1_000 * UNIT),
		])
		.with_xcm_assets(vec![XcmAssetInitialization {
			asset_id: 1,
			xcm_location: xcm::v4::Location::parent(),
			name: "RelayToken",
			symbol: "Relay",
			decimals: 12,
			balances: vec![(AccountId::from(ALICE), 1_000_000_000_000_000)],
		}])
		.build()
		.execute_with(|| {
			let source_id: moonbase_runtime::AssetId = 1;
			assert_ok!(XcmTransactor::register(
				root_origin(),
				AccountId::from(ALICE),
				0,
			));

			// Root can set transact info
			assert_ok!(XcmTransactor::set_transact_info(
				root_origin(),
				Box::new(xcm::VersionedLocation::V4(Location::parent())),
				// Relay charges 1000 for every instruction, and we have 3, so 3000
				3000.into(),
				20000.into(),
				None
			));
			// Root can set transact info
			assert_ok!(XcmTransactor::set_fee_per_second(
				root_origin(),
				Box::new(xcm::VersionedLocation::V4(Location::parent())),
				1,
			));

			assert_noop!(
				XcmTransactor::transact_through_derivative(
					origin_of(AccountId::from(ALICE)),
					moonbase_runtime::xcm_config::Transactors::Relay,
					0,
					CurrencyPayment {
						currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::V4(
							Location::parent()
						))),
						fee_amount: None
					},
					vec![],
					// 20000 is the max
					TransactWeights {
						transact_required_weight_at_most: 17001.into(),
						overall_weight: None
					},
					false
				),
				pallet_xcm_transactor::Error::<Runtime>::MaxWeightTransactReached
			);
			assert_noop!(
				XcmTransactor::transact_through_derivative(
					origin_of(AccountId::from(ALICE)),
					moonbase_runtime::xcm_config::Transactors::Relay,
					0,
					CurrencyPayment {
						currency: Currency::AsCurrencyId(
							moonbase_runtime::xcm_config::CurrencyId::ForeignAsset(source_id)
						),
						fee_amount: None
					},
					vec![],
					// 20000 is the max
					TransactWeights {
						transact_required_weight_at_most: 17001.into(),
						overall_weight: None
					},
					false
				),
				pallet_xcm_transactor::Error::<Runtime>::MaxWeightTransactReached
			);
		})
}

#[test]
fn root_can_use_hrmp_manage() {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * UNIT),
			(AccountId::from(BOB), 1_000 * UNIT),
		])
		.build()
		.execute_with(|| {
			// It fails sending, because the router does not work in test mode
			// But all rest checks pass
			assert_noop!(
				XcmTransactor::hrmp_manage(
					root_origin(),
					HrmpOperation::Accept {
						para_id: 2000u32.into()
					},
					CurrencyPayment {
						currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::V4(
							Location::parent()
						))),
						fee_amount: Some(10000)
					},
					// 20000 is the max
					TransactWeights {
						transact_required_weight_at_most: 17001.into(),
						overall_weight: Some(Limited(20000.into()))
					}
				),
				pallet_xcm_transactor::Error::<Runtime>::ErrorValidating
			);
		})
}

#[test]
fn transact_through_signed_precompile_works_v1() {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * UNIT),
			(AccountId::from(BOB), 1_000 * UNIT),
		])
		.with_safe_xcm_version(2)
		.build()
		.execute_with(|| {
			// Destination
			let dest = Location::parent();

			let fee_payer_asset = Location::parent();

			let bytes = vec![1u8, 2u8, 3u8];

			let xcm_transactor_v1_precompile_address = H160::from_low_u64_be(2054);

			// Root can set transact info
			assert_ok!(XcmTransactor::set_transact_info(
				root_origin(),
				Box::new(xcm::VersionedLocation::V4(Location::parent())),
				// Relay charges 1000 for every instruction, and we have 3, so 3000
				3000.into(),
				Weight::from_parts(200_000, (xcm_primitives::DEFAULT_PROOF_SIZE) + 4000),
				Some(4000.into())
			));
			// Root can set transact info
			assert_ok!(XcmTransactor::set_fee_per_second(
				root_origin(),
				Box::new(xcm::VersionedLocation::V4(Location::parent())),
				1,
			));

			Precompiles::new()
				.prepare_test(
					ALICE,
					xcm_transactor_v1_precompile_address,
					XcmTransactorV1PCall::transact_through_signed_multilocation {
						dest,
						fee_asset: fee_payer_asset,
						weight: 15000,
						call: bytes.into(),
					},
				)
				.expect_cost(18748)
				.expect_no_logs()
				.execute_returns(());
		});
}

#[test]
fn transact_through_signed_precompile_works_v2() {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * UNIT),
			(AccountId::from(BOB), 1_000 * UNIT),
		])
		.with_safe_xcm_version(2)
		.build()
		.execute_with(|| {
			// Destination
			let dest = Location::parent();

			let fee_payer_asset = Location::parent();

			let bytes = vec![1u8, 2u8, 3u8];

			let total_weight = 1_000_000_000u64;

			let xcm_transactor_v2_precompile_address = H160::from_low_u64_be(2061);

			Precompiles::new()
				.prepare_test(
					ALICE,
					xcm_transactor_v2_precompile_address,
					XcmTransactorV2PCall::transact_through_signed_multilocation {
						dest,
						fee_asset: fee_payer_asset,
						weight: 4_000_000,
						call: bytes.into(),
						fee_amount: u128::from(total_weight).into(),
						overall_weight: total_weight,
					},
				)
				.expect_cost(18748)
				.expect_no_logs()
				.execute_returns(());
		});
}

#[test]
fn transact_through_signed_cannot_send_to_local_chain() {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * UNIT),
			(AccountId::from(BOB), 1_000 * UNIT),
		])
		.with_safe_xcm_version(2)
		.build()
		.execute_with(|| {
			// Destination
			let dest = Location::here();

			let fee_payer_asset = Location::parent();

			let bytes = vec![1u8, 2u8, 3u8];

			let total_weight = 1_000_000_000u64;

			let xcm_transactor_v2_precompile_address = H160::from_low_u64_be(2061);

			Precompiles::new()
				.prepare_test(
					ALICE,
					xcm_transactor_v2_precompile_address,
					XcmTransactorV2PCall::transact_through_signed_multilocation {
						dest,
						fee_asset: fee_payer_asset,
						weight: 4_000_000,
						call: bytes.into(),
						fee_amount: u128::from(total_weight).into(),
						overall_weight: total_weight,
					},
				)
				.execute_reverts(|output| {
					from_utf8(&output)
						.unwrap()
						.contains("Dispatched call failed with error:")
						&& from_utf8(&output).unwrap().contains("ErrorValidating")
				});
		});
}

// Test to ensure we can use either in crowdloan rewards without worrying for migrations
#[test]
fn account_id_32_encodes_like_32_byte_u8_slice() {
	let account_as_account_id_32: sp_runtime::AccountId32 = [1u8; 32].into();
	let account_as_slice = [1u8; 32];
	assert_eq!(account_as_account_id_32.encode(), account_as_slice.encode());
}

#[test]
fn author_mapping_precompile_associate_update_and_clear() {
	ExtBuilder::default()
		.with_balances(vec![(AccountId::from(ALICE), 1_000 * UNIT)])
		.build()
		.execute_with(|| {
			let author_mapping_precompile_address = H160::from_low_u64_be(2055);
			let first_nimbus_id: NimbusId =
				sp_core::sr25519::Public::unchecked_from([1u8; 32]).into();
			let first_vrf_id: session_keys_primitives::VrfId =
				sp_core::sr25519::Public::unchecked_from([1u8; 32]).into();
			let second_nimbus_id: NimbusId =
				sp_core::sr25519::Public::unchecked_from([2u8; 32]).into();
			let second_vrf_id: session_keys_primitives::VrfId =
				sp_core::sr25519::Public::unchecked_from([2u8; 32]).into();

			// Associate it
			Precompiles::new()
				.prepare_test(
					ALICE,
					author_mapping_precompile_address,
					AuthorMappingPCall::add_association {
						nimbus_id: [1u8; 32].into(),
					},
				)
				.expect_cost(15119)
				.expect_no_logs()
				.execute_returns(());

			let expected_associate_event =
				RuntimeEvent::AuthorMapping(pallet_author_mapping::Event::KeysRegistered {
					nimbus_id: first_nimbus_id.clone(),
					account_id: AccountId::from(ALICE),
					keys: first_vrf_id.clone(),
				});
			assert_eq!(last_event(), expected_associate_event);

			// Update it
			Precompiles::new()
				.prepare_test(
					ALICE,
					author_mapping_precompile_address,
					AuthorMappingPCall::update_association {
						old_nimbus_id: [1u8; 32].into(),
						new_nimbus_id: [2u8; 32].into(),
					},
				)
				.expect_cost(14723)
				.expect_no_logs()
				.execute_returns(());

			let expected_update_event =
				RuntimeEvent::AuthorMapping(pallet_author_mapping::Event::KeysRotated {
					new_nimbus_id: second_nimbus_id.clone(),
					account_id: AccountId::from(ALICE),
					new_keys: second_vrf_id.clone(),
				});
			assert_eq!(last_event(), expected_update_event);

			// Clear it
			Precompiles::new()
				.prepare_test(
					ALICE,
					author_mapping_precompile_address,
					AuthorMappingPCall::clear_association {
						nimbus_id: [2u8; 32].into(),
					},
				)
				.expect_cost(15158)
				.expect_no_logs()
				.execute_returns(());

			let expected_clear_event =
				RuntimeEvent::AuthorMapping(pallet_author_mapping::Event::KeysRemoved {
					nimbus_id: second_nimbus_id,
					account_id: AccountId::from(ALICE),
					keys: second_vrf_id,
				});
			assert_eq!(last_event(), expected_clear_event);
		});
}

#[test]
fn author_mapping_register_and_set_keys() {
	ExtBuilder::default()
		.with_balances(vec![(AccountId::from(ALICE), 1_000 * UNIT)])
		.build()
		.execute_with(|| {
			let author_mapping_precompile_address = H160::from_low_u64_be(2055);
			let first_nimbus_id: NimbusId =
				sp_core::sr25519::Public::unchecked_from([1u8; 32]).into();
			let first_vrf_key: session_keys_primitives::VrfId =
				sp_core::sr25519::Public::unchecked_from([3u8; 32]).into();
			let second_nimbus_id: NimbusId =
				sp_core::sr25519::Public::unchecked_from([2u8; 32]).into();
			let second_vrf_key: session_keys_primitives::VrfId =
				sp_core::sr25519::Public::unchecked_from([4u8; 32]).into();

			// Associate it
			Precompiles::new()
				.prepare_test(
					ALICE,
					author_mapping_precompile_address,
					AuthorMappingPCall::set_keys {
						keys: solidity::encode_arguments((
							H256::from([1u8; 32]),
							H256::from([3u8; 32]),
						))
						.into(),
					},
				)
				.expect_cost(16233)
				.expect_no_logs()
				.execute_returns(());

			let expected_associate_event =
				RuntimeEvent::AuthorMapping(pallet_author_mapping::Event::KeysRegistered {
					nimbus_id: first_nimbus_id.clone(),
					account_id: AccountId::from(ALICE),
					keys: first_vrf_key.clone(),
				});
			assert_eq!(last_event(), expected_associate_event);

			// Update it
			Precompiles::new()
				.prepare_test(
					ALICE,
					author_mapping_precompile_address,
					AuthorMappingPCall::set_keys {
						keys: solidity::encode_arguments((
							H256::from([2u8; 32]),
							H256::from([4u8; 32]),
						))
						.into(),
					},
				)
				.expect_cost(16233)
				.expect_no_logs()
				.execute_returns(());

			let expected_update_event =
				RuntimeEvent::AuthorMapping(pallet_author_mapping::Event::KeysRotated {
					new_nimbus_id: second_nimbus_id.clone(),
					account_id: AccountId::from(ALICE),
					new_keys: second_vrf_key.clone(),
				});
			assert_eq!(last_event(), expected_update_event);
		});
}

#[test]
fn test_xcm_utils_ml_tp_account() {
	ExtBuilder::default().build().execute_with(|| {
		let xcm_utils_precompile_address = H160::from_low_u64_be(2060);
		let expected_address_parent: H160 =
			ParentIsPreset::<AccountId>::convert_location(&Location::parent())
				.unwrap()
				.into();

		Precompiles::new()
			.prepare_test(
				ALICE,
				xcm_utils_precompile_address,
				XcmUtilsPCall::multilocation_to_address {
					location: Location::parent(),
				},
			)
			.expect_cost(1000)
			.expect_no_logs()
			.execute_returns(Address(expected_address_parent));

		let parachain_2000_multilocation = Location::new(1, [Parachain(2000)]);
		let expected_address_parachain: H160 =
			SiblingParachainConvertsVia::<Sibling, AccountId>::convert_location(
				&parachain_2000_multilocation,
			)
			.unwrap()
			.into();

		Precompiles::new()
			.prepare_test(
				ALICE,
				xcm_utils_precompile_address,
				XcmUtilsPCall::multilocation_to_address {
					location: parachain_2000_multilocation,
				},
			)
			.expect_cost(1000)
			.expect_no_logs()
			.execute_returns(Address(expected_address_parachain));

		let alice_in_parachain_2000_multilocation = Location::new(
			1,
			[
				Parachain(2000),
				AccountKey20 {
					network: None,
					key: ALICE,
				},
			],
		);
		let expected_address_alice_in_parachain_2000 =
			xcm_builder::HashedDescription::<
				AccountId,
				xcm_builder::DescribeFamily<xcm_builder::DescribeAllTerminal>,
			>::convert_location(&alice_in_parachain_2000_multilocation)
			.unwrap()
			.into();

		Precompiles::new()
			.prepare_test(
				ALICE,
				xcm_utils_precompile_address,
				XcmUtilsPCall::multilocation_to_address {
					location: alice_in_parachain_2000_multilocation,
				},
			)
			.expect_cost(1000)
			.expect_no_logs()
			.execute_returns(Address(expected_address_alice_in_parachain_2000));
	});
}

#[test]
fn test_xcm_utils_weight_message() {
	ExtBuilder::default().build().execute_with(|| {
		let xcm_utils_precompile_address = H160::from_low_u64_be(2060);
		let expected_weight =
			XcmWeight::<moonbase_runtime::Runtime, RuntimeCall>::clear_origin().ref_time();

		let message: Vec<u8> = xcm::VersionedXcm::<()>::V4(Xcm(vec![ClearOrigin])).encode();

		let input = XcmUtilsPCall::weight_message {
			message: message.into(),
		};

		Precompiles::new()
			.prepare_test(ALICE, xcm_utils_precompile_address, input)
			.expect_cost(0)
			.expect_no_logs()
			.execute_returns(expected_weight);
	});
}

#[test]
fn test_xcm_utils_get_units_per_second() {
	ExtBuilder::default().build().execute_with(|| {
		let xcm_utils_precompile_address = H160::from_low_u64_be(2060);
		let location = SelfReserve::get();

		let input = XcmUtilsPCall::get_units_per_second { location };

		let expected_units =
			WEIGHT_REF_TIME_PER_SECOND as u128 * moonbase_runtime::currency::WEIGHT_FEE;

		Precompiles::new()
			.prepare_test(ALICE, xcm_utils_precompile_address, input)
			.expect_cost(1000)
			.expect_no_logs()
			.execute_returns(expected_units);
	});
}

#[test]
fn precompile_existence() {
	ExtBuilder::default().build().execute_with(|| {
		let precompiles = Precompiles::new();
		let precompile_addresses: std::collections::BTreeSet<_> = vec![
			1, 2, 3, 4, 5, 6, 7, 8, 9, 256, 1024, 1025, 1026, 1027, 2048, 2049, 2050, 2051, 2052,
			2053, 2054, 2055, 2056, 2057, 2058, 2059, 2060, 2061, 2062, 2063, 2064, 2065, 2066,
			2067, 2068, 2069, 2070, 2071, 2072, 2073, 2074, 2075,
		]
		.into_iter()
		.map(H160::from_low_u64_be)
		.collect();

		for i in 0..3000 {
			let address = H160::from_low_u64_be(i);

			if precompile_addresses.contains(&address) {
				assert!(
					is_precompile_or_fail::<Runtime>(address, 100_000u64).expect("to be ok"),
					"is_precompile({}) should return true",
					i
				);

				assert!(
					precompiles
						.execute(&mut MockHandle::new(
							address,
							Context {
								address,
								caller: H160::zero(),
								apparent_value: U256::zero()
							}
						),)
						.is_some(),
					"execute({},..) should return Some(_)",
					i
				);
			} else {
				assert!(
					!is_precompile_or_fail::<Runtime>(address, 100_000u64).expect("to be ok"),
					"is_precompile({}) should return false",
					i
				);

				assert!(
					precompiles
						.execute(&mut MockHandle::new(
							address,
							Context {
								address,
								caller: H160::zero(),
								apparent_value: U256::zero()
							}
						),)
						.is_none(),
					"execute({},..) should return None",
					i
				);
			}
		}
	});
}

#[test]
fn removed_precompiles() {
	ExtBuilder::default().build().execute_with(|| {
		let precompiles = Precompiles::new();
		let removed_precompiles = [1025, 2051, 2062, 2063];

		for i in 1..3000 {
			let address = H160::from_low_u64_be(i);

			if !is_precompile_or_fail::<Runtime>(address, 100_000u64).expect("to be ok") {
				continue;
			}

			if !removed_precompiles.contains(&i) {
				assert!(
					match precompiles.is_active_precompile(address, 100_000u64) {
						IsPrecompileResult::Answer { is_precompile, .. } => is_precompile,
						_ => false,
					},
					"{i} should be an active precompile"
				);
				continue;
			}

			assert!(
				!match precompiles.is_active_precompile(address, 100_000u64) {
					IsPrecompileResult::Answer { is_precompile, .. } => is_precompile,
					_ => false,
				},
				"{i} shouldn't be an active precompile"
			);

			precompiles
				.prepare_test(Alice, address, [])
				.execute_reverts(|out| out == b"Removed precompile");
		}
	})
}

#[test]
fn substrate_based_fees_zero_txn_costs_only_base_extrinsic() {
	use frame_support::dispatch::{DispatchInfo, Pays};
	use moonbase_runtime::{currency, EXTRINSIC_BASE_WEIGHT};

	ExtBuilder::default().build().execute_with(|| {
		let size_bytes = 0;
		let tip = 0;
		let dispatch_info = DispatchInfo {
			weight: Weight::zero(),
			class: DispatchClass::Normal,
			pays_fee: Pays::Yes,
		};

		assert_eq!(
			TransactionPayment::compute_fee(size_bytes, &dispatch_info, tip),
			EXTRINSIC_BASE_WEIGHT.ref_time() as u128 * currency::WEIGHT_FEE,
		);
	});
}

#[test]
fn deal_with_fees_handles_tip() {
	use frame_support::traits::OnUnbalanced;
	use moonbase_runtime::{DealWithFees, Treasury};

	ExtBuilder::default().build().execute_with(|| {
		// This test checks the functionality of the `DealWithFees` trait implementation in the runtime.
		// It simulates a scenario where a fee and a tip are issued to an account and ensures that the
		// treasury receives the correct amount (20% of the total), and the rest is burned (80%).
		//
		// The test follows these steps:
		// 1. It issues a fee of 100 and a tip of 1000.
		// 2. It checks the total supply before the fee and tip are dealt with, which should be 1_100.
		// 3. It checks that the treasury's balance is initially 0.
		// 4. It calls `DealWithFees::on_unbalanceds` with the fee and tip.
		// 5. It checks that the treasury's balance is now 220 (20% of the fee and tip).
		// 6. It checks that the total supply has decreased by 880 (80% of the fee and tip), indicating
		//    that this amount was burned.
		let fee = <pallet_balances::Pallet<Runtime> as frame_support::traits::fungible::Balanced<
			AccountId,
		>>::issue(100);
		let tip = <pallet_balances::Pallet<Runtime> as frame_support::traits::fungible::Balanced<
			AccountId,
		>>::issue(1000);

		let total_supply_before = Balances::total_issuance();
		assert_eq!(total_supply_before, 1_100);
		assert_eq!(Balances::free_balance(&Treasury::account_id()), 0);

		DealWithFees::on_unbalanceds(vec![fee, tip].into_iter());

		// treasury should have received 20%
		assert_eq!(Balances::free_balance(&Treasury::account_id()), 220);

		// verify 80% burned
		let total_supply_after = Balances::total_issuance();
		assert_eq!(total_supply_before - total_supply_after, 880);
	});
}

#[test]
fn evm_revert_substrate_events() {
	ExtBuilder::default()
		.with_balances(vec![(AccountId::from(ALICE), 1_000 * UNIT)])
		.build()
		.execute_with(|| {
			let batch_precompile_address = H160::from_low_u64_be(2056);

			// Batch a transfer followed by an invalid call to batch.
			// Thus BatchAll will revert the transfer.
			assert_ok!(RuntimeCall::EVM(pallet_evm::Call::call {
				source: ALICE.into(),
				target: batch_precompile_address,
				input: BatchPCall::batch_all {
					to: vec![Address(BOB.into()), Address(batch_precompile_address)].into(),
					value: vec![U256::from(1 * UNIT), U256::zero()].into(),
					call_data: vec![].into(),
					gas_limit: vec![].into()
				}
				.into(),
				value: U256::zero(), // No value sent in EVM
				gas_limit: 500_000,
				max_fee_per_gas: U256::from(BASE_FEE_GENISIS),
				max_priority_fee_per_gas: None,
				nonce: Some(U256::from(0)),
				access_list: Vec::new(),
			})
			.dispatch(<Runtime as frame_system::Config>::RuntimeOrigin::root()));

			let transfer_count = System::events()
				.iter()
				.filter(|r| match r.event {
					RuntimeEvent::Balances(pallet_balances::Event::Transfer { .. }) => true,
					_ => false,
				})
				.count();

			assert_eq!(transfer_count, 0, "there should be no transfer event");
		});
}

#[test]
fn evm_success_keeps_substrate_events() {
	ExtBuilder::default()
		.with_balances(vec![(AccountId::from(ALICE), 1_000 * UNIT)])
		.build()
		.execute_with(|| {
			let batch_precompile_address = H160::from_low_u64_be(2056);

			assert_ok!(RuntimeCall::EVM(pallet_evm::Call::call {
				source: ALICE.into(),
				target: batch_precompile_address,
				input: BatchPCall::batch_all {
					to: vec![Address(BOB.into())].into(),
					value: vec![U256::from(1 * UNIT)].into(),
					call_data: vec![].into(),
					gas_limit: vec![].into()
				}
				.into(),
				value: U256::zero(), // No value sent in EVM
				gas_limit: 500_000,
				max_fee_per_gas: U256::from(BASE_FEE_GENISIS),
				max_priority_fee_per_gas: None,
				nonce: Some(U256::from(0)),
				access_list: Vec::new(),
			})
			.dispatch(<Runtime as frame_system::Config>::RuntimeOrigin::root()));

			let transfer_count = System::events()
				.iter()
				.filter(|r| match r.event {
					RuntimeEvent::Balances(pallet_balances::Event::Transfer { .. }) => true,
					_ => false,
				})
				.count();

			assert_eq!(transfer_count, 1, "there should be 1 transfer event");
		});
}

#[test]
fn validate_transaction_fails_on_filtered_call() {
	use sp_runtime::transaction_validity::{
		InvalidTransaction, TransactionSource, TransactionValidityError,
	};
	use sp_transaction_pool::runtime_api::runtime_decl_for_tagged_transaction_queue::TaggedTransactionQueueV3; // editorconfig-checker-disable-line

	ExtBuilder::default().build().execute_with(|| {
		let xt = UncheckedExtrinsic::new_unsigned(
			pallet_evm::Call::<Runtime>::call {
				source: Default::default(),
				target: H160::default(),
				input: Vec::new(),
				value: Default::default(),
				gas_limit: Default::default(),
				max_fee_per_gas: Default::default(),
				max_priority_fee_per_gas: Default::default(),
				nonce: Default::default(),
				access_list: Default::default(),
			}
			.into(),
		);

		assert_eq!(
			Runtime::validate_transaction(TransactionSource::External, xt, Default::default(),),
			Err(TransactionValidityError::Invalid(InvalidTransaction::Call)),
		);
	});
}

#[cfg(test)]
mod fee_tests {
	use super::*;
	use fp_evm::FeeCalculator;
	use frame_support::{
		traits::{ConstU128, OnFinalize},
		weights::{ConstantMultiplier, WeightToFee},
	};
	use moonbase_runtime::{
		currency, BlockWeights, FastAdjustingFeeUpdate, LengthToFee, MinimumMultiplier,
		TargetBlockFullness, TransactionPaymentAsGasPrice, NORMAL_WEIGHT, WEIGHT_PER_GAS,
	};
	use sp_runtime::{BuildStorage, FixedPointNumber, Perbill};

	fn run_with_system_weight<F>(w: Weight, mut assertions: F)
	where
		F: FnMut() -> (),
	{
		let mut t: sp_io::TestExternalities = frame_system::GenesisConfig::<Runtime>::default()
			.build_storage()
			.unwrap()
			.into();
		t.execute_with(|| {
			System::set_block_consumed_resources(w, 0);
			assertions()
		});
	}

	#[test]
	fn test_multiplier_can_grow_from_zero() {
		let minimum_multiplier = MinimumMultiplier::get();
		let target = TargetBlockFullness::get()
			* BlockWeights::get()
				.get(DispatchClass::Normal)
				.max_total
				.unwrap();
		// if the min is too small, then this will not change, and we are doomed forever.
		// the weight is 1/100th bigger than target.
		run_with_system_weight(target * 101 / 100, || {
			let next = FastAdjustingFeeUpdate::<Runtime>::convert(minimum_multiplier);
			assert!(
				next > minimum_multiplier,
				"{:?} !>= {:?}",
				next,
				minimum_multiplier
			);
		})
	}

	#[test]
	fn test_fee_calculation() {
		let base_extrinsic = BlockWeights::get()
			.get(DispatchClass::Normal)
			.base_extrinsic;
		let multiplier = sp_runtime::FixedU128::from_float(0.999000000000000000);
		let extrinsic_len = 100u32;
		let extrinsic_weight = 5_000u64;
		let tip = 42u128;
		type WeightToFeeImpl =
			ConstantMultiplier<u128, ConstU128<{ moonbase_runtime::currency::WEIGHT_FEE }>>;
		type LengthToFeeImpl = LengthToFee;

		// base_fee + (multiplier * extrinsic_weight_fee) + extrinsic_length_fee + tip
		let expected_fee =
			WeightToFeeImpl::weight_to_fee(&base_extrinsic)
				+ multiplier.saturating_mul_int(WeightToFeeImpl::weight_to_fee(
					&Weight::from_parts(extrinsic_weight, 1),
				)) + LengthToFeeImpl::weight_to_fee(&Weight::from_parts(extrinsic_len as u64, 1))
				+ tip;

		let mut t: sp_io::TestExternalities = frame_system::GenesisConfig::<Runtime>::default()
			.build_storage()
			.unwrap()
			.into();
		t.execute_with(|| {
			pallet_transaction_payment::NextFeeMultiplier::<Runtime>::set(multiplier);
			let actual_fee = TransactionPayment::compute_fee(
				extrinsic_len,
				&frame_support::dispatch::DispatchInfo {
					class: DispatchClass::Normal,
					pays_fee: frame_support::dispatch::Pays::Yes,
					weight: Weight::from_parts(extrinsic_weight, 1),
				},
				tip,
			);

			assert_eq!(
				expected_fee,
				actual_fee,
				"The actual fee did not match the expected fee, diff {}",
				actual_fee - expected_fee
			);
		});
	}

	#[test]
	fn test_min_gas_price_is_deterministic() {
		let mut t: sp_io::TestExternalities = frame_system::GenesisConfig::<Runtime>::default()
			.build_storage()
			.unwrap()
			.into();
		t.execute_with(|| {
			let multiplier = sp_runtime::FixedU128::from_u32(1);
			pallet_transaction_payment::NextFeeMultiplier::<Runtime>::set(multiplier);
			let actual = TransactionPaymentAsGasPrice::min_gas_price().0;
			let expected: U256 = multiplier
				.saturating_mul_int(currency::WEIGHT_FEE.saturating_mul(WEIGHT_PER_GAS as u128))
				.into();

			assert_eq!(expected, actual);
		});
	}

	#[test]
	fn test_min_gas_price_has_no_precision_loss_from_saturating_mul_int() {
		let mut t: sp_io::TestExternalities = frame_system::GenesisConfig::<Runtime>::default()
			.build_storage()
			.unwrap()
			.into();
		t.execute_with(|| {
			let multiplier_1 = sp_runtime::FixedU128::from_float(0.999593900000000000);
			let multiplier_2 = sp_runtime::FixedU128::from_float(0.999593200000000000);

			pallet_transaction_payment::NextFeeMultiplier::<Runtime>::set(multiplier_1);
			let a = TransactionPaymentAsGasPrice::min_gas_price();
			pallet_transaction_payment::NextFeeMultiplier::<Runtime>::set(multiplier_2);
			let b = TransactionPaymentAsGasPrice::min_gas_price();

			assert_ne!(
				a, b,
				"both gas prices were equal, unexpected precision loss incurred"
			);
		});
	}

	#[test]
	fn test_fee_scenarios() {
		use sp_runtime::FixedU128;
		let mut t: sp_io::TestExternalities = frame_system::GenesisConfig::<Runtime>::default()
			.build_storage()
			.unwrap()
			.into();
		t.execute_with(|| {
			let weight_fee_per_gas = currency::WEIGHT_FEE.saturating_mul(WEIGHT_PER_GAS as u128);
			let sim = |start_gas_price: u128, fullness: Perbill, num_blocks: u64| -> U256 {
				let start_multiplier =
					FixedU128::from_rational(start_gas_price, weight_fee_per_gas);
				pallet_transaction_payment::NextFeeMultiplier::<Runtime>::set(start_multiplier);

				let block_weight = NORMAL_WEIGHT * fullness;

				for i in 0..num_blocks {
					System::set_block_number(i as u32);
					System::set_block_consumed_resources(block_weight, 0);
					TransactionPayment::on_finalize(i as u32);
				}

				TransactionPaymentAsGasPrice::min_gas_price().0
			};

			// The expected values are the ones observed during test execution,
			// they are expected to change when parameters that influence
			// the fee calculation are changed, and should be updated accordingly.
			// If a test fails when nothing specific to fees has changed,
			// it may indicate an unexpected collateral effect and should be investigated

			assert_eq!(
				sim(1_000_000_000, Perbill::from_percent(0), 1),
				U256::from(998_600_980),
			);
			assert_eq!(
				sim(1_000_000_000, Perbill::from_percent(25), 1),
				U256::from(999_600_080),
			);
			assert_eq!(
				sim(1_000_000_000, Perbill::from_percent(50), 1),
				U256::from(1_000_600_180),
			);
			assert_eq!(
				sim(1_000_000_000, Perbill::from_percent(100), 1),
				U256::from(1_002_603_380),
			);

			// 1 "real" hour (at 6-second blocks)
			assert_eq!(
				sim(1_000_000_000, Perbill::from_percent(0), 600),
				U256::from(431_710_642),
			);
			assert_eq!(
				sim(1_000_000_000, Perbill::from_percent(25), 600),
				U256::from(786_627_866),
			);
			assert_eq!(
				sim(1_000_000_000, Perbill::from_percent(50), 600),
				U256::from(1_433_329_383u128),
			);
			assert_eq!(
				sim(1_000_000_000, Perbill::from_percent(100), 600),
				U256::from(4_758_812_897u128),
			);

			// 1 "real" day (at 6-second blocks)
			assert_eq!(
				sim(1_000_000_000, Perbill::from_percent(0), 14400),
				U256::from(125_000_000), // lower bound enforced
			);
			assert_eq!(
				sim(1_000_000_000, Perbill::from_percent(25), 14400),
				U256::from(125_000_000),
			);
			assert_eq!(
				sim(1_000_000_000, Perbill::from_percent(50), 14400),
				U256::from(5_653_326_895_069u128),
			);
			assert_eq!(
				sim(1_000_000_000, Perbill::from_percent(100), 14400),
				U256::from(125_000_000_000_000u128), // upper bound enforced
			);
		});
	}
}
