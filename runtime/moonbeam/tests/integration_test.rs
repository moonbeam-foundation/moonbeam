// Copyright 2019-2025 PureStake Inc.
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

//! Moonbeam Runtime Integration Tests

#![cfg(test)]

mod common;

use common::*;

use fp_evm::{Context, IsPrecompileResult};
use frame_support::{
	assert_noop, assert_ok,
	dispatch::DispatchClass,
	traits::{
		Contains, Currency as CurrencyT, EnsureOrigin, OnInitialize, PalletInfo, StorageInfo,
		StorageInfoTrait,
	},
	weights::{constants::WEIGHT_REF_TIME_PER_SECOND, Weight},
	StorageHasher, Twox128,
};
use moonbeam_runtime::currency::{GIGAWEI, WEI};
use moonbeam_runtime::runtime_params::dynamic_params;
use moonbeam_runtime::xcm_config::{AssetHubLocation, XcmExecutor};
use moonbeam_runtime::{
	currency::GLMR,
	moonbeam_xcm_weights,
	xcm_config::{CurrencyId, SelfReserve},
	AccountId, Balances, EvmForeignAssets, Executive, NormalFilter, OpenTechCommitteeCollective,
	ParachainStaking, PolkadotXcm, Precompiles, ProxyType, Runtime, RuntimeBlockWeights,
	RuntimeCall, RuntimeEvent, System, TransactionPayment, TransactionPaymentAsGasPrice, Treasury,
	TreasuryCouncilCollective, XcmTransactor, WEIGHT_PER_GAS,
};
use moonbeam_xcm_weights::XcmWeight;
use nimbus_primitives::NimbusId;
use pallet_evm::PrecompileSet;
use pallet_moonbeam_foreign_assets::AssetStatus;
use pallet_parachain_staking::InflationDistributionAccount;
use pallet_transaction_payment::Multiplier;
use pallet_xcm_transactor::{Currency, CurrencyPayment, TransactWeights};
use parity_scale_codec::Encode;
use polkadot_parachain::primitives::Sibling;
use precompile_utils::{
	precompile_set::{is_precompile_or_fail, IsActivePrecompile},
	prelude::*,
	testing::*,
};
use sp_core::{ByteArray, Get, H160, U256};
use sp_runtime::{
	traits::{Convert, Dispatchable},
	BuildStorage, DispatchError, ModuleError, Percent,
};
use std::str::from_utf8;
use xcm::{latest::prelude::*, VersionedAssetId, VersionedAssets, VersionedLocation, VersionedXcm};
use xcm_builder::{ParentIsPreset, SiblingParachainConvertsVia};
use xcm_executor::traits::{ConvertLocation, TransferType};
use xcm_primitives::split_location_into_chain_part_and_beneficiary;

type BatchPCall = pallet_evm_precompile_batch::BatchPrecompileCall<Runtime>;
type XcmUtilsPCall = pallet_evm_precompile_xcm_utils::XcmUtilsPrecompileCall<
	Runtime,
	moonbeam_runtime::xcm_config::XcmExecutorConfig,
>;
type XcmTransactorV2PCall =
	pallet_evm_precompile_xcm_transactor::v2::XcmTransactorPrecompileV2Call<Runtime>;

const BASE_FEE_GENESIS: u128 = 10000 * GIGAWEI;

fn currency_to_asset(currency_id: CurrencyId, amount: u128) -> Asset {
	Asset {
		id: AssetId(
			<moonbeam_runtime::Runtime as pallet_xcm_transactor::Config>::CurrencyIdToLocation::convert(
				currency_id,
			)
			.unwrap(),
		),
		fun: Fungibility::Fungible(amount),
	}
}
#[test]
fn xcmp_queue_controller_origin_is_root() {
	// important for the XcmExecutionManager impl of PauseExecution which uses root origin
	// to suspend/resume XCM execution in xcmp_queue::on_idle
	assert_ok!(
		<moonbeam_runtime::Runtime as cumulus_pallet_xcmp_queue::Config
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
			<moonbeam_runtime::Runtime as frame_system::Config>::PalletInfo::name::<P>(),
			Some(name)
		);
	}
	// TODO: use StorageInfoTrait once https://github.com/paritytech/substrate/pull/9246
	// is pulled in substrate deps.
	is_pallet_prefix::<moonbeam_runtime::System>("System");
	is_pallet_prefix::<moonbeam_runtime::Utility>("Utility");
	is_pallet_prefix::<moonbeam_runtime::ParachainSystem>("ParachainSystem");
	is_pallet_prefix::<moonbeam_runtime::TransactionPayment>("TransactionPayment");
	is_pallet_prefix::<moonbeam_runtime::ParachainInfo>("ParachainInfo");
	is_pallet_prefix::<moonbeam_runtime::EthereumChainId>("EthereumChainId");
	is_pallet_prefix::<moonbeam_runtime::EVM>("EVM");
	is_pallet_prefix::<moonbeam_runtime::Ethereum>("Ethereum");
	is_pallet_prefix::<moonbeam_runtime::ParachainStaking>("ParachainStaking");
	is_pallet_prefix::<moonbeam_runtime::Scheduler>("Scheduler");
	is_pallet_prefix::<moonbeam_runtime::OpenTechCommitteeCollective>(
		"OpenTechCommitteeCollective",
	);
	is_pallet_prefix::<moonbeam_runtime::Treasury>("Treasury");
	is_pallet_prefix::<moonbeam_runtime::AuthorInherent>("AuthorInherent");
	is_pallet_prefix::<moonbeam_runtime::AuthorFilter>("AuthorFilter");
	is_pallet_prefix::<moonbeam_runtime::CrowdloanRewards>("CrowdloanRewards");
	is_pallet_prefix::<moonbeam_runtime::AuthorMapping>("AuthorMapping");
	is_pallet_prefix::<moonbeam_runtime::MaintenanceMode>("MaintenanceMode");
	is_pallet_prefix::<moonbeam_runtime::Identity>("Identity");
	is_pallet_prefix::<moonbeam_runtime::XcmpQueue>("XcmpQueue");
	is_pallet_prefix::<moonbeam_runtime::CumulusXcm>("CumulusXcm");
	is_pallet_prefix::<moonbeam_runtime::PolkadotXcm>("PolkadotXcm");
	is_pallet_prefix::<moonbeam_runtime::XcmTransactor>("XcmTransactor");
	is_pallet_prefix::<moonbeam_runtime::ProxyGenesisCompanion>("ProxyGenesisCompanion");
	is_pallet_prefix::<moonbeam_runtime::MoonbeamOrbiters>("MoonbeamOrbiters");
	is_pallet_prefix::<moonbeam_runtime::TreasuryCouncilCollective>("TreasuryCouncilCollective");
	is_pallet_prefix::<moonbeam_runtime::MoonbeamLazyMigrations>("MoonbeamLazyMigrations");
	is_pallet_prefix::<moonbeam_runtime::RelayStorageRoots>("RelayStorageRoots");
	is_pallet_prefix::<moonbeam_runtime::BridgeKusamaGrandpa>("BridgeKusamaGrandpa");
	is_pallet_prefix::<moonbeam_runtime::BridgeKusamaParachains>("BridgeKusamaParachains");
	is_pallet_prefix::<moonbeam_runtime::BridgeKusamaMessages>("BridgeKusamaMessages");
	is_pallet_prefix::<moonbeam_runtime::BridgeXcmOverMoonriver>("BridgeXcmOverMoonriver");

	let prefix = |pallet_name, storage_name| {
		let mut res = [0u8; 32];
		res[0..16].copy_from_slice(&Twox128::hash(pallet_name));
		res[16..32].copy_from_slice(&Twox128::hash(storage_name));
		res.to_vec()
	};
	assert_eq!(
		<moonbeam_runtime::Timestamp as StorageInfoTrait>::storage_info(),
		vec![
			StorageInfo {
				pallet_name: b"Timestamp".to_vec(),
				storage_name: b"Now".to_vec(),
				prefix: prefix(b"Timestamp", b"Now"),
				max_values: Some(1),
				max_size: Some(8),
			},
			StorageInfo {
				pallet_name: b"Timestamp".to_vec(),
				storage_name: b"DidUpdate".to_vec(),
				prefix: prefix(b"Timestamp", b"DidUpdate"),
				max_values: Some(1),
				max_size: Some(1),
			}
		]
	);
	assert_eq!(
		<moonbeam_runtime::Balances as StorageInfoTrait>::storage_info(),
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
				max_size: Some(91),
			},
			StorageInfo {
				pallet_name: b"Balances".to_vec(),
				storage_name: b"Freezes".to_vec(),
				prefix: prefix(b"Balances", b"Freezes"),
				max_values: None,
				max_size: Some(73),
			},
		]
	);
	assert_eq!(
		<moonbeam_runtime::Proxy as StorageInfoTrait>::storage_info(),
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
		<moonbeam_runtime::MaintenanceMode as StorageInfoTrait>::storage_info(),
		vec![StorageInfo {
			pallet_name: b"MaintenanceMode".to_vec(),
			storage_name: b"MaintenanceMode".to_vec(),
			prefix: prefix(b"MaintenanceMode", b"MaintenanceMode"),
			max_values: Some(1),
			max_size: None,
		},]
	);
	assert_eq!(
		<moonbeam_runtime::RelayStorageRoots as StorageInfoTrait>::storage_info(),
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
		<moonbeam_runtime::TreasuryCouncilCollective as StorageInfoTrait>::storage_info()
	{
		assert_eq!(pallet_name, b"TreasuryCouncilCollective".to_vec());
	}

	for StorageInfo { pallet_name, .. } in
		<moonbeam_runtime::OpenTechCommitteeCollective as StorageInfoTrait>::storage_info()
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
	use moonbeam_runtime::{
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
			<moonbeam_runtime::Runtime as frame_system::Config>::PalletInfo::index::<P>(),
			Some(index)
		);
	}

	// System support
	is_pallet_index::<moonbeam_runtime::System>(0);
	is_pallet_index::<moonbeam_runtime::ParachainSystem>(1);
	is_pallet_index::<moonbeam_runtime::Timestamp>(3);
	is_pallet_index::<moonbeam_runtime::ParachainInfo>(4);
	// Monetary
	is_pallet_index::<moonbeam_runtime::Balances>(10);
	is_pallet_index::<moonbeam_runtime::TransactionPayment>(11);
	// Consensus support
	is_pallet_index::<moonbeam_runtime::ParachainStaking>(20);
	is_pallet_index::<moonbeam_runtime::AuthorInherent>(21);
	is_pallet_index::<moonbeam_runtime::AuthorFilter>(22);
	is_pallet_index::<moonbeam_runtime::AuthorMapping>(23);
	is_pallet_index::<moonbeam_runtime::MoonbeamOrbiters>(24);
	// Handy utilities
	is_pallet_index::<moonbeam_runtime::Utility>(30);
	is_pallet_index::<moonbeam_runtime::Proxy>(31);
	is_pallet_index::<moonbeam_runtime::MaintenanceMode>(32);
	is_pallet_index::<moonbeam_runtime::Identity>(33);
	is_pallet_index::<moonbeam_runtime::ProxyGenesisCompanion>(35);
	is_pallet_index::<moonbeam_runtime::MoonbeamLazyMigrations>(37);
	// Ethereum compatibility
	is_pallet_index::<moonbeam_runtime::EthereumChainId>(50);
	is_pallet_index::<moonbeam_runtime::EVM>(51);
	is_pallet_index::<moonbeam_runtime::Ethereum>(52);
	// Governance
	is_pallet_index::<moonbeam_runtime::Scheduler>(60);
	// is_pallet_index::<moonbeam_runtime::Democracy>(61); Removed
	// Council
	// is_pallet_index::<moonbeam_runtime::CouncilCollective>(70); Removed
	// is_pallet_index::<moonbeam_runtime::TechCommitteeCollective>(71); Removed
	is_pallet_index::<moonbeam_runtime::TreasuryCouncilCollective>(72);
	is_pallet_index::<moonbeam_runtime::OpenTechCommitteeCollective>(73);
	// Treasury
	is_pallet_index::<moonbeam_runtime::Treasury>(80);
	// Crowdloan
	is_pallet_index::<moonbeam_runtime::CrowdloanRewards>(90);
	// XCM Stuff
	is_pallet_index::<moonbeam_runtime::XcmpQueue>(100);
	is_pallet_index::<moonbeam_runtime::CumulusXcm>(101);
	is_pallet_index::<moonbeam_runtime::PolkadotXcm>(103);
	// is_pallet_index::<moonbeam_runtime::Assets>(104); Removed
	// is_pallet_index::<moonbeam_runtime::AssetManager>(105); Removed
	// is_pallet_index::<moonbeam_runtime::XTokens>(106); Removed
	is_pallet_index::<moonbeam_runtime::XcmTransactor>(107);
	is_pallet_index::<moonbeam_runtime::BridgeKusamaGrandpa>(130);
	is_pallet_index::<moonbeam_runtime::BridgeKusamaParachains>(131);
	is_pallet_index::<moonbeam_runtime::BridgeKusamaMessages>(132);
	is_pallet_index::<moonbeam_runtime::BridgeXcmOverMoonriver>(133);
}

#[test]
fn verify_reserved_indices() {
	let mut t: sp_io::TestExternalities = frame_system::GenesisConfig::<Runtime>::default()
		.build_storage()
		.unwrap()
		.into();

	t.execute_with(|| {
		use frame_metadata::*;
		let metadata = moonbeam_runtime::Runtime::metadata();
		let metadata = match metadata.1 {
			RuntimeMetadata::V14(metadata) => metadata,
			_ => panic!("metadata has been bumped, test needs to be updated"),
		};
		// 40: Sudo
		// 53: BaseFee
		// 108: pallet_assets::<Instance1>
		let reserved = vec![40, 53, 108];
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
	assert_eq!(moonbeam_runtime::ProxyType::Any as u8, 0);
	assert_eq!(moonbeam_runtime::ProxyType::NonTransfer as u8, 1);
	assert_eq!(moonbeam_runtime::ProxyType::Governance as u8, 2);
	assert_eq!(moonbeam_runtime::ProxyType::Staking as u8, 3);
	assert_eq!(moonbeam_runtime::ProxyType::CancelProxy as u8, 4);
	assert_eq!(moonbeam_runtime::ProxyType::Balances as u8, 5);
	assert_eq!(moonbeam_runtime::ProxyType::AuthorMapping as u8, 6);
	assert_eq!(moonbeam_runtime::ProxyType::IdentityJudgement as u8, 7);
}

// This test ensure that we not filter out pure proxy calls
#[test]
fn verify_normal_filter_allow_pure_proxy() {
	ExtBuilder::default().build().execute_with(|| {
		assert!(NormalFilter::contains(&RuntimeCall::Proxy(
			pallet_proxy::Call::<Runtime>::create_pure {
				proxy_type: ProxyType::Any,
				delay: 0,
				index: 0,
			}
		)));
		assert!(NormalFilter::contains(&RuntimeCall::Proxy(
			pallet_proxy::Call::<Runtime>::kill_pure {
				spawner: AccountId::from(ALICE),
				proxy_type: ProxyType::Any,
				index: 0,
				height: 0,
				ext_index: 0,
			}
		)));
	});
}

#[test]
fn join_collator_candidates() {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), 10_000_000 * GLMR),
			(AccountId::from(BOB), 10_000_000 * GLMR),
			(AccountId::from(CHARLIE), 10_000_000 * GLMR),
			(AccountId::from(DAVE), 10_000_000 * GLMR),
		])
		.with_collators(vec![
			(AccountId::from(ALICE), 2_000_000 * GLMR),
			(AccountId::from(BOB), 2_000_000 * GLMR),
		])
		.with_delegations(vec![
			(
				AccountId::from(CHARLIE),
				AccountId::from(ALICE),
				5_000 * GLMR,
			),
			(AccountId::from(CHARLIE), AccountId::from(BOB), 5_000 * GLMR),
		])
		.build()
		.execute_with(|| {
			assert_noop!(
				ParachainStaking::join_candidates(
					origin_of(AccountId::from(ALICE)),
					2_000_000 * GLMR,
					2u32
				),
				pallet_parachain_staking::Error::<Runtime>::CandidateExists
			);
			assert_noop!(
				ParachainStaking::join_candidates(
					origin_of(AccountId::from(CHARLIE)),
					2_000_000 * GLMR,
					2u32
				),
				pallet_parachain_staking::Error::<Runtime>::DelegatorExists
			);
			assert!(System::events().is_empty());
			assert_ok!(ParachainStaking::join_candidates(
				origin_of(AccountId::from(DAVE)),
				2_000_000 * GLMR,
				2u32
			));
			assert_eq!(
				last_event(),
				RuntimeEvent::ParachainStaking(
					pallet_parachain_staking::Event::JoinedCollatorCandidates {
						account: AccountId::from(DAVE),
						amount_locked: 2_000_000 * GLMR,
						new_total_amt_locked: 6_010_000 * GLMR
					}
				)
			);
			let candidates = ParachainStaking::candidate_pool();
			assert_eq!(candidates.0[0].owner, AccountId::from(ALICE));
			assert_eq!(candidates.0[0].amount, 2_005_000 * GLMR);
			assert_eq!(candidates.0[1].owner, AccountId::from(BOB));
			assert_eq!(candidates.0[1].amount, 2_005_000 * GLMR);
			assert_eq!(candidates.0[2].owner, AccountId::from(DAVE));
			assert_eq!(candidates.0[2].amount, 2_000_000 * GLMR);
		});
}

#[test]
fn transfer_through_evm_to_stake() {
	ExtBuilder::default()
		.with_balances(vec![(AccountId::from(ALICE), 10_000_000 * GLMR)])
		.build()
		.execute_with(|| {
			// Charlie has no balance => fails to stake
			assert_noop!(
				ParachainStaking::join_candidates(
					origin_of(AccountId::from(CHARLIE)),
					2_000_000 * GLMR,
					2u32
				),
				DispatchError::Module(ModuleError {
					index: 20,
					error: [8, 0, 0, 0],
					message: Some("InsufficientBalance")
				})
			);
			// Alice transfer from free balance 3_000_000 GLMR to Bob
			assert_ok!(Balances::transfer_allow_death(
				origin_of(AccountId::from(ALICE)),
				AccountId::from(BOB),
				3_000_000 * GLMR,
			));
			assert_eq!(
				Balances::free_balance(AccountId::from(BOB)),
				3_000_000 * GLMR
			);

			let gas_limit = 100000u64;
			let gas_price: U256 = BASE_FEE_GENESIS.into();
			// Bob transfers 2_000_000 GLMR to Charlie via EVM
			assert_ok!(RuntimeCall::EVM(pallet_evm::Call::<Runtime>::call {
				source: H160::from(BOB),
				target: H160::from(CHARLIE),
				input: vec![],
				value: (2_000_000 * GLMR).into(),
				gas_limit,
				max_fee_per_gas: gas_price,
				max_priority_fee_per_gas: None,
				nonce: None,
				access_list: Vec::new(),
				authorization_list: Vec::new(),
			})
			.dispatch(<Runtime as frame_system::Config>::RuntimeOrigin::root()));
			assert_eq!(
				Balances::free_balance(AccountId::from(CHARLIE)),
				2_000_000 * GLMR,
			);

			// Charlie can stake now
			assert_ok!(ParachainStaking::join_candidates(
				origin_of(AccountId::from(CHARLIE)),
				2_000_000 * GLMR,
				2u32
			),);
			let candidates = ParachainStaking::candidate_pool();
			assert_eq!(candidates.0[0].owner, AccountId::from(CHARLIE));
			assert_eq!(candidates.0[0].amount, 2_000_000 * GLMR);
		});
}

#[test]
fn reward_block_authors() {
	ExtBuilder::default()
		.with_balances(vec![
			// Alice gets 10k extra tokens for her mapping deposit
			(AccountId::from(ALICE), 10_010_000 * GLMR),
			(AccountId::from(BOB), 10_000_000 * GLMR),
		])
		.with_collators(vec![(AccountId::from(ALICE), 2_000_000 * GLMR)])
		.with_delegations(vec![(
			AccountId::from(BOB),
			AccountId::from(ALICE),
			50_000 * GLMR,
		)])
		.with_mappings(vec![(
			NimbusId::from_slice(&ALICE_NIMBUS).unwrap(),
			AccountId::from(ALICE),
		)])
		.build()
		.execute_with(|| {
			increase_last_relay_slot_number(1);
			// Just before round 3
			run_to_block(7199, Some(NimbusId::from_slice(&ALICE_NIMBUS).unwrap()));
			// no rewards doled out yet
			assert_eq!(
				Balances::usable_balance(AccountId::from(ALICE)),
				8_010_000 * GLMR,
			);
			assert_eq!(
				Balances::usable_balance(AccountId::from(BOB)),
				9_950_000 * GLMR,
			);
			run_to_block(7201, Some(NimbusId::from_slice(&ALICE_NIMBUS).unwrap()));
			// rewards minted and distributed
			assert_eq!(
				Balances::usable_balance(AccountId::from(ALICE)),
				8990978048702400000000000,
			);
			assert_eq!(
				Balances::usable_balance(AccountId::from(BOB)),
				9969521950497200000000000,
			);
		});
}

#[test]
fn reward_block_authors_with_parachain_bond_reserved() {
	ExtBuilder::default()
		.with_balances(vec![
			// Alice gets 10k extra tokens for her mapping deposit
			(AccountId::from(ALICE), 10_010_000 * GLMR),
			(AccountId::from(BOB), 10_000_000 * GLMR),
			(AccountId::from(CHARLIE), 10_000 * GLMR),
		])
		.with_collators(vec![(AccountId::from(ALICE), 2_000_000 * GLMR)])
		.with_delegations(vec![(
			AccountId::from(BOB),
			AccountId::from(ALICE),
			50_000 * GLMR,
		)])
		.with_mappings(vec![(
			NimbusId::from_slice(&ALICE_NIMBUS).unwrap(),
			AccountId::from(ALICE),
		)])
		.build()
		.execute_with(|| {
			increase_last_relay_slot_number(1);
			assert_ok!(ParachainStaking::set_inflation_distribution_config(
				root_origin(),
				[
					InflationDistributionAccount {
						account: AccountId::from(CHARLIE),
						percent: Percent::from_percent(30),
					},
					InflationDistributionAccount::default(),
				]
				.into()
			));

			// Stop just before round 3
			run_to_block(7199, Some(NimbusId::from_slice(&ALICE_NIMBUS).unwrap()));
			// no collators rewards doled out yet
			assert_eq!(
				Balances::usable_balance(AccountId::from(ALICE)),
				8_010_000 * GLMR,
			);
			assert_eq!(
				Balances::usable_balance(AccountId::from(BOB)),
				9_950_000 * GLMR,
			);
			// 30% reserved for parachain bond
			assert_eq!(
				Balances::usable_balance(AccountId::from(CHARLIE)),
				310300000000000000000000,
			);

			// Go to round 3
			run_to_block(7201, Some(NimbusId::from_slice(&ALICE_NIMBUS).unwrap()));

			// collators rewards minted and distributed
			assert_eq!(
				Balances::usable_balance(AccountId::from(ALICE)),
				8698492682878000000000000,
			);
			assert_eq!(
				Balances::usable_balance(AccountId::from(BOB)),
				9962207316621500000000000,
			);
			// 30% reserved for parachain bond again
			assert_eq!(
				Balances::usable_balance(AccountId::from(CHARLIE)),
				615104500000000000000000,
			);
		});
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
		let uxt: TestXt<_, ()> = TestXt::new_signed(RuntimeCall::System(call), 1u64, (), ());

		let calc_fee = |len: u32| -> Balance {
			moonbeam_runtime::TransactionPayment::query_fee_details(uxt.clone(), len)
				.inclusion_fee
				.expect("fee should be calculated")
				.len_fee
		};

		//                  left: cost of length fee, right: size in bytes
		//                             /------------- proportional component: O(N * 1B)
		//                             |           /- exponential component: O(N ** 3)
		//                             |           |
		assert_eq!(                    100_000_000_100, calc_fee(1));
		assert_eq!(                  1_000_000_100_000, calc_fee(10));
		assert_eq!(                 10_000_100_000_000, calc_fee(100));
		assert_eq!(                100_100_000_000_000, calc_fee(1_000));
		assert_eq!(              1_100_000_000_000_000, calc_fee(10_000)); // inflection point
		assert_eq!(            110_000_000_000_000_000, calc_fee(100_000));
		assert_eq!(        100_100_000_000_000_000_000, calc_fee(1_000_000)); // 100 GLMR, ~ 1MB
		assert_eq!(    100_001_000_000_000_000_000_000, calc_fee(10_000_000));
		assert_eq!(100_000_010_000_000_000_000_000_000, calc_fee(100_000_000));
	});
}

#[test]
fn multiplier_can_grow_from_zero() {
	use frame_support::traits::Get;

	let minimum_multiplier = moonbeam_runtime::MinimumMultiplier::get();
	let target = moonbeam_runtime::TargetBlockFullness::get()
		* RuntimeBlockWeights::get()
			.get(DispatchClass::Normal)
			.max_total
			.unwrap();
	// if the min is too small, then this will not change, and we are doomed forever.
	// the weight is 1/100th bigger than target.
	run_with_system_weight(target * 101 / 100, || {
		let next = moonbeam_runtime::SlowAdjustingFeeUpdate::<Runtime>::convert(minimum_multiplier);
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
		set_parachain_inherent_data();
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
fn initial_gas_fee_is_correct() {
	use fp_evm::FeeCalculator;

	ExtBuilder::default().build().execute_with(|| {
		let multiplier = TransactionPayment::next_fee_multiplier();
		assert_eq!(multiplier, Multiplier::from(1u128));

		assert_eq!(
			TransactionPaymentAsGasPrice::min_gas_price(),
			(
				31_250_000_000u128.into(),
				Weight::from_parts(<Runtime as frame_system::Config>::DbWeight::get().read, 0)
			)
		);
	});
}

#[test]
fn min_gas_fee_is_correct() {
	use fp_evm::FeeCalculator;
	use frame_support::traits::Hooks;

	ExtBuilder::default().build().execute_with(|| {
		pallet_transaction_payment::NextFeeMultiplier::<Runtime>::put(Multiplier::from(0));
		TransactionPayment::on_finalize(System::block_number()); // should trigger min to kick in

		let multiplier = TransactionPayment::next_fee_multiplier();
		assert_eq!(multiplier, Multiplier::from(1u128));

		assert_eq!(
			TransactionPaymentAsGasPrice::min_gas_price(),
			(
				31_250_000_000u128.into(),
				Weight::from_parts(<Runtime as frame_system::Config>::DbWeight::get().read, 0)
			)
		);
	});
}

#[test]
fn transfer_ed_0_substrate() {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), (1 * GLMR) + (1 * WEI)),
			(AccountId::from(BOB), existential_deposit()),
		])
		.build()
		.execute_with(|| {
			// Substrate transfer
			assert_ok!(Balances::transfer_allow_death(
				origin_of(AccountId::from(ALICE)),
				AccountId::from(BOB),
				1 * GLMR,
			));
			// 1 WEI is left in the account
			assert_eq!(Balances::free_balance(AccountId::from(ALICE)), 1 * WEI);
		});
}

#[test]
fn transfer_ed_0_evm() {
	ExtBuilder::default()
		.with_balances(vec![
			(
				AccountId::from(ALICE),
				((1 * GLMR) + (21_000 * BASE_FEE_GENESIS)) + (1 * WEI),
			),
			(AccountId::from(BOB), existential_deposit()),
		])
		.build()
		.execute_with(|| {
			set_parachain_inherent_data();
			// EVM transfer
			assert_ok!(RuntimeCall::EVM(pallet_evm::Call::<Runtime>::call {
				source: H160::from(ALICE),
				target: H160::from(BOB),
				input: Vec::new(),
				value: (1 * GLMR).into(),
				gas_limit: 21_000u64,
				max_fee_per_gas: BASE_FEE_GENESIS.into(),
				max_priority_fee_per_gas: Some(BASE_FEE_GENESIS.into()),
				nonce: Some(U256::from(0)),
				access_list: Vec::new(),
				authorization_list: Vec::new(),
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
				((1 * GLMR) + (21_777 * BASE_FEE_GENESIS) + existential_deposit()),
			),
			(AccountId::from(BOB), existential_deposit()),
		])
		.build()
		.execute_with(|| {
			set_parachain_inherent_data();
			// EVM transfer that zeroes ALICE
			assert_ok!(RuntimeCall::EVM(pallet_evm::Call::<Runtime>::call {
				source: H160::from(ALICE),
				target: H160::from(BOB),
				input: Vec::new(),
				value: (1 * GLMR).into(),
				gas_limit: 21_777u64,
				max_fee_per_gas: BASE_FEE_GENESIS.into(),
				max_priority_fee_per_gas: Some(BASE_FEE_GENESIS.into()),
				nonce: Some(U256::from(0)),
				access_list: Vec::new(),
				authorization_list: Vec::new(),
			})
			.dispatch(<Runtime as frame_system::Config>::RuntimeOrigin::root()));
			// ALICE is refunded
			assert_eq!(
				Balances::free_balance(AccountId::from(ALICE)),
				777 * BASE_FEE_GENESIS + existential_deposit(),
			);
		});
}

#[test]
fn author_does_receive_priority_fee() {
	ExtBuilder::default()
		.with_balances(vec![(
			AccountId::from(BOB),
			(1 * GLMR) + (21_000 * (500 * GIGAWEI)),
		)])
		.build()
		.execute_with(|| {
			// Some block author as seen by pallet-evm.
			let author = AccountId::from(<pallet_evm::Pallet<Runtime>>::find_author());
			pallet_author_inherent::Author::<Runtime>::put(author);
			// Currently the default impl of the evm uses `deposit_into_existing`.
			// If we were to use this implementation, and for an author to receive eventual tips,
			// the account needs to be somehow initialized, otherwise the deposit would fail.
			Balances::make_free_balance_be(&author, 100 * GLMR);

			// EVM transfer.
			assert_ok!(RuntimeCall::EVM(pallet_evm::Call::<Runtime>::call {
				source: H160::from(BOB),
				target: H160::from(ALICE),
				input: Vec::new(),
				value: (1 * GLMR).into(),
				gas_limit: 21_000u64,
				max_fee_per_gas: U256::from(300 * GIGAWEI),
				max_priority_fee_per_gas: Some(U256::from(200 * GIGAWEI)),
				nonce: Some(U256::from(0)),
				access_list: Vec::new(),
				authorization_list: Vec::new(),
			})
			.dispatch(<Runtime as frame_system::Config>::RuntimeOrigin::root()));

			let priority_fee = 200 * GIGAWEI * 21_000;
			// Author free balance increased by priority fee.
			assert_eq!(Balances::free_balance(author), 100 * GLMR + priority_fee,);
		});
}

#[test]
fn total_issuance_after_evm_transaction_with_priority_fee() {
	use fp_evm::FeeCalculator;
	ExtBuilder::default()
		.with_balances(vec![
			(
				AccountId::from(BOB),
				(1 * GLMR) + (21_000 * (200 * GIGAWEI) + existential_deposit()),
			),
			(
				<pallet_treasury::TreasuryAccountId<Runtime> as sp_core::TypedGet>::get(),
				existential_deposit(),
			),
		])
		.build()
		.execute_with(|| {
			let issuance_before = <Runtime as pallet_evm::Config>::Currency::total_issuance();
			let author = AccountId::from(<pallet_evm::Pallet<Runtime>>::find_author());
			pallet_author_inherent::Author::<Runtime>::put(author);
			// EVM transfer.
			assert_ok!(RuntimeCall::EVM(pallet_evm::Call::<Runtime>::call {
				source: H160::from(BOB),
				target: H160::from(ALICE),
				input: Vec::new(),
				value: (1 * GLMR).into(),
				gas_limit: 21_000u64,
				max_fee_per_gas: U256::from(125 * GIGAWEI),
				max_priority_fee_per_gas: Some(U256::from(100 * GIGAWEI)),
				nonce: Some(U256::from(0)),
				access_list: Vec::new(),
				authorization_list: Vec::new(),
			})
			.dispatch(<Runtime as frame_system::Config>::RuntimeOrigin::root()));

			let issuance_after = <Runtime as pallet_evm::Config>::Currency::total_issuance();

			let base_fee = TransactionPaymentAsGasPrice::min_gas_price().0.as_u128();

			let base_fee: Balance = base_fee * 21_000;

			let treasury_proportion = dynamic_params::runtime_config::FeesTreasuryProportion::get();

			// only base fee is split between being burned and sent to treasury
			let treasury_base_fee_part: Balance = treasury_proportion.mul_floor(base_fee);
			let burnt_base_fee_part: Balance = base_fee - treasury_base_fee_part;

			assert_eq!(issuance_after, issuance_before - burnt_base_fee_part);

			assert_eq!(moonbeam_runtime::Treasury::pot(), treasury_base_fee_part);
		});
}

#[test]
fn total_issuance_after_evm_transaction_without_priority_fee() {
	use fp_evm::FeeCalculator;
	ExtBuilder::default()
		.with_balances(vec![
			(
				AccountId::from(BOB),
				(1 * GLMR) + (21_000 * BASE_FEE_GENESIS + existential_deposit()),
			),
			(
				<pallet_treasury::TreasuryAccountId<Runtime> as sp_core::TypedGet>::get(),
				existential_deposit(),
			),
		])
		.build()
		.execute_with(|| {
			set_parachain_inherent_data();
			let issuance_before = <Runtime as pallet_evm::Config>::Currency::total_issuance();
			// EVM transfer.
			assert_ok!(RuntimeCall::EVM(pallet_evm::Call::<Runtime>::call {
				source: H160::from(BOB),
				target: H160::from(ALICE),
				input: Vec::new(),
				value: (1 * GLMR).into(),
				gas_limit: 21_000u64,
				max_fee_per_gas: BASE_FEE_GENESIS.into(),
				max_priority_fee_per_gas: Some(BASE_FEE_GENESIS.into()),
				nonce: Some(U256::from(0)),
				access_list: Vec::new(),
				authorization_list: Vec::new(),
			})
			.dispatch(<Runtime as frame_system::Config>::RuntimeOrigin::root()));

			let issuance_after = <Runtime as pallet_evm::Config>::Currency::total_issuance();

			let base_fee = TransactionPaymentAsGasPrice::min_gas_price().0.as_u128();

			let base_fee: Balance = base_fee * 21_000;

			let treasury_proportion = dynamic_params::runtime_config::FeesTreasuryProportion::get();

			// only base fee is split between being burned and sent to treasury
			let treasury_base_fee_part: Balance = treasury_proportion.mul_floor(base_fee);
			let burnt_base_fee_part: Balance = base_fee - treasury_base_fee_part;

			assert_eq!(issuance_after, issuance_before - burnt_base_fee_part);

			assert_eq!(moonbeam_runtime::Treasury::pot(), treasury_base_fee_part);
		});
}

#[test]
fn root_can_change_default_xcm_vers() {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * GLMR),
			(AccountId::from(BOB), 1_000 * GLMR),
		])
		.with_xcm_assets(vec![XcmAssetInitialization {
			asset_id: 1,
			xcm_location: AssetHubLocation::get(),
			name: "Dot",
			symbol: "Dot",
			decimals: 12,
			balances: vec![(AccountId::from(ALICE), 1_000_000_000_000_000)],
		}])
		.build()
		.execute_with(|| {
			let source_id: moonbeam_runtime::AssetId = 1;
			let currency_id = moonbeam_runtime::xcm_config::CurrencyId::ForeignAsset(source_id);
			let asset = Asset {
				id: AssetId(
					<Runtime as pallet_xcm_transactor::Config>::CurrencyIdToLocation::convert(
						currency_id,
					)
					.unwrap(),
				),
				fun: Fungibility::Fungible(100_000_000_000_000),
			};
			// Default XCM version is not set yet, so xtokens should fail because it does not
			// know with which version to send
			assert_noop!(
				PolkadotXcm::transfer_assets(
					origin_of(AccountId::from(ALICE)),
					Box::new(VersionedLocation::from(Location::parent())),
					Box::new(VersionedLocation::from(Location {
						parents: 0,
						interior: [AccountId32 {
							network: None,
							id: [1u8; 32],
						}]
						.into(),
					})),
					Box::new(VersionedAssets::from(asset.clone())),
					0,
					WeightLimit::Unlimited,
				),
				pallet_xcm::Error::<Runtime>::LocalExecutionIncompleteWithError {
					index: 2,
					error: pallet_xcm::ExecutionError::DestinationUnsupported
				}
			);

			// Root sets the defaultXcm
			assert_ok!(PolkadotXcm::force_default_xcm_version(
				root_origin(),
				Some(5)
			));

			// Now transferring does not fail
			assert_ok!(PolkadotXcm::transfer_assets(
				origin_of(AccountId::from(ALICE)),
				Box::new(VersionedLocation::from(Location::parent())),
				Box::new(VersionedLocation::from(Location {
					parents: 0,
					interior: [AccountId32 {
						network: None,
						id: [1u8; 32],
					}]
					.into(),
				})),
				Box::new(VersionedAssets::from(asset)),
				0,
				WeightLimit::Unlimited
			));
		})
}

#[test]
fn asset_can_be_registered() {
	ExtBuilder::default().build().execute_with(|| {
		let source_location = Location::parent();
		let source_id = 1;

		assert_ok!(EvmForeignAssets::create_foreign_asset(
			moonbeam_runtime::RuntimeOrigin::root(),
			source_id,
			source_location.clone(),
			12,
			b"Relay".to_vec().try_into().unwrap(),
			b"RelayToken".to_vec().try_into().unwrap(),
		));

		// Check that the asset was created
		// First check if the asset ID exists
		let location = EvmForeignAssets::assets_by_id(source_id).expect("Asset should exist");
		assert_eq!(location.clone(), source_location);

		// Then check the status using AssetsByLocation
		let (asset_id, status) =
			EvmForeignAssets::assets_by_location(&location).expect("Asset location should exist");
		assert_eq!(asset_id, source_id);
		assert_eq!(status, AssetStatus::Active);
	});
}

// The precoompile asset-erc20 is deprecated and not used anymore for new evm foreign assets
// We don't have testing tools in rust test to call real evm smart contract, so we rely on ts tests.
/*
#[test]
fn xcm_asset_erc20_precompiles_supply_and_balance() {
	ExtBuilder::default()
		.with_xcm_assets(vec![XcmAssetInitialization {
			asset_type: AssetType::Xcm(xcm::v3::Location::parent()),
			metadata: AssetRegistrarMetadata {
				name: b"RelayToken".to_vec(),
				symbol: b"Relay".to_vec(),
				decimals: 12,
				is_frozen: false,
			},
			balances: vec![(AccountId::from(ALICE), 1_000 * GLMR)],
			is_sufficient: true,
		}])
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * GLMR),
			(AccountId::from(BOB), 1_000 * GLMR),
		])
		.build()
		.execute_with(|| {
			// We have the assetId that corresponds to the relay chain registered
			let relay_asset_id: moonbeam_runtime::AssetId =
				AssetType::Xcm(xcm::v3::Location::parent()).into();

			// Its address is
			let asset_precompile_address = Runtime::asset_id_to_account(
				FOREIGN_ASSET_PRECOMPILE_ADDRESS_PREFIX,
				relay_asset_id,
			);

			// Assert the asset has been created with the correct supply
			assert_eq!(
				moonbeam_runtime::Assets::total_supply(relay_asset_id),
				1_000 * GLMR
			);

			// Access totalSupply through precompile. Important that the context is correct
			Precompiles::new()
				.prepare_test(
					ALICE,
					asset_precompile_address,
					ForeignAssetsPCall::total_supply {},
				)
				.expect_cost(5007)
				.expect_no_logs()
				.execute_returns(U256::from(1000 * GLMR));

			// Access balanceOf through precompile
			Precompiles::new()
				.prepare_test(
					ALICE,
					asset_precompile_address,
					ForeignAssetsPCall::balance_of {
						who: Address(ALICE.into()),
					},
				)
				.expect_cost(5007)
				.expect_no_logs()
				.execute_returns(U256::from(1000 * GLMR));
		});
}

#[test]
fn xcm_asset_erc20_precompiles_transfer() {
	ExtBuilder::default()
		.with_xcm_assets(vec![XcmAssetInitialization {
			asset_type: AssetType::Xcm(xcm::v3::Location::parent()),
			metadata: AssetRegistrarMetadata {
				name: b"RelayToken".to_vec(),
				symbol: b"Relay".to_vec(),
				decimals: 12,
				is_frozen: false,
			},
			balances: vec![(AccountId::from(ALICE), 1_000 * GLMR)],
			is_sufficient: true,
		}])
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * GLMR),
			(AccountId::from(BOB), 1_000 * GLMR),
		])
		.build()
		.execute_with(|| {
			// We have the assetId that corresponds to the relay chain registered
			let relay_asset_id: moonbeam_runtime::AssetId =
				AssetType::Xcm(xcm::v3::Location::parent()).into();

			// Its address is
			let asset_precompile_address = Runtime::asset_id_to_account(
				FOREIGN_ASSET_PRECOMPILE_ADDRESS_PREFIX,
				relay_asset_id,
			);

			// Transfer tokens from Aice to Bob, 400 GLMR.
			Precompiles::new()
				.prepare_test(
					ALICE,
					asset_precompile_address,
					ForeignAssetsPCall::transfer {
						to: Address(BOB.into()),
						value: { 400 * GLMR }.into(),
					},
				)
				.expect_cost(26580)
				.expect_log(log3(
					asset_precompile_address,
					SELECTOR_LOG_TRANSFER,
					H160::from(ALICE),
					H160::from(BOB),
					solidity::encode_event_data(U256::from(400 * GLMR)),
				))
				.execute_returns(true);

			// Make sure BOB has 400 GLMR
			Precompiles::new()
				.prepare_test(
					BOB,
					asset_precompile_address,
					ForeignAssetsPCall::balance_of {
						who: Address(BOB.into()),
					},
				)
				.expect_cost(5007)
				.expect_no_logs()
				.execute_returns(U256::from(400 * GLMR));
		});
}

#[test]
fn xcm_asset_erc20_precompiles_approve() {
	ExtBuilder::default()
		.with_xcm_assets(vec![XcmAssetInitialization {
			asset_type: AssetType::Xcm(xcm::v3::Location::parent()),
			metadata: AssetRegistrarMetadata {
				name: b"RelayToken".to_vec(),
				symbol: b"Relay".to_vec(),
				decimals: 12,
				is_frozen: false,
			},
			balances: vec![(AccountId::from(ALICE), 1_000 * GLMR)],
			is_sufficient: true,
		}])
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * GLMR),
			(AccountId::from(BOB), 1_000 * GLMR),
		])
		.build()
		.execute_with(|| {
			// We have the assetId that corresponds to the relay chain registered
			let relay_asset_id: moonbeam_runtime::AssetId =
				AssetType::Xcm(xcm::v3::Location::parent()).into();

			// Its address is
			let asset_precompile_address = Runtime::asset_id_to_account(
				FOREIGN_ASSET_PRECOMPILE_ADDRESS_PREFIX,
				relay_asset_id,
			);

			// Aprove Bob for spending 400 GLMR from Alice
			Precompiles::new()
				.prepare_test(
					ALICE,
					asset_precompile_address,
					ForeignAssetsPCall::approve {
						spender: Address(BOB.into()),
						value: { 400 * GLMR }.into(),
					},
				)
				.expect_cost(17323)
				.expect_log(log3(
					asset_precompile_address,
					SELECTOR_LOG_APPROVAL,
					H160::from(ALICE),
					H160::from(BOB),
					solidity::encode_event_data(U256::from(400 * GLMR)),
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
						value: { 400 * GLMR }.into(),
					},
				)
				.expect_cost(31887)
				.expect_log(log3(
					asset_precompile_address,
					SELECTOR_LOG_TRANSFER,
					H160::from(ALICE),
					H160::from(CHARLIE),
					solidity::encode_event_data(U256::from(400 * GLMR)),
				))
				.execute_returns(true);

			// Make sure CHARLIE has 400 GLMR
			Precompiles::new()
				.prepare_test(
					CHARLIE,
					asset_precompile_address,
					ForeignAssetsPCall::balance_of {
						who: Address(CHARLIE.into()),
					},
				)
				.expect_cost(5007)
				.expect_no_logs()
				.execute_returns(U256::from(400 * GLMR));
		});
}*/

#[test]
fn make_sure_glmr_can_be_transferred_precompile() {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * GLMR),
			(AccountId::from(BOB), 1_000 * GLMR),
		])
		.with_collators(vec![(AccountId::from(ALICE), 1_000 * GLMR)])
		.with_mappings(vec![(
			NimbusId::from_slice(&ALICE_NIMBUS).unwrap(),
			AccountId::from(ALICE),
		)])
		.with_safe_xcm_version(3)
		.build()
		.execute_with(|| {
			assert_ok!(PolkadotXcm::transfer_assets(
				origin_of(AccountId::from(ALICE)),
				Box::new(VersionedLocation::from(Location::parent())),
				Box::new(VersionedLocation::from(Location {
					parents: 0,
					interior: [AccountId32 {
						network: None,
						id: [1u8; 32],
					}]
					.into(),
				})),
				Box::new(VersionedAssets::from(Asset {
					id: AssetId(moonbeam_runtime::xcm_config::SelfReserve::get()),
					fun: Fungible(1000)
				})),
				0,
				WeightLimit::Limited(40000.into())
			));
		});
}

#[test]
fn make_sure_glmr_can_be_transferred() {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * GLMR),
			(AccountId::from(BOB), 1_000 * GLMR),
		])
		.with_collators(vec![(AccountId::from(ALICE), 1_000 * GLMR)])
		.with_mappings(vec![(
			NimbusId::from_slice(&ALICE_NIMBUS).unwrap(),
			AccountId::from(ALICE),
		)])
		.with_safe_xcm_version(3)
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
			assert_ok!(PolkadotXcm::transfer_assets(
				origin_of(AccountId::from(ALICE)),
				Box::new(VersionedLocation::from(Location::parent())),
				Box::new(VersionedLocation::from(dest)),
				Box::new(VersionedAssets::from(Asset {
					id: AssetId(moonbeam_runtime::xcm_config::SelfReserve::get()),
					fun: Fungible(100)
				})),
				0,
				WeightLimit::Limited(40000.into())
			));
		});
}

#[test]
fn make_sure_polkadot_xcm_cannot_be_called() {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * GLMR),
			(AccountId::from(BOB), 1_000 * GLMR),
		])
		.with_collators(vec![(AccountId::from(ALICE), 1_000 * GLMR)])
		.with_mappings(vec![(
			NimbusId::from_slice(&ALICE_NIMBUS).unwrap(),
			AccountId::from(ALICE),
		)])
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
			let assets: Assets = [Asset {
				id: AssetId(moonbeam_runtime::xcm_config::SelfLocation::get()),
				fun: Fungible(1000),
			}]
			.to_vec()
			.into();
			assert_noop!(
				RuntimeCall::PolkadotXcm(pallet_xcm::Call::<Runtime>::reserve_transfer_assets {
					dest: Box::new(VersionedLocation::from(dest.clone())),
					beneficiary: Box::new(VersionedLocation::from(dest)),
					assets: Box::new(VersionedAssets::from(assets)),
					fee_asset_item: 0,
				})
				.dispatch(<Runtime as frame_system::Config>::RuntimeOrigin::signed(
					AccountId::from(ALICE)
				)),
				frame_system::Error::<Runtime>::CallFiltered
			);
		});
}

#[test]
fn transact_through_signed_precompile_works_v2() {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * GLMR),
			(AccountId::from(BOB), 1_000 * GLMR),
		])
		.with_safe_xcm_version(3)
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
				.expect_cost(31331)
				.expect_no_logs()
				.execute_returns(());
		});
}

#[test]
fn transact_through_signed_cannot_send_to_local_chain() {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * GLMR),
			(AccountId::from(BOB), 1_000 * GLMR),
		])
		.with_safe_xcm_version(3)
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

#[test]
fn transactor_cannot_use_more_than_max_weight() {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * GLMR),
			(AccountId::from(BOB), 1_000 * GLMR),
		])
		.with_xcm_assets(vec![XcmAssetInitialization {
			asset_id: 1,
			xcm_location: xcm::v5::Location::parent(),
			name: "RelayToken",
			symbol: "Relay",
			decimals: 12,
			balances: vec![(AccountId::from(ALICE), 1_000_000_000_000_000)],
		}])
		.build()
		.execute_with(|| {
			let source_id: moonbeam_runtime::AssetId = 1;
			assert_ok!(XcmTransactor::register(
				root_origin(),
				AccountId::from(ALICE),
				0,
			));

			// Root can set transact info
			assert_ok!(XcmTransactor::set_transact_info(
				root_origin(),
				Box::new(xcm::VersionedLocation::from(Location::parent())),
				// Relay charges 1000 for every instruction, and we have 3, so 3000
				3000.into(),
				20000.into(),
				None
			));
			// Root can set transact info
			assert_ok!(XcmTransactor::set_fee_per_second(
				root_origin(),
				Box::new(xcm::VersionedLocation::from(Location::parent())),
				1,
			));

			assert_noop!(
				XcmTransactor::transact_through_derivative(
					origin_of(AccountId::from(ALICE)),
					moonbeam_runtime::xcm_config::Transactors::Relay,
					0,
					CurrencyPayment {
						currency: Currency::AsMultiLocation(Box::new(
							xcm::VersionedLocation::from(Location::parent())
						)),
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
					moonbeam_runtime::xcm_config::Transactors::Relay,
					0,
					CurrencyPayment {
						currency: Currency::AsCurrencyId(
							moonbeam_runtime::xcm_config::CurrencyId::ForeignAsset(source_id)
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
fn call_pallet_xcm_with_fee() {
	let asset_id = 1;
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * GLMR),
			(AccountId::from(BOB), 1_000 * GLMR),
		])
		.with_safe_xcm_version(3)
		.with_xcm_assets(vec![XcmAssetInitialization {
			asset_id,
			xcm_location: Location::parent(),
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

			let before_balance =
				EvmForeignAssets::balance(asset_id, AccountId::from(ALICE)).unwrap();
			let (chain_part, beneficiary) =
				split_location_into_chain_part_and_beneficiary(dest).unwrap();
			let asset = currency_to_asset(CurrencyId::ForeignAsset(asset_id), 100_000_000_000_000);
			let asset_fee = currency_to_asset(CurrencyId::ForeignAsset(asset_id), 100);
			let fees_id: VersionedAssetId = AssetId(Location::parent()).into();
			let xcm_on_dest = Xcm::<()>(vec![DepositAsset {
				assets: Wild(All),
				beneficiary: beneficiary.clone(),
			}]);
			assert_ok!(PolkadotXcm::transfer_assets_using_type_and_then(
				origin_of(AccountId::from(ALICE)),
				Box::new(VersionedLocation::from(chain_part)),
				Box::new(VersionedAssets::from(vec![asset_fee, asset])),
				Box::new(TransferType::DestinationReserve),
				Box::new(fees_id),
				Box::new(TransferType::DestinationReserve),
				Box::new(VersionedXcm::V5(xcm_on_dest)),
				WeightLimit::Limited(4000000000.into())
			));

			let after_balance =
				EvmForeignAssets::balance(asset_id, AccountId::from(ALICE)).unwrap();
			// Balance should have been reduced by the transfer amount plus fees
			assert!(after_balance < before_balance);
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
			.expect_cost(
				<Runtime as frame_system::Config>::DbWeight::get()
					.read
					.saturating_div(WEIGHT_PER_GAS)
					.saturating_mul(2),
			)
			.expect_no_logs()
			.execute_returns(Address(expected_address_parent));

		let parachain_2000_location = Location::new(1, [Parachain(2000)]);
		let expected_address_parachain: H160 =
			SiblingParachainConvertsVia::<Sibling, AccountId>::convert_location(
				&parachain_2000_location,
			)
			.unwrap()
			.into();

		Precompiles::new()
			.prepare_test(
				ALICE,
				xcm_utils_precompile_address,
				XcmUtilsPCall::multilocation_to_address {
					location: parachain_2000_location,
				},
			)
			.expect_cost(
				<Runtime as frame_system::Config>::DbWeight::get()
					.read
					.saturating_div(WEIGHT_PER_GAS)
					.saturating_mul(2),
			)
			.expect_no_logs()
			.execute_returns(Address(expected_address_parachain));

		let alice_in_parachain_2000_location = Location::new(
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
			>::convert_location(&alice_in_parachain_2000_location)
			.unwrap()
			.into();

		Precompiles::new()
			.prepare_test(
				ALICE,
				xcm_utils_precompile_address,
				XcmUtilsPCall::multilocation_to_address {
					location: alice_in_parachain_2000_location,
				},
			)
			.expect_cost(
				<Runtime as frame_system::Config>::DbWeight::get()
					.read
					.saturating_div(WEIGHT_PER_GAS)
					.saturating_mul(2),
			)
			.expect_no_logs()
			.execute_returns(Address(expected_address_alice_in_parachain_2000));
	});
}

#[test]
fn test_xcm_utils_weight_message() {
	ExtBuilder::default().build().execute_with(|| {
		let xcm_utils_precompile_address = H160::from_low_u64_be(2060);
		let expected_weight =
			XcmWeight::<moonbeam_runtime::Runtime, RuntimeCall>::clear_origin().ref_time();

		let message: Vec<u8> = xcm::VersionedXcm::<()>::V5(Xcm(vec![ClearOrigin])).encode();

		let input = XcmUtilsPCall::weight_message {
			message: message.into(),
		};

		Precompiles::new()
			.prepare_test(ALICE, xcm_utils_precompile_address, input)
			.expect_cost(
				<Runtime as frame_system::Config>::DbWeight::get()
					.read
					.saturating_div(WEIGHT_PER_GAS),
			)
			.expect_no_logs()
			.execute_returns(expected_weight);
	});
}

#[test]
fn test_nested_batch_calls_from_xcm_transact() {
	ExtBuilder::default().build().execute_with(|| {
		// This ensures we notice if MAX_XCM_DECODE_DEPTH changes
		// in a future polkadot-sdk version
		assert_eq!(xcm::MAX_XCM_DECODE_DEPTH, 8);

		let mut valid_nested_calls =
			RuntimeCall::System(frame_system::Call::remark { remark: vec![] });
		for _ in 0..xcm::MAX_XCM_DECODE_DEPTH {
			valid_nested_calls = RuntimeCall::Utility(pallet_utility::Call::batch {
				calls: vec![valid_nested_calls],
			});
		}

		let valid_message = Xcm(vec![Transact {
			origin_kind: OriginKind::SovereignAccount,
			fallback_max_weight: None,
			call: valid_nested_calls.encode().into(),
		}]);

		assert!(XcmExecutor::prepare(valid_message, Weight::MAX).is_ok());

		let excessive_nested_calls = RuntimeCall::Utility(pallet_utility::Call::batch {
			calls: vec![valid_nested_calls],
		});

		let invalid_message = Xcm(vec![Transact {
			origin_kind: OriginKind::SovereignAccount,
			fallback_max_weight: None,
			call: excessive_nested_calls.encode().into(),
		}]);
		// Expect to fail because we have too many nested calls
		assert!(XcmExecutor::prepare(invalid_message, Weight::MAX).is_err());
	});
}

#[test]
fn test_xcm_utils_get_units_per_second() {
	ExtBuilder::default().build().execute_with(|| {
		let xcm_utils_precompile_address = H160::from_low_u64_be(2060);
		let location = SelfReserve::get();

		let input = XcmUtilsPCall::get_units_per_second { location };

		let expected_units =
			WEIGHT_REF_TIME_PER_SECOND as u128 * moonbeam_runtime::currency::WEIGHT_FEE;

		Precompiles::new()
			.prepare_test(ALICE, xcm_utils_precompile_address, input)
			.expect_cost(
				<Runtime as frame_system::Config>::DbWeight::get()
					.read
					.saturating_div(WEIGHT_PER_GAS)
					.saturating_mul(2),
			)
			.expect_no_logs()
			.execute_returns(expected_units);
	});
}

#[test]
fn precompile_existence() {
	ExtBuilder::default().build().execute_with(|| {
		let precompiles = Precompiles::new();
		let precompile_addresses: std::collections::BTreeSet<_> = vec![
			1, 2, 3, 4, 5, 6, 7, 8, 9, 11, 12, 13, 14, 15, 16, 17, 256, 1024, 1025, 1026, 2048,
			2049, 2050, 2051, 2052, 2053, 2054, 2055, 2056, 2057, 2058, 2059, 2060, 2061, 2062,
			2063, 2064, 2065, 2066, 2067, 2068, 2069, 2070, 2071, 2072, 2073, 2074,
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
		let removed_precompiles = [1025, 1027, 2051, 2062, 2063];

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
fn deal_with_fees_handles_tip() {
	use frame_support::traits::OnUnbalanced;
	use moonbeam_runtime::Treasury;
	use moonbeam_runtime_common::deal_with_fees::DealWithSubstrateFeesAndTip;

	ExtBuilder::default().build().execute_with(|| {
		set_parachain_inherent_data();
		// This test validates the functionality of the `DealWithSubstrateFeesAndTip` trait implementation
		// in the Moonbeam runtime. It verifies that:
		// - The correct proportion of the fee is sent to the treasury.
		// - The remaining fee is burned (removed from the total supply).
		// - The entire tip is sent to the block author.

		// The test details:
		// 1. Simulate issuing a `fee` of 100 and a `tip` of 1000.
		// 2. Confirm the initial total supply is 1,100 (equal to the sum of the issued fee and tip).
		// 3. Confirm the treasury's balance is initially 0.
		// 4. Execute the `DealWithSubstrateFeesAndTip::on_unbalanceds` function with the `fee` and `tip`.
		// 5. Validate that the treasury's balance has increased by 20% of the fee (based on FeesTreasuryProportion).
		// 6. Validate that 80% of the fee is burned, and the total supply decreases accordingly.
		// 7. Validate that the entire tip (100%) is sent to the block author (collator).

		// Step 1: Issue the fee and tip amounts.
		let fee = <pallet_balances::Pallet<Runtime> as frame_support::traits::fungible::Balanced<
			AccountId,
		>>::issue(100);
		let tip = <pallet_balances::Pallet<Runtime> as frame_support::traits::fungible::Balanced<
			AccountId,
		>>::issue(1000);

		// Step 2: Validate the initial supply and balances.
		let total_supply_before = Balances::total_issuance();
		let block_author = pallet_author_inherent::Pallet::<Runtime>::get();
		let block_author_balance_before = Balances::free_balance(&block_author);
		assert_eq!(total_supply_before, 1_100);
		assert_eq!(Balances::free_balance(&Treasury::account_id()), 0);

		// Step 3: Execute the fees handling logic.
		DealWithSubstrateFeesAndTip::<
			Runtime,
			dynamic_params::runtime_config::FeesTreasuryProportion,
		>::on_unbalanceds(vec![fee, tip].into_iter());

		// Step 4: Compute the split between treasury and burned fees based on FeesTreasuryProportion (20%).
		let treasury_proportion = dynamic_params::runtime_config::FeesTreasuryProportion::get();

		let treasury_fee_part: Balance = treasury_proportion.mul_floor(100);
		let burnt_fee_part: Balance = 100 - treasury_fee_part;

		// Step 5: Validate the treasury received 20% of the fee.
		assert_eq!(
			Balances::free_balance(&Treasury::account_id()),
			treasury_fee_part,
		);

		// Step 6: Verify that 80% of the fee was burned (removed from the total supply).
		let total_supply_after = Balances::total_issuance();
		assert_eq!(total_supply_before - total_supply_after, burnt_fee_part,);

		// Step 7: Validate that the block author (collator) received 100% of the tip.
		let block_author_balance_after = Balances::free_balance(&block_author);
		assert_eq!(
			block_author_balance_after - block_author_balance_before,
			1000,
		);
	});
}

#[test]
fn evm_revert_substrate_events() {
	ExtBuilder::default()
		.with_balances(vec![(AccountId::from(ALICE), 100_000 * GLMR)])
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
					value: vec![U256::from(1 * GLMR), U256::zero()].into(),
					call_data: vec![].into(),
					gas_limit: vec![].into()
				}
				.into(),
				value: U256::zero(), // No value sent in EVM
				gas_limit: 500_000,
				max_fee_per_gas: BASE_FEE_GENESIS.into(),
				max_priority_fee_per_gas: None,
				nonce: Some(U256::from(0)),
				access_list: Vec::new(),
				authorization_list: Vec::new(),
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
		.with_balances(vec![(AccountId::from(ALICE), 100_000 * GLMR)])
		.build()
		.execute_with(|| {
			let batch_precompile_address = H160::from_low_u64_be(2056);

			assert_ok!(RuntimeCall::EVM(pallet_evm::Call::call {
				source: ALICE.into(),
				target: batch_precompile_address,
				input: BatchPCall::batch_all {
					to: vec![Address(BOB.into())].into(),
					value: vec![U256::from(1 * GLMR)].into(),
					call_data: vec![].into(),
					gas_limit: vec![].into()
				}
				.into(),
				value: U256::zero(), // No value sent in EVM
				gas_limit: 500_000,
				max_fee_per_gas: BASE_FEE_GENESIS.into(),
				max_priority_fee_per_gas: None,
				nonce: Some(U256::from(0)),
				access_list: Vec::new(),
				authorization_list: Vec::new(),
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

#[cfg(test)]
mod bridge_tests {
	use crate::common::{origin_of, root_origin, ExtBuilder, XcmAssetInitialization, ALICE, BOB};
	use crate::currency_to_asset;
	use bp_messages::target_chain::DispatchMessageData;
	use bp_messages::ReceptionResult;
	use bp_runtime::messages::MessageDispatchResult;
	use cumulus_primitives_core::AggregateMessageOrigin;
	use frame_support::assert_ok;
	use frame_support::pallet_prelude::{Hooks, PalletInfoAccess};
	use moonbeam_core_primitives::AccountId;
	use moonbeam_runtime::bridge_config::{
		KusamaGlobalConsensusNetwork, WithKusamaMessagesInstance,
	};
	use moonbeam_runtime::currency::GLMR;
	use moonbeam_runtime::xcm_config::CurrencyId;
	use moonbeam_runtime::{
		Balances, BridgeKusamaMessages, BridgeXcmOverMoonriver, MessageQueue, PolkadotXcm, Runtime,
		RuntimeEvent, System,
	};
	use pallet_bridge_messages::{LanesManager, StoredMessagePayload};
	use pallet_xcm_bridge::XcmBlobMessageDispatchResult::Dispatched;
	use parity_scale_codec::{Decode, Encode};
	use sp_core::H256;
	use xcm::latest::Junctions::X1;
	use xcm::latest::{Junctions, Location, NetworkId, WeightLimit, Xcm};
	use xcm::prelude::{
		AccountKey20, Asset, AssetFilter, AssetId, BuyExecution, ClearOrigin, DepositAsset,
		DescendOrigin, Fungible, GlobalConsensus, PalletInstance, Parachain, ReserveAssetDeposited,
		SetTopic, UniversalOrigin, XCM_VERSION,
	};
	use xcm::v5::WildAsset;
	use xcm::{VersionedAssets, VersionedInteriorLocation, VersionedLocation, VersionedXcm};
	use xcm_builder::BridgeMessage;

	fn next_block() {
		System::reset_events();
		System::set_block_number(System::block_number() + 1u32);
		System::on_initialize(System::block_number());
		MessageQueue::on_initialize(System::block_number());
	}

	#[test]
	fn transfer_asset_moonbeam_to_moonriver() {
		frame_support::__private::sp_tracing::init_for_tests();

		ExtBuilder::default()
			.with_balances(vec![
				(AccountId::from(ALICE), 2_000 * GLMR),
				(AccountId::from(BOB), 1_000 * GLMR),
			])
			.with_safe_xcm_version(XCM_VERSION)
			.with_open_bridges(vec![(
				Location::new(
					1,
					[Parachain(
						<bp_moonbeam::Moonbeam as bp_runtime::Parachain>::PARACHAIN_ID,
					)],
				),
				Junctions::from([
					NetworkId::Kusama.into(),
					Parachain(<bp_moonriver::Moonriver as bp_runtime::Parachain>::PARACHAIN_ID),
				]),
				Some(bp_moonbeam::LaneId::from_inner(H256([0u8; 32])))
			)])
			.build()
			.execute_with(|| {
				assert_ok!(PolkadotXcm::force_xcm_version(
					root_origin(),
					Box::new(bp_moonriver::GlobalConsensusLocation::get()),
					XCM_VERSION
				));

				let asset = currency_to_asset(CurrencyId::SelfReserve, 100 * GLMR);

				let message_data = BridgeKusamaMessages::outbound_message_data(
					bp_moonriver::LaneId::from_inner(H256([0u8; 32])),
					1u64,
				);
				assert!(message_data.is_none());

				assert_ok!(PolkadotXcm::transfer_assets(
					origin_of(AccountId::from(ALICE)),
					Box::new(VersionedLocation::V5(bp_moonriver::GlobalConsensusLocation::get())),
					Box::new(VersionedLocation::V5(Location {
						parents: 0,
						interior: [AccountKey20 {
							network: None,
							key: ALICE,
						}]
						.into(),
					})),
					Box::new(VersionedAssets::V5(asset.into())),
					0,
					WeightLimit::Unlimited
				));

				let message_data = BridgeKusamaMessages::outbound_message_data(
					bp_moonriver::LaneId::from_inner(H256([0u8; 32])),
					1u64,
				).unwrap();
				let decoded: StoredMessagePayload::<Runtime, WithKusamaMessagesInstance> = Decode::decode(&mut &message_data[..]).unwrap();

				let BridgeMessage { universal_dest, message } =
					Decode::decode(&mut &decoded[..]).unwrap();

				let expected_universal_dest: VersionedInteriorLocation = bp_moonriver::GlobalConsensusLocation::get().interior.into();
				assert_eq!(universal_dest, expected_universal_dest);

				assert_eq!(
					message,
					VersionedXcm::V5(
						Xcm(
							[
								UniversalOrigin(GlobalConsensus(NetworkId::Polkadot)),
								DescendOrigin(X1([Parachain(<bp_moonbeam::Moonbeam as bp_runtime::Parachain>::PARACHAIN_ID)].into())),
								ReserveAssetDeposited(
									vec![
										Asset {
											id: AssetId(
												Location::new(
													2,
													[
														GlobalConsensus(NetworkId::Polkadot),
														Parachain(<bp_moonbeam::Moonbeam as bp_runtime::Parachain>::PARACHAIN_ID),
														PalletInstance(<Balances as PalletInfoAccess>::index() as u8)
													]
												)
											),
											fun: Fungible(100_000_000_000_000_000_000)
										}
									].into()
								),
								ClearOrigin,
								BuyExecution {
									fees: Asset {
										id: AssetId(
											Location::new(
												2,
												[
													GlobalConsensus(NetworkId::Polkadot),
													Parachain(<bp_moonbeam::Moonbeam as bp_runtime::Parachain>::PARACHAIN_ID),
													PalletInstance(<Balances as PalletInfoAccess>::index() as u8)
												]
											)
										),
										fun: Fungible(100_000_000_000_000_000_000)
									},
									weight_limit: WeightLimit::Unlimited
								},
								DepositAsset {
									assets: AssetFilter::Wild(WildAsset::AllCounted(1)),
									beneficiary: Location::new(0, [AccountKey20 { network: None, key: ALICE}]),
								},
								SetTopic([24, 73, 92, 41, 231, 15, 196, 44, 136, 120, 145, 143, 224, 187, 112, 187, 47, 89, 154, 44, 193, 175, 174, 249, 30, 194, 97, 183, 171, 39, 87, 147])
							].into()
						)
					)
				);
			})
	}

	#[test]
	fn receive_message_from_moonriver() {
		frame_support::__private::sp_tracing::init_for_tests();

		ExtBuilder::default()
			.with_balances(vec![
				(AccountId::from(ALICE), 2_000 * GLMR),
				(AccountId::from(BOB), 1_000 * GLMR),
			])
			.with_xcm_assets(vec![XcmAssetInitialization {
				asset_id: 1,
				xcm_location: Location::new(
					2,
					[
						GlobalConsensus(KusamaGlobalConsensusNetwork::get()),
						Parachain(<bp_moonriver::Moonriver as bp_runtime::Parachain>::PARACHAIN_ID),
						PalletInstance(<Balances as PalletInfoAccess>::index() as u8)
					]
				),
				name: "xcMOVR",
				symbol: "xcMOVR",
				decimals: 18,
				balances: vec![(AccountId::from(ALICE), 1_000_000_000_000_000)],
			}])
			.with_safe_xcm_version(XCM_VERSION)
			.with_open_bridges(vec![(
				Location::new(
					1,
					[Parachain(
						<bp_moonbeam::Moonbeam as bp_runtime::Parachain>::PARACHAIN_ID,
					)],
				),
				Junctions::from([
					NetworkId::Kusama.into(),
					Parachain(<bp_moonriver::Moonriver as bp_runtime::Parachain>::PARACHAIN_ID),
				]),
				Some(bp_moonriver::LaneId::from_inner(H256([0u8; 32]))),
			)])
			.build()
			.execute_with(|| {
				assert_ok!(PolkadotXcm::force_xcm_version(
					root_origin(),
					Box::new(bp_moonriver::GlobalConsensusLocation::get()),
					XCM_VERSION
				));

				let bridge_message: BridgeMessage = BridgeMessage {
					universal_dest: VersionedInteriorLocation::V5(
						[
							GlobalConsensus(NetworkId::Polkadot),
							Parachain(<bp_moonbeam::Moonbeam as bp_runtime::Parachain>::PARACHAIN_ID)
						].into()
					),
					message: VersionedXcm::V5(
						Xcm(
							[
								UniversalOrigin(GlobalConsensus(NetworkId::Kusama)),
								DescendOrigin(X1([Parachain(<bp_moonriver::Moonriver as bp_runtime::Parachain>::PARACHAIN_ID)].into())),
								ReserveAssetDeposited(
									vec![
										Asset {
											id: AssetId(
												Location::new(
													2,
													[
														GlobalConsensus(NetworkId::Kusama),
														Parachain(<bp_moonriver::Moonriver as bp_runtime::Parachain>::PARACHAIN_ID),
														PalletInstance(<Balances as PalletInfoAccess>::index() as u8)
													]
												)
											),
											fun: Fungible(10_000_000_000_000_000_000_000_000_000_000_000)
										}
									].into()
								),
								ClearOrigin,
								BuyExecution {
									fees: Asset {
										id: AssetId(
											Location::new(
												2,
												[
													GlobalConsensus(NetworkId::Kusama),
													Parachain(<bp_moonriver::Moonriver as bp_runtime::Parachain>::PARACHAIN_ID),
													PalletInstance(<Balances as PalletInfoAccess>::index() as u8)
												]
											)
										),
										fun: Fungible(6_000_000_000_000_000_000_000_000_000_000_000)
									},
									weight_limit: WeightLimit::Unlimited
								},
								DepositAsset {
									assets: AssetFilter::Wild(WildAsset::AllCounted(1)),
									beneficiary: Location::new(0, [AccountKey20 { network: None, key: ALICE }]),
								},
								SetTopic([24, 73, 92, 41, 231, 15, 196, 44, 136, 120, 145, 143, 224, 187, 112, 187, 47, 89, 154, 44, 193, 175, 174, 249, 30, 194, 97, 183, 171, 39, 87, 147])
							].into()
						)
					)
				};

				let mut inbound_lane = LanesManager::<Runtime, WithKusamaMessagesInstance>::new()
					.active_inbound_lane(Default::default())
					.unwrap();

				let msg = DispatchMessageData { payload: Ok(bridge_message.encode()) };
	 			let result = inbound_lane.receive_message::<BridgeXcmOverMoonriver>(&AccountId::from(ALICE),
					1,
					msg,
				);

				assert_eq!(result, ReceptionResult::Dispatched(MessageDispatchResult { unspent_weight: Default::default(), dispatch_level_result: Dispatched }));

				// Produce next block
				next_block();
				// Confirm that the xcm message was successfully processed
				assert!(System::events().iter().any(|evt| {
					matches!(
						evt.event,
						RuntimeEvent::MessageQueue(
							pallet_message_queue::Event::Processed {
								origin: AggregateMessageOrigin::Here,
								success: true,
								..
							}
						)
					)
				}));
			});
	}
}

#[cfg(test)]
mod treasury_tests {
	use super::*;
	use frame_support::traits::fungible::NativeOrWithId;
	use moonbeam_runtime::XcmWeightTrader;
	use sp_core::bounded_vec;
	use sp_runtime::traits::Hash;

	fn expect_events(events: Vec<RuntimeEvent>) {
		let block_events: Vec<RuntimeEvent> =
			System::events().into_iter().map(|r| r.event).collect();

		assert!(events.iter().all(|evt| block_events.contains(evt)))
	}

	fn next_block() {
		System::reset_events();
		System::set_block_number(System::block_number() + 1u32);
		System::on_initialize(System::block_number());
		Treasury::on_initialize(System::block_number());
	}

	fn get_asset_balance(id: &u128, account: &AccountId) -> U256 {
		pallet_moonbeam_foreign_assets::Pallet::<Runtime>::balance(id.clone(), account.clone())
			.expect("failed to get account balance")
	}

	#[test]
	fn test_treasury_spend_local_with_council_origin() {
		let initial_treasury_balance = 1_000 * GLMR;
		ExtBuilder::default()
			.with_balances(vec![
				(AccountId::from(ALICE), 2_000 * GLMR),
				(Treasury::account_id(), initial_treasury_balance),
			])
			.build()
			.execute_with(|| {
				let spend_amount = 100u128 * GLMR;
				let spend_beneficiary = AccountId::from(BOB);

				next_block();

				// TreasuryCouncilCollective
				assert_ok!(TreasuryCouncilCollective::set_members(
					root_origin(),
					vec![AccountId::from(ALICE)],
					Some(AccountId::from(ALICE)),
					1
				));

				next_block();

				// Perform treasury spending
				let valid_from = System::block_number() + 5u32;
				let proposal = RuntimeCall::Treasury(pallet_treasury::Call::spend {
					amount: spend_amount,
					asset_kind: Box::new(NativeOrWithId::Native),
					beneficiary: Box::new(AccountId::from(BOB)),
					valid_from: Some(valid_from),
				});
				assert_ok!(TreasuryCouncilCollective::propose(
					origin_of(AccountId::from(ALICE)),
					1,
					Box::new(proposal.clone()),
					1_000
				));

				let payout_period =
					<<Runtime as pallet_treasury::Config>::PayoutPeriod as Get<u32>>::get();
				let expected_events = [
					RuntimeEvent::Treasury(pallet_treasury::Event::AssetSpendApproved {
						index: 0,
						asset_kind: NativeOrWithId::Native,
						amount: spend_amount,
						beneficiary: spend_beneficiary,
						valid_from,
						expire_at: payout_period + valid_from,
					}),
					RuntimeEvent::TreasuryCouncilCollective(pallet_collective::Event::Executed {
						proposal_hash: sp_runtime::traits::BlakeTwo256::hash_of(&proposal),
						result: Ok(()),
					}),
				]
				.to_vec();
				expect_events(expected_events);

				while System::block_number() < valid_from {
					next_block();
				}

				assert_ok!(Treasury::payout(origin_of(spend_beneficiary), 0));

				let expected_events = [
					RuntimeEvent::Treasury(pallet_treasury::Event::Paid {
						index: 0,
						payment_id: (),
					}),
					RuntimeEvent::Balances(pallet_balances::Event::Transfer {
						from: Treasury::account_id(),
						to: spend_beneficiary,
						amount: spend_amount,
					}),
				]
				.to_vec();
				expect_events(expected_events);
			});
	}

	#[test]
	fn test_treasury_spend_foreign_asset_with_council_origin() {
		let initial_treasury_balance = 1_000 * GLMR;
		let asset_id = 1000100010001000u128;
		ExtBuilder::default()
			.with_balances(vec![(AccountId::from(ALICE), 2_000 * GLMR)])
			.build()
			.execute_with(|| {
				let spend_amount = 100u128 * GLMR;
				let spend_beneficiary = AccountId::from(BOB);

				let asset_location: Location = Location {
					parents: 1,
					interior: Junctions::Here,
				};

				assert_ok!(EvmForeignAssets::create_foreign_asset(
					root_origin(),
					asset_id,
					asset_location.clone(),
					12,
					bounded_vec![b'M', b'T'],
					bounded_vec![b'M', b'y', b'T', b'o', b'k'],
				));

				assert_ok!(XcmWeightTrader::add_asset(
					root_origin(),
					asset_location,
					1u128
				));

				assert_ok!(EvmForeignAssets::mint_into(
					asset_id,
					Treasury::account_id(),
					initial_treasury_balance.into()
				));

				assert_eq!(
					get_asset_balance(&asset_id, &Treasury::account_id()),
					initial_treasury_balance.into(),
					"Treasury balance not updated"
				);

				// TreasuryCouncilCollective
				assert_ok!(TreasuryCouncilCollective::set_members(
					root_origin(),
					vec![AccountId::from(ALICE)],
					Some(AccountId::from(ALICE)),
					1
				));

				// Perform treasury spending
				let proposal = RuntimeCall::Treasury(pallet_treasury::Call::spend {
					amount: spend_amount,
					asset_kind: Box::new(NativeOrWithId::WithId(asset_id)),
					beneficiary: Box::new(spend_beneficiary),
					valid_from: None,
				});
				assert_ok!(TreasuryCouncilCollective::propose(
					origin_of(AccountId::from(ALICE)),
					1,
					Box::new(proposal.clone()),
					1_000
				));

				let payout_period =
					<<Runtime as pallet_treasury::Config>::PayoutPeriod as Get<u32>>::get();

				let current_block = System::block_number();
				let expected_events = [
					RuntimeEvent::Treasury(pallet_treasury::Event::AssetSpendApproved {
						index: 0,
						asset_kind: NativeOrWithId::WithId(asset_id),
						amount: spend_amount,
						beneficiary: spend_beneficiary,
						valid_from: current_block,
						expire_at: current_block + payout_period,
					}),
					RuntimeEvent::TreasuryCouncilCollective(pallet_collective::Event::Executed {
						proposal_hash: sp_runtime::traits::BlakeTwo256::hash_of(&proposal),
						result: Ok(()),
					}),
				]
				.to_vec();
				expect_events(expected_events);

				assert_ok!(Treasury::payout(origin_of(spend_beneficiary), 0));

				expect_events(vec![RuntimeEvent::Treasury(pallet_treasury::Event::Paid {
					index: 0,
					payment_id: (),
				})]);

				assert_eq!(
					get_asset_balance(&asset_id, &Treasury::account_id()),
					(initial_treasury_balance - spend_amount).into(),
					"Treasury balance not updated"
				);

				assert_eq!(
					get_asset_balance(&asset_id, &spend_beneficiary),
					spend_amount.into(),
					"Treasury payout failed"
				);
			});
	}
}

#[cfg(test)]
mod fee_tests {
	use super::*;
	use fp_evm::FeeCalculator;
	use frame_support::{
		traits::{ConstU128, OnFinalize},
		weights::{ConstantMultiplier, WeightToFee},
	};
	use moonbeam_runtime::{
		currency, LengthToFee, MinimumMultiplier, RuntimeBlockWeights, SlowAdjustingFeeUpdate,
		TargetBlockFullness, TransactionPaymentAsGasPrice, NORMAL_WEIGHT, WEIGHT_PER_GAS,
	};
	use sp_core::Get;
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
			* RuntimeBlockWeights::get()
				.get(DispatchClass::Normal)
				.max_total
				.unwrap();
		// if the min is too small, then this will not change, and we are doomed forever.
		// the weight is 1/100th bigger than target.
		run_with_system_weight(target * 101 / 100, || {
			let next = SlowAdjustingFeeUpdate::<Runtime>::convert(minimum_multiplier);
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
		let base_extrinsic = RuntimeBlockWeights::get()
			.get(DispatchClass::Normal)
			.base_extrinsic;
		let multiplier = sp_runtime::FixedU128::from_float(0.999000000000000000);
		let extrinsic_len = 100u32;
		let extrinsic_weight = 5_000u64;
		let tip = 42u128;
		type WeightToFeeImpl = ConstantMultiplier<u128, ConstU128<{ currency::WEIGHT_FEE }>>;
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
					call_weight: Weight::from_parts(extrinsic_weight, 1),
					extension_weight: Weight::zero(),
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
			let weight_fee_per_gas = (currency::WEIGHT_FEE).saturating_mul(WEIGHT_PER_GAS as u128);
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
				U256::from(31_250_000_000u128), // lower bound enforced
			);
			assert_eq!(
				sim(1_000_000_000, Perbill::from_percent(25), 1),
				U256::from(31_250_000_000u128),
			);
			assert_eq!(
				sim(1_000_000_000, Perbill::from_percent(50), 1),
				U256::from(31_268_755_625u128), // slightly higher than lower bound
			);
			assert_eq!(
				sim(1_000_000_000, Perbill::from_percent(100), 1),
				U256::from(31_331_355_625u128), // a bit higher than before
			);

			// 1 "real" hour (at 12-second blocks)
			assert_eq!(
				sim(1_000_000_000, Perbill::from_percent(0), 600),
				U256::from(31_250_000_000u128), // lower bound enforced
			);
			assert_eq!(
				sim(1_000_000_000, Perbill::from_percent(25), 600),
				U256::from(31_250_000_000u128),
			);
			assert_eq!(
				sim(1_000_000_000, Perbill::from_percent(50), 600),
				U256::from(44_791_543_237u128), // a bit higher than lower bound
			);
			assert_eq!(
				sim(1_000_000_000, Perbill::from_percent(100), 600),
				U256::from(148_712_903_041u128), // a lot more
			);

			// 1 "real" day (at 12-second blocks)
			assert_eq!(
				sim(1_000_000_000, Perbill::from_percent(0), 14400),
				U256::from(31_250_000_000u128), // lower bound enforced
			);
			assert_eq!(
				sim(1_000_000_000, Perbill::from_percent(25), 14400),
				U256::from(31_250_000_000u128),
			);
			assert_eq!(
				sim(1_000_000_000, Perbill::from_percent(50), 14400),
				U256::from(176_666_465_470_908u128), // significantly higher
			);
			assert_eq!(
				sim(1_000_000_000, Perbill::from_percent(100), 14400),
				U256::from(3_125_000_000_000_000u128), // upper bound enforced
			);
		});
	}
}

#[cfg(test)]
mod balance_tests {
	use crate::common::{ExtBuilder, ALICE};
	use frame_support::assert_ok;
	use frame_support::traits::LockableCurrency;
	use frame_support::traits::{LockIdentifier, ReservableCurrency, WithdrawReasons};
	use moonbeam_core_primitives::AccountId;
	use moonbeam_runtime::{Balances, Runtime, System};

	#[test]
	fn reserve_should_work_for_frozen_balance() {
		let alice = AccountId::from(ALICE);
		const ID_1: LockIdentifier = *b"1       ";

		ExtBuilder::default()
			.with_balances(vec![(alice, 10)])
			.build()
			.execute_with(|| {
				// Check balances
				let account = System::account(&alice).data;
				assert_eq!(account.free, 10);
				assert_eq!(account.frozen, 0);
				assert_eq!(account.reserved, 0);

				Balances::set_lock(ID_1, &alice, 9, WithdrawReasons::RESERVE);

				let account = System::account(&alice).data;
				assert_eq!(account.free, 10);
				assert_eq!(account.frozen, 9);
				assert_eq!(account.reserved, 0);

				assert_ok!(Balances::reserve(&alice, 5));

				let account = System::account(&alice).data;
				assert_eq!(account.free, 5);
				assert_eq!(account.frozen, 9);
				assert_eq!(account.reserved, 5);

				let previous_reserved_amount = account.reserved;
				let ed: u128 = <Runtime as pallet_balances::Config>::ExistentialDeposit::get();
				let next_reserve = account.free.saturating_sub(ed);
				assert_ok!(Balances::reserve(&alice, next_reserve));

				let account = System::account(&alice).data;
				assert_eq!(account.free, ed);
				assert_eq!(account.frozen, 9);
				assert_eq!(
					account.reserved,
					previous_reserved_amount.saturating_add(next_reserve)
				);
			});
	}
}

moonbeam_runtime_common::generate_common_xcm_tests!(moonbeam_runtime);
