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

//! Moonriver Runtime Integration Tests

#![cfg(test)]

mod common;
use common::*;

use fp_evm::Context;
use frame_support::{
	assert_noop, assert_ok,
	dispatch::Dispatchable,
	traits::{
		fungible::Inspect, fungibles::Inspect as FungiblesInspect, Currency as CurrencyT,
		EnsureOrigin, PalletInfo, StorageInfo, StorageInfoTrait,
	},
	weights::{DispatchClass, Weight},
	StorageHasher, Twox128,
};
use moonriver_runtime::{
	asset_config::LocalAssetInstance, xcm_config::CurrencyId, AssetId, BaseFee, BlockWeights,
	LocalAssets, PolkadotXcm, Precompiles, XTokens, XcmTransactor,
	FOREIGN_ASSET_PRECOMPILE_ADDRESS_PREFIX, LOCAL_ASSET_PRECOMPILE_ADDRESS_PREFIX,
};
use nimbus_primitives::NimbusId;
use pallet_evm::PrecompileSet;
use pallet_evm_precompile_batch::Action as BatchAction;
use pallet_evm_precompile_crowdloan_rewards::Action as CrowdloanAction;
use pallet_evm_precompile_xcm_transactor::v1::Action as XcmTransactorActionV1;
use pallet_evm_precompile_xcm_utils::Action as XcmUtilsAction;
use pallet_evm_precompile_xtokens::Action as XtokensAction;
use pallet_evm_precompileset_assets_erc20::{
	AccountIdAssetIdConversion, Action as AssetAction, SELECTOR_LOG_APPROVAL, SELECTOR_LOG_TRANSFER,
};
use pallet_transaction_payment::Multiplier;
use pallet_xcm_transactor::{Currency, CurrencyPayment, TransactWeights};
use parity_scale_codec::Encode;
use polkadot_parachain::primitives::Sibling;
use precompile_utils::{prelude::*, testing::*};
use sha3::{Digest, Keccak256};
use sp_core::{ByteArray, Pair, H160, U256};
use sp_runtime::{
	traits::{Convert, One},
	DispatchError, ModuleError, TokenError,
};
use std::str::from_utf8;
use xcm::latest::prelude::*;
use xcm::{VersionedMultiAssets, VersionedMultiLocation};
use xcm_builder::{ParentIsPreset, SiblingParachainConvertsVia};
use xcm_executor::traits::Convert as XcmConvert;

#[test]
fn xcmp_queue_controller_origin_is_root() {
	// important for the XcmExecutionManager impl of PauseExecution which uses root origin
	// to suspend/resume XCM execution in xcmp_queue::on_idle
	assert_ok!(
		<moonriver_runtime::Runtime as cumulus_pallet_xcmp_queue::Config
		>::ControllerOrigin::ensure_origin(root_origin())
	);
}

#[test]
fn fast_track_available() {
	assert!(moonriver_runtime::get!(
		pallet_democracy,
		InstantAllowed,
		bool
	));
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
			<moonriver_runtime::Runtime as frame_system::Config>::PalletInfo::name::<P>(),
			Some(name)
		);
	}
	// TODO: use StorageInfoTrait from https://github.com/paritytech/substrate/pull/9246
	// This is now available with polkadot-v0.9.9 dependencies
	is_pallet_prefix::<moonriver_runtime::System>("System");
	is_pallet_prefix::<moonriver_runtime::Utility>("Utility");
	is_pallet_prefix::<moonriver_runtime::RandomnessCollectiveFlip>("RandomnessCollectiveFlip");
	is_pallet_prefix::<moonriver_runtime::ParachainSystem>("ParachainSystem");
	is_pallet_prefix::<moonriver_runtime::TransactionPayment>("TransactionPayment");
	is_pallet_prefix::<moonriver_runtime::ParachainInfo>("ParachainInfo");
	is_pallet_prefix::<moonriver_runtime::EthereumChainId>("EthereumChainId");
	is_pallet_prefix::<moonriver_runtime::EVM>("EVM");
	is_pallet_prefix::<moonriver_runtime::Ethereum>("Ethereum");
	is_pallet_prefix::<moonriver_runtime::ParachainStaking>("ParachainStaking");
	is_pallet_prefix::<moonriver_runtime::MaintenanceMode>("MaintenanceMode");
	is_pallet_prefix::<moonriver_runtime::Scheduler>("Scheduler");
	is_pallet_prefix::<moonriver_runtime::Democracy>("Democracy");
	is_pallet_prefix::<moonriver_runtime::CouncilCollective>("CouncilCollective");
	is_pallet_prefix::<moonriver_runtime::TechCommitteeCollective>("TechCommitteeCollective");
	is_pallet_prefix::<moonriver_runtime::Treasury>("Treasury");
	is_pallet_prefix::<moonriver_runtime::AuthorInherent>("AuthorInherent");
	is_pallet_prefix::<moonriver_runtime::AuthorFilter>("AuthorFilter");
	is_pallet_prefix::<moonriver_runtime::CrowdloanRewards>("CrowdloanRewards");
	is_pallet_prefix::<moonriver_runtime::AuthorMapping>("AuthorMapping");
	is_pallet_prefix::<moonriver_runtime::Identity>("Identity");
	is_pallet_prefix::<moonriver_runtime::XcmpQueue>("XcmpQueue");
	is_pallet_prefix::<moonriver_runtime::CumulusXcm>("CumulusXcm");
	is_pallet_prefix::<moonriver_runtime::DmpQueue>("DmpQueue");
	is_pallet_prefix::<moonriver_runtime::PolkadotXcm>("PolkadotXcm");
	is_pallet_prefix::<moonriver_runtime::Assets>("Assets");
	is_pallet_prefix::<moonriver_runtime::XTokens>("XTokens");
	is_pallet_prefix::<moonriver_runtime::AssetManager>("AssetManager");
	is_pallet_prefix::<moonriver_runtime::Migrations>("Migrations");
	is_pallet_prefix::<moonriver_runtime::XcmTransactor>("XcmTransactor");
	is_pallet_prefix::<moonriver_runtime::ProxyGenesisCompanion>("ProxyGenesisCompanion");
	is_pallet_prefix::<moonriver_runtime::BaseFee>("BaseFee");
	is_pallet_prefix::<moonriver_runtime::LocalAssets>("LocalAssets");
	is_pallet_prefix::<moonriver_runtime::MoonbeamOrbiters>("MoonbeamOrbiters");
	is_pallet_prefix::<moonriver_runtime::TreasuryCouncilCollective>("TreasuryCouncilCollective");

	let prefix = |pallet_name, storage_name| {
		let mut res = [0u8; 32];
		res[0..16].copy_from_slice(&Twox128::hash(pallet_name));
		res[16..32].copy_from_slice(&Twox128::hash(storage_name));
		res.to_vec()
	};
	assert_eq!(
		<moonriver_runtime::Timestamp as StorageInfoTrait>::storage_info(),
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
		<moonriver_runtime::Balances as StorageInfoTrait>::storage_info(),
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
				storage_name: b"StorageVersion".to_vec(),
				prefix: prefix(b"Balances", b"StorageVersion"),
				max_values: Some(1),
				max_size: Some(1),
			}
		]
	);
	assert_eq!(
		<moonriver_runtime::Proxy as StorageInfoTrait>::storage_info(),
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
		<moonriver_runtime::MaintenanceMode as StorageInfoTrait>::storage_info(),
		vec![StorageInfo {
			pallet_name: b"MaintenanceMode".to_vec(),
			storage_name: b"MaintenanceMode".to_vec(),
			prefix: prefix(b"MaintenanceMode", b"MaintenanceMode"),
			max_values: Some(1),
			max_size: None,
		},]
	);
}

#[test]
fn test_collectives_storage_item_prefixes() {
	for StorageInfo { pallet_name, .. } in
		<moonriver_runtime::CouncilCollective as StorageInfoTrait>::storage_info()
	{
		assert_eq!(pallet_name, b"CouncilCollective".to_vec());
	}

	for StorageInfo { pallet_name, .. } in
		<moonriver_runtime::TechCommitteeCollective as StorageInfoTrait>::storage_info()
	{
		assert_eq!(pallet_name, b"TechCommitteeCollective".to_vec());
	}
}

#[test]
fn verify_pallet_indices() {
	fn is_pallet_index<P: 'static>(index: usize) {
		assert_eq!(
			<moonriver_runtime::Runtime as frame_system::Config>::PalletInfo::index::<P>(),
			Some(index)
		);
	}
	// System support
	is_pallet_index::<moonriver_runtime::System>(0);
	is_pallet_index::<moonriver_runtime::ParachainSystem>(1);
	is_pallet_index::<moonriver_runtime::RandomnessCollectiveFlip>(2);
	is_pallet_index::<moonriver_runtime::Timestamp>(3);
	is_pallet_index::<moonriver_runtime::ParachainInfo>(4);
	// Monetary
	is_pallet_index::<moonriver_runtime::Balances>(10);
	is_pallet_index::<moonriver_runtime::TransactionPayment>(11);
	// Consensus support
	is_pallet_index::<moonriver_runtime::ParachainStaking>(20);
	is_pallet_index::<moonriver_runtime::AuthorInherent>(21);
	is_pallet_index::<moonriver_runtime::AuthorFilter>(22);
	is_pallet_index::<moonriver_runtime::AuthorMapping>(23);
	is_pallet_index::<moonriver_runtime::MoonbeamOrbiters>(24);
	// Handy utilities
	is_pallet_index::<moonriver_runtime::Utility>(30);
	is_pallet_index::<moonriver_runtime::Proxy>(31);
	is_pallet_index::<moonriver_runtime::MaintenanceMode>(32);
	is_pallet_index::<moonriver_runtime::Identity>(33);
	is_pallet_index::<moonriver_runtime::Migrations>(34);
	is_pallet_index::<moonriver_runtime::ProxyGenesisCompanion>(35);
	// TODO Sudo was previously index 40, should we test that there is nothing there now?
	// Ethereum compatibility
	is_pallet_index::<moonriver_runtime::EthereumChainId>(50);
	is_pallet_index::<moonriver_runtime::EVM>(51);
	is_pallet_index::<moonriver_runtime::Ethereum>(52);
	is_pallet_index::<moonriver_runtime::BaseFee>(53);
	// Governance
	is_pallet_index::<moonriver_runtime::Scheduler>(60);
	is_pallet_index::<moonriver_runtime::Democracy>(61);
	// Council
	is_pallet_index::<moonriver_runtime::CouncilCollective>(70);
	is_pallet_index::<moonriver_runtime::TechCommitteeCollective>(71);
	is_pallet_index::<moonriver_runtime::TreasuryCouncilCollective>(72);
	// Treasury
	is_pallet_index::<moonriver_runtime::Treasury>(80);
	// Crowdloan
	is_pallet_index::<moonriver_runtime::CrowdloanRewards>(90);
	// XCM Stuff
	is_pallet_index::<moonriver_runtime::XcmpQueue>(100);
	is_pallet_index::<moonriver_runtime::CumulusXcm>(101);
	is_pallet_index::<moonriver_runtime::DmpQueue>(102);
	is_pallet_index::<moonriver_runtime::PolkadotXcm>(103);
	is_pallet_index::<moonriver_runtime::Assets>(104);
	is_pallet_index::<moonriver_runtime::AssetManager>(105);
	is_pallet_index::<moonriver_runtime::XTokens>(106);
	is_pallet_index::<moonriver_runtime::XcmTransactor>(107);
	is_pallet_index::<moonriver_runtime::LocalAssets>(108);
}

#[test]
fn verify_proxy_type_indices() {
	assert_eq!(moonriver_runtime::ProxyType::Any as u8, 0);
	assert_eq!(moonriver_runtime::ProxyType::NonTransfer as u8, 1);
	assert_eq!(moonriver_runtime::ProxyType::Governance as u8, 2);
	assert_eq!(moonriver_runtime::ProxyType::Staking as u8, 3);
	assert_eq!(moonriver_runtime::ProxyType::CancelProxy as u8, 4);
	assert_eq!(moonriver_runtime::ProxyType::Balances as u8, 5);
	assert_eq!(moonriver_runtime::ProxyType::AuthorMapping as u8, 6);
	assert_eq!(moonriver_runtime::ProxyType::IdentityJudgement as u8, 7);
}

#[test]
fn join_collator_candidates() {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * MOVR),
			(AccountId::from(BOB), 2_000 * MOVR),
			(AccountId::from(CHARLIE), 1_100 * MOVR),
			(AccountId::from(DAVE), 1_000 * MOVR),
		])
		.with_collators(vec![
			(AccountId::from(ALICE), 1_000 * MOVR),
			(AccountId::from(BOB), 1_000 * MOVR),
		])
		.with_delegations(vec![
			(AccountId::from(CHARLIE), AccountId::from(ALICE), 50 * MOVR),
			(AccountId::from(CHARLIE), AccountId::from(BOB), 50 * MOVR),
		])
		.build()
		.execute_with(|| {
			assert_noop!(
				ParachainStaking::join_candidates(
					origin_of(AccountId::from(ALICE)),
					1_000 * MOVR,
					2u32
				),
				pallet_parachain_staking::Error::<Runtime>::CandidateExists
			);
			assert_noop!(
				ParachainStaking::join_candidates(
					origin_of(AccountId::from(CHARLIE)),
					1_000 * MOVR,
					2u32
				),
				pallet_parachain_staking::Error::<Runtime>::DelegatorExists
			);
			assert!(System::events().is_empty());
			assert_ok!(ParachainStaking::join_candidates(
				origin_of(AccountId::from(DAVE)),
				1_000 * MOVR,
				2u32
			));
			assert_eq!(
				last_event(),
				Event::ParachainStaking(
					pallet_parachain_staking::Event::JoinedCollatorCandidates {
						account: AccountId::from(DAVE),
						amount_locked: 1_000 * MOVR,
						new_total_amt_locked: 3_100 * MOVR
					}
				)
			);
			let candidates = ParachainStaking::candidate_pool();
			assert_eq!(candidates.0[0].owner, AccountId::from(ALICE));
			assert_eq!(candidates.0[0].amount, 1_050 * MOVR);
			assert_eq!(candidates.0[1].owner, AccountId::from(BOB));
			assert_eq!(candidates.0[1].amount, 1_050 * MOVR);
			assert_eq!(candidates.0[2].owner, AccountId::from(DAVE));
			assert_eq!(candidates.0[2].amount, 1_000 * MOVR);
		});
}

#[test]
fn transfer_through_evm_to_stake() {
	ExtBuilder::default()
		.with_balances(vec![(AccountId::from(ALICE), 2_000 * MOVR)])
		.build()
		.execute_with(|| {
			// Charlie has no balance => fails to stake
			assert_noop!(
				ParachainStaking::join_candidates(
					origin_of(AccountId::from(CHARLIE)),
					1_000 * MOVR,
					2u32
				),
				DispatchError::Module(ModuleError {
					index: 20,
					error: [8, 0, 0, 0],
					message: Some("InsufficientBalance")
				})
			);
			// Alice transfer from free balance 2000 MOVR to Bob
			assert_ok!(Balances::transfer(
				origin_of(AccountId::from(ALICE)),
				AccountId::from(BOB),
				2_000 * MOVR,
			));
			assert_eq!(Balances::free_balance(AccountId::from(BOB)), 2_000 * MOVR);

			let gas_limit = 100000u64;
			let gas_price: U256 = 1_000_000_000u64.into();
			// Bob transfers 1000 MOVR to Charlie via EVM
			assert_ok!(Call::EVM(pallet_evm::Call::<Runtime>::call {
				source: H160::from(BOB),
				target: H160::from(CHARLIE),
				input: vec![],
				value: (1_000 * MOVR).into(),
				gas_limit,
				max_fee_per_gas: gas_price,
				max_priority_fee_per_gas: None,
				nonce: None,
				access_list: Vec::new(),
			})
			.dispatch(<Runtime as frame_system::Config>::Origin::root()));
			assert_eq!(
				Balances::free_balance(AccountId::from(CHARLIE)),
				1_000 * MOVR,
			);

			// Charlie can stake now
			assert_ok!(ParachainStaking::join_candidates(
				origin_of(AccountId::from(CHARLIE)),
				1_000 * MOVR,
				2u32,
			),);
			let candidates = ParachainStaking::candidate_pool();
			assert_eq!(candidates.0[0].owner, AccountId::from(CHARLIE));
			assert_eq!(candidates.0[0].amount, 1_000 * MOVR);
		});
}

#[test]
fn reward_block_authors() {
	ExtBuilder::default()
		.with_balances(vec![
			// Alice gets 100 extra tokens for her mapping deposit
			(AccountId::from(ALICE), 2_100 * MOVR),
			(AccountId::from(BOB), 1_000 * MOVR),
		])
		.with_collators(vec![(AccountId::from(ALICE), 1_000 * MOVR)])
		.with_delegations(vec![(
			AccountId::from(BOB),
			AccountId::from(ALICE),
			500 * MOVR,
		)])
		.with_mappings(vec![(
			NimbusId::from_slice(&ALICE_NIMBUS).unwrap(),
			AccountId::from(ALICE),
		)])
		.build()
		.execute_with(|| {
			set_parachain_inherent_data();
			for x in 2..1199 {
				run_to_block(x, Some(NimbusId::from_slice(&ALICE_NIMBUS).unwrap()));
			}
			// no rewards doled out yet
			assert_eq!(
				Balances::usable_balance(AccountId::from(ALICE)),
				1_000 * MOVR,
			);
			assert_eq!(Balances::usable_balance(AccountId::from(BOB)), 500 * MOVR,);
			run_to_block(1200, Some(NimbusId::from_slice(&ALICE_NIMBUS).unwrap()));
			// rewards minted and distributed
			assert_eq!(
				Balances::usable_balance(AccountId::from(ALICE)),
				1113666666584000000000,
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
			(AccountId::from(ALICE), 2_100 * MOVR),
			(AccountId::from(BOB), 1_000 * MOVR),
			(AccountId::from(CHARLIE), MOVR),
		])
		.with_collators(vec![(AccountId::from(ALICE), 1_000 * MOVR)])
		.with_delegations(vec![(
			AccountId::from(BOB),
			AccountId::from(ALICE),
			500 * MOVR,
		)])
		.with_mappings(vec![(
			NimbusId::from_slice(&ALICE_NIMBUS).unwrap(),
			AccountId::from(ALICE),
		)])
		.build()
		.execute_with(|| {
			set_parachain_inherent_data();
			assert_ok!(ParachainStaking::set_parachain_bond_account(
				root_origin(),
				AccountId::from(CHARLIE),
			),);
			for x in 2..1199 {
				run_to_block(x, Some(NimbusId::from_slice(&ALICE_NIMBUS).unwrap()));
			}
			// no rewards doled out yet
			assert_eq!(
				Balances::usable_balance(AccountId::from(ALICE)),
				1_000 * MOVR,
			);
			assert_eq!(Balances::usable_balance(AccountId::from(BOB)), 500 * MOVR,);
			assert_eq!(Balances::usable_balance(AccountId::from(CHARLIE)), MOVR,);
			run_to_block(1200, Some(NimbusId::from_slice(&ALICE_NIMBUS).unwrap()));
			// rewards minted and distributed
			assert_eq!(
				Balances::usable_balance(AccountId::from(ALICE)),
				1082693333281650000000,
			);
			assert_eq!(
				Balances::usable_balance(AccountId::from(BOB)),
				525841666640825000000,
			);
			// 30% reserved for parachain bond
			assert_eq!(
				Balances::usable_balance(AccountId::from(CHARLIE)),
				47515000000000000000,
			);
		});
}

#[test]
fn initialize_crowdloan_addresses_with_batch_and_pay() {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * MOVR),
			(AccountId::from(BOB), 1_000 * MOVR),
		])
		.with_collators(vec![(AccountId::from(ALICE), 1_000 * MOVR)])
		.with_mappings(vec![(
			NimbusId::from_slice(&ALICE_NIMBUS).unwrap(),
			AccountId::from(ALICE),
		)])
		.with_crowdloan_fund(3_000_000 * MOVR)
		.build()
		.execute_with(|| {
			// set parachain inherent data
			set_parachain_inherent_data();
			let init_block = CrowdloanRewards::init_vesting_block();
			// This matches the previous vesting
			let end_block = init_block + 48 * WEEKS;
			// Batch calls always succeed. We just need to check the inner event
			assert_ok!(Call::Utility(pallet_utility::Call::<Runtime>::batch_all {
				calls: vec![
					Call::CrowdloanRewards(
						pallet_crowdloan_rewards::Call::<Runtime>::initialize_reward_vec {
							rewards: vec![(
								[4u8; 32].into(),
								Some(AccountId::from(CHARLIE)),
								1_500_000 * MOVR
							)]
						}
					),
					Call::CrowdloanRewards(
						pallet_crowdloan_rewards::Call::<Runtime>::initialize_reward_vec {
							rewards: vec![(
								[5u8; 32].into(),
								Some(AccountId::from(DAVE)),
								1_500_000 * MOVR
							)]
						}
					),
					Call::CrowdloanRewards(
						pallet_crowdloan_rewards::Call::<Runtime>::complete_initialization {
							lease_ending_block: end_block
						}
					)
				]
			})
			.dispatch(root_origin()));
			// 30 percent initial payout
			assert_eq!(Balances::balance(&AccountId::from(CHARLIE)), 450_000 * MOVR);
			// 30 percent initial payout
			assert_eq!(Balances::balance(&AccountId::from(DAVE)), 450_000 * MOVR);
			let expected = Event::Utility(pallet_utility::Event::BatchCompleted);
			assert_eq!(last_event(), expected);
			// This one should fail, as we already filled our data
			assert_ok!(Call::Utility(pallet_utility::Call::<Runtime>::batch {
				calls: vec![Call::CrowdloanRewards(pallet_crowdloan_rewards::Call::<
					Runtime,
				>::initialize_reward_vec {
					rewards: vec![([4u8; 32].into(), Some(AccountId::from(ALICE)), 432000)]
				})]
			})
			.dispatch(root_origin()));
			let expected_fail = Event::Utility(pallet_utility::Event::BatchInterrupted {
				index: 0,
				error: DispatchError::Module(ModuleError {
					index: 90,
					error: [8, 0, 0, 0],
					message: None,
				}),
			});
			assert_eq!(last_event(), expected_fail);
			// Claim 1 block.
			assert_ok!(CrowdloanRewards::claim(origin_of(AccountId::from(CHARLIE))));
			assert_ok!(CrowdloanRewards::claim(origin_of(AccountId::from(DAVE))));

			let vesting_period = 48 * WEEKS as u128;
			let per_block = (1_050_000 * MOVR) / vesting_period;

			assert_eq!(
				CrowdloanRewards::accounts_payable(&AccountId::from(CHARLIE))
					.unwrap()
					.claimed_reward,
				(450_000 * MOVR) + per_block
			);
			assert_eq!(
				CrowdloanRewards::accounts_payable(&AccountId::from(DAVE))
					.unwrap()
					.claimed_reward,
				(450_000 * MOVR) + per_block
			);
			// The total claimed reward should be equal to the account balance at this point.
			assert_eq!(
				Balances::balance(&AccountId::from(CHARLIE)),
				(450_000 * MOVR) + per_block
			);
			assert_eq!(
				Balances::balance(&AccountId::from(DAVE)),
				(450_000 * MOVR) + per_block
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
			(AccountId::from(ALICE), 2_000 * MOVR),
			(AccountId::from(BOB), 1_000 * MOVR),
		])
		.with_collators(vec![(AccountId::from(ALICE), 1_000 * MOVR)])
		.with_mappings(vec![(
			NimbusId::from_slice(&ALICE_NIMBUS).unwrap(),
			AccountId::from(ALICE),
		)])
		.with_crowdloan_fund(3_000_000 * MOVR)
		.build()
		.execute_with(|| {
			// set parachain inherent data
			set_parachain_inherent_data();
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
			message.append(&mut b"moonriver-".to_vec());
			message.append(&mut AccountId::from(DAVE).encode());
			message.append(&mut AccountId::from(CHARLIE).encode());
			message.append(&mut pallet_crowdloan_rewards::WRAPPED_BYTES_POSTFIX.to_vec());
			let signature1 = pair1.sign(&message);
			let signature2 = pair2.sign(&message);

			// Batch calls always succeed. We just need to check the inner event
			assert_ok!(
				// two relay accounts pointing at the same reward account
				Call::Utility(pallet_utility::Call::<Runtime>::batch_all {
					calls: vec![
						Call::CrowdloanRewards(
							pallet_crowdloan_rewards::Call::<Runtime>::initialize_reward_vec {
								rewards: vec![(
									public1.into(),
									Some(AccountId::from(CHARLIE)),
									1_500_000 * MOVR
								)]
							}
						),
						Call::CrowdloanRewards(
							pallet_crowdloan_rewards::Call::<Runtime>::initialize_reward_vec {
								rewards: vec![(
									public2.into(),
									Some(AccountId::from(CHARLIE)),
									1_500_000 * MOVR
								)]
							}
						),
						Call::CrowdloanRewards(
							pallet_crowdloan_rewards::Call::<Runtime>::complete_initialization {
								lease_ending_block: end_block
							}
						)
					]
				})
				.dispatch(root_origin())
			);
			// 30 percent initial payout
			assert_eq!(Balances::balance(&AccountId::from(CHARLIE)), 900_000 * MOVR);

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
				(900_000 * MOVR)
			);
		});
}

#[test]
fn claim_via_precompile() {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * MOVR),
			(AccountId::from(BOB), 1_000 * MOVR),
		])
		.with_collators(vec![(AccountId::from(ALICE), 1_000 * MOVR)])
		.with_mappings(vec![(
			NimbusId::from_slice(&ALICE_NIMBUS).unwrap(),
			AccountId::from(ALICE),
		)])
		.with_crowdloan_fund(3_000_000 * MOVR)
		.build()
		.execute_with(|| {
			// set parachain inherent data
			set_parachain_inherent_data();
			let init_block = CrowdloanRewards::init_vesting_block();
			// This matches the previous vesting
			let end_block = init_block + 4 * WEEKS;
			// Batch calls always succeed. We just need to check the inner event
			assert_ok!(Call::Utility(pallet_utility::Call::<Runtime>::batch_all {
				calls: vec![
					Call::CrowdloanRewards(
						pallet_crowdloan_rewards::Call::<Runtime>::initialize_reward_vec {
							rewards: vec![(
								[4u8; 32].into(),
								Some(AccountId::from(CHARLIE)),
								1_500_000 * MOVR
							)]
						}
					),
					Call::CrowdloanRewards(
						pallet_crowdloan_rewards::Call::<Runtime>::initialize_reward_vec {
							rewards: vec![(
								[5u8; 32].into(),
								Some(AccountId::from(DAVE)),
								1_500_000 * MOVR
							)]
						}
					),
					Call::CrowdloanRewards(
						pallet_crowdloan_rewards::Call::<Runtime>::complete_initialization {
							lease_ending_block: end_block
						}
					)
				]
			})
			.dispatch(root_origin()));

			assert!(CrowdloanRewards::initialized());

			// 30 percent initial payout
			assert_eq!(Balances::balance(&AccountId::from(CHARLIE)), 450_000 * MOVR);
			// 30 percent initial payout
			assert_eq!(Balances::balance(&AccountId::from(DAVE)), 450_000 * MOVR);

			let crowdloan_precompile_address = H160::from_low_u64_be(2049);

			// Alice uses the crowdloan precompile to claim through the EVM
			let gas_limit = 100000u64;
			let gas_price: U256 = 1_000_000_000u64.into();

			// Construct the call data (selector, amount)
			let mut call_data = Vec::<u8>::from([0u8; 4]);
			call_data[0..4].copy_from_slice(&Keccak256::digest(b"claim()")[0..4]);

			assert_ok!(Call::EVM(pallet_evm::Call::<Runtime>::call {
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
			.dispatch(<Runtime as frame_system::Config>::Origin::root()));

			let vesting_period = 4 * WEEKS as u128;
			let per_block = (1_050_000 * MOVR) / vesting_period;

			assert_eq!(
				CrowdloanRewards::accounts_payable(&AccountId::from(CHARLIE))
					.unwrap()
					.claimed_reward,
				(450_000 * MOVR) + per_block
			);
		})
}

#[test]
fn is_contributor_via_precompile() {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * MOVR),
			(AccountId::from(BOB), 1_000 * MOVR),
		])
		.with_collators(vec![(AccountId::from(ALICE), 1_000 * MOVR)])
		.with_mappings(vec![(
			NimbusId::from_slice(&ALICE_NIMBUS).unwrap(),
			AccountId::from(ALICE),
		)])
		.with_crowdloan_fund(3_000_000 * MOVR)
		.build()
		.execute_with(|| {
			// set parachain inherent data
			set_parachain_inherent_data();
			let init_block = CrowdloanRewards::init_vesting_block();
			// This matches the previous vesting
			let end_block = init_block + 4 * WEEKS;
			// Batch calls always succeed. We just need to check the inner event
			assert_ok!(Call::Utility(pallet_utility::Call::<Runtime>::batch_all {
				calls: vec![
					Call::CrowdloanRewards(
						pallet_crowdloan_rewards::Call::<Runtime>::initialize_reward_vec {
							rewards: vec![(
								[4u8; 32].into(),
								Some(AccountId::from(CHARLIE)),
								1_500_000 * MOVR
							)]
						}
					),
					Call::CrowdloanRewards(
						pallet_crowdloan_rewards::Call::<Runtime>::initialize_reward_vec {
							rewards: vec![(
								[5u8; 32].into(),
								Some(AccountId::from(DAVE)),
								1_500_000 * MOVR
							)]
						}
					),
					Call::CrowdloanRewards(
						pallet_crowdloan_rewards::Call::<Runtime>::complete_initialization {
							lease_ending_block: end_block
						}
					)
				]
			})
			.dispatch(root_origin()));

			let crowdloan_precompile_address = H160::from_low_u64_be(2049);

			// Assert precompile reports Bob is not a contributor
			Precompiles::new()
				.prepare_test(
					ALICE,
					crowdloan_precompile_address,
					EvmDataWriter::new_with_selector(CrowdloanAction::IsContributor)
						.write(Address(AccountId::from(BOB).into()))
						.build(),
				)
				.expect_cost(1000)
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(false).build());

			// Assert precompile reports Charlie is a nominator
			Precompiles::new()
				.prepare_test(
					ALICE,
					crowdloan_precompile_address,
					EvmDataWriter::new_with_selector(CrowdloanAction::IsContributor)
						.write(Address(AccountId::from(CHARLIE).into()))
						.build(),
				)
				.expect_cost(1000)
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(true).build());
		})
}

#[test]
fn reward_info_via_precompile() {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * MOVR),
			(AccountId::from(BOB), 1_000 * MOVR),
		])
		.with_collators(vec![(AccountId::from(ALICE), 1_000 * MOVR)])
		.with_mappings(vec![(
			NimbusId::from_slice(&ALICE_NIMBUS).unwrap(),
			AccountId::from(ALICE),
		)])
		.with_crowdloan_fund(3_000_000 * MOVR)
		.build()
		.execute_with(|| {
			// set parachain inherent data
			set_parachain_inherent_data();
			let init_block = CrowdloanRewards::init_vesting_block();
			// This matches the previous vesting
			let end_block = init_block + 4 * WEEKS;
			// Batch calls always succeed. We just need to check the inner event
			assert_ok!(Call::Utility(pallet_utility::Call::<Runtime>::batch_all {
				calls: vec![
					Call::CrowdloanRewards(
						pallet_crowdloan_rewards::Call::<Runtime>::initialize_reward_vec {
							rewards: vec![(
								[4u8; 32].into(),
								Some(AccountId::from(CHARLIE)),
								1_500_000 * MOVR
							)]
						}
					),
					Call::CrowdloanRewards(
						pallet_crowdloan_rewards::Call::<Runtime>::initialize_reward_vec {
							rewards: vec![(
								[5u8; 32].into(),
								Some(AccountId::from(DAVE)),
								1_500_000 * MOVR
							)]
						}
					),
					Call::CrowdloanRewards(
						pallet_crowdloan_rewards::Call::<Runtime>::complete_initialization {
							lease_ending_block: end_block
						}
					)
				]
			})
			.dispatch(root_origin()));

			let crowdloan_precompile_address = H160::from_low_u64_be(2049);

			let expected_total: U256 = (1_500_000 * MOVR).into();
			let expected_claimed: U256 = (450_000 * MOVR).into();

			// Assert precompile reports correct Charlie reward info.
			Precompiles::new()
				.prepare_test(
					ALICE,
					crowdloan_precompile_address,
					EvmDataWriter::new_with_selector(CrowdloanAction::RewardInfo)
						.write(Address(AccountId::from(CHARLIE).into()))
						.build(),
				)
				.expect_cost(1000)
				.expect_no_logs()
				.execute_returns(
					EvmDataWriter::new()
						.write(expected_total)
						.write(expected_claimed)
						.build(),
				);
		})
}

#[test]
fn update_reward_address_via_precompile() {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * MOVR),
			(AccountId::from(BOB), 1_000 * MOVR),
		])
		.with_collators(vec![(AccountId::from(ALICE), 1_000 * MOVR)])
		.with_mappings(vec![(
			NimbusId::from_slice(&ALICE_NIMBUS).unwrap(),
			AccountId::from(ALICE),
		)])
		.with_crowdloan_fund(3_000_000 * MOVR)
		.build()
		.execute_with(|| {
			// set parachain inherent data
			set_parachain_inherent_data();
			let init_block = CrowdloanRewards::init_vesting_block();
			// This matches the previous vesting
			let end_block = init_block + 4 * WEEKS;
			// Batch calls always succeed. We just need to check the inner event
			assert_ok!(Call::Utility(pallet_utility::Call::<Runtime>::batch_all {
				calls: vec![
					Call::CrowdloanRewards(
						pallet_crowdloan_rewards::Call::<Runtime>::initialize_reward_vec {
							rewards: vec![(
								[4u8; 32].into(),
								Some(AccountId::from(CHARLIE)),
								1_500_000 * MOVR
							)]
						}
					),
					Call::CrowdloanRewards(
						pallet_crowdloan_rewards::Call::<Runtime>::initialize_reward_vec {
							rewards: vec![(
								[5u8; 32].into(),
								Some(AccountId::from(DAVE)),
								1_500_000 * MOVR
							)]
						}
					),
					Call::CrowdloanRewards(
						pallet_crowdloan_rewards::Call::<Runtime>::complete_initialization {
							lease_ending_block: end_block
						}
					)
				]
			})
			.dispatch(root_origin()));

			let crowdloan_precompile_address = H160::from_low_u64_be(2049);

			// Charlie uses the crowdloan precompile to update address through the EVM
			let gas_limit = 100000u64;
			let gas_price: U256 = 1_000_000_000u64.into();

			// Construct the input data to check if Bob is a contributor
			let mut call_data = Vec::<u8>::from([0u8; 36]);
			call_data[0..4]
				.copy_from_slice(&Keccak256::digest(b"update_reward_address(address)")[0..4]);
			call_data[16..36].copy_from_slice(&ALICE);

			assert_ok!(Call::EVM(pallet_evm::Call::<Runtime>::call {
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
			.dispatch(<Runtime as frame_system::Config>::Origin::root()));

			assert!(CrowdloanRewards::accounts_payable(&AccountId::from(CHARLIE)).is_none());
			assert_eq!(
				CrowdloanRewards::accounts_payable(&AccountId::from(ALICE))
					.unwrap()
					.claimed_reward,
				(450_000 * MOVR)
			);
		})
}

fn run_with_system_weight<F>(w: Weight, mut assertions: F)
where
	F: FnMut() -> (),
{
	let mut t: sp_io::TestExternalities = frame_system::GenesisConfig::default()
		.build_storage::<Runtime>()
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
			moonriver_runtime::TransactionPayment::query_fee_details(uxt.clone(), len)
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
		assert_eq!(        1_001_000_000_000_000_000, calc_fee(1_000_000)); // one MOVR, ~ 1MB
		assert_eq!(    1_000_010_000_000_000_000_000, calc_fee(10_000_000));
		assert_eq!(1_000_000_100_000_000_000_000_000, calc_fee(100_000_000));
		// editorconfig-checker-enable
	});
}

#[test]
fn multiplier_can_grow_from_zero() {
	let minimum_multiplier = moonriver_runtime::MinimumMultiplier::get();
	let target = moonriver_runtime::TargetBlockFullness::get()
		* BlockWeights::get()
			.get(DispatchClass::Normal)
			.max_total
			.unwrap();
	// if the min is too small, then this will not change, and we are doomed forever.
	// the weight is 1/100th bigger than target.
	run_with_system_weight(target * 101 / 100, || {
		let next =
			moonriver_runtime::SlowAdjustingFeeUpdate::<Runtime>::convert(minimum_multiplier);
		assert!(
			next > minimum_multiplier,
			"{:?} !>= {:?}",
			next,
			minimum_multiplier
		);
	})
}

#[test]
#[ignore] // test runs for a very long time
fn multiplier_growth_simulator() {
	// assume the multiplier is initially set to its minimum. We update it with values twice the
	//target (target is 25%, thus 50%) and we see at which point it reaches 1.
	let mut multiplier = moonriver_runtime::MinimumMultiplier::get();
	let block_weight = moonriver_runtime::TargetBlockFullness::get()
		* BlockWeights::get()
			.get(DispatchClass::Normal)
			.max_total
			.unwrap()
		* 2;
	let mut blocks = 0;
	while multiplier <= Multiplier::one() {
		run_with_system_weight(block_weight, || {
			let next = moonriver_runtime::SlowAdjustingFeeUpdate::<Runtime>::convert(multiplier);
			// ensure that it is growing as well.
			assert!(next > multiplier, "{:?} !>= {:?}", next, multiplier);
			multiplier = next;
		});
		blocks += 1;
		println!("block = {} multiplier {:?}", blocks, multiplier);
	}
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
					sp_runtime::transaction_validity::InvalidTransaction::Custom(3u8)
				)
			)
		);
	});
}

#[test]
fn transfer_ed_0_substrate() {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), (1 * MOVR) + (1 * WEI)),
			(AccountId::from(BOB), 0),
		])
		.build()
		.execute_with(|| {
			// Substrate transfer
			assert_ok!(Balances::transfer(
				origin_of(AccountId::from(ALICE)),
				AccountId::from(BOB),
				1 * MOVR,
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
				((1 * MOVR) + (21_000 * 1_000_000_000)) + (1 * WEI),
			),
			(AccountId::from(BOB), 0),
		])
		.build()
		.execute_with(|| {
			// EVM transfer
			assert_ok!(Call::EVM(pallet_evm::Call::<Runtime>::call {
				source: H160::from(ALICE),
				target: H160::from(BOB),
				input: Vec::new(),
				value: (1 * MOVR).into(),
				gas_limit: 21_000u64,
				max_fee_per_gas: U256::from(1_000_000_000),
				max_priority_fee_per_gas: None,
				nonce: Some(U256::from(0)),
				access_list: Vec::new(),
			})
			.dispatch(<Runtime as frame_system::Config>::Origin::root()));
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
				((1 * MOVR) + (21_777 * 1_000_000_000)),
			),
			(AccountId::from(BOB), 0),
		])
		.build()
		.execute_with(|| {
			// EVM transfer that zeroes ALICE
			assert_ok!(Call::EVM(pallet_evm::Call::<Runtime>::call {
				source: H160::from(ALICE),
				target: H160::from(BOB),
				input: Vec::new(),
				value: (1 * MOVR).into(),
				gas_limit: 21_777u64,
				max_fee_per_gas: U256::from(1_000_000_000),
				max_priority_fee_per_gas: None,
				nonce: Some(U256::from(0)),
				access_list: Vec::new(),
			})
			.dispatch(<Runtime as frame_system::Config>::Origin::root()));
			// ALICE is refunded
			assert_eq!(
				Balances::free_balance(AccountId::from(ALICE)),
				777 * 1_000_000_000,
			);
		});
}

#[test]
fn author_does_not_receive_priority_fee() {
	ExtBuilder::default()
		.with_balances(vec![(
			AccountId::from(BOB),
			(1 * MOVR) + (21_000 * (500 * GIGAWEI)),
		)])
		.build()
		.execute_with(|| {
			// Some block author as seen by pallet-evm.
			let author = AccountId::from(<pallet_evm::Pallet<Runtime>>::find_author());
			// Currently the default impl of the evm uses `deposit_into_existing`.
			// If we were to use this implementation, and for an author to receive eventual tips,
			// the account needs to be somehow initialized, otherwise the deposit would fail.
			Balances::make_free_balance_be(&author, 100 * MOVR);

			// EVM transfer.
			assert_ok!(Call::EVM(pallet_evm::Call::<Runtime>::call {
				source: H160::from(BOB),
				target: H160::from(ALICE),
				input: Vec::new(),
				value: (1 * MOVR).into(),
				gas_limit: 21_000u64,
				max_fee_per_gas: U256::from(300 * GIGAWEI),
				max_priority_fee_per_gas: Some(U256::from(200 * GIGAWEI)),
				nonce: Some(U256::from(0)),
				access_list: Vec::new(),
			})
			.dispatch(<Runtime as frame_system::Config>::Origin::root()));
			// Author free balance didn't change.
			assert_eq!(Balances::free_balance(author), 100 * MOVR,);
		});
}

#[test]
fn total_issuance_after_evm_transaction_with_priority_fee() {
	ExtBuilder::default()
		.with_balances(vec![(
			AccountId::from(BOB),
			(1 * MOVR) + (21_000 * (2 * GIGAWEI)),
		)])
		.build()
		.execute_with(|| {
			let issuance_before = <Runtime as pallet_evm::Config>::Currency::total_issuance();
			// EVM transfer.
			assert_ok!(Call::EVM(pallet_evm::Call::<Runtime>::call {
				source: H160::from(BOB),
				target: H160::from(ALICE),
				input: Vec::new(),
				value: (1 * MOVR).into(),
				gas_limit: 21_000u64,
				max_fee_per_gas: U256::from(2 * GIGAWEI),
				max_priority_fee_per_gas: Some(U256::from(1 * GIGAWEI)),
				nonce: Some(U256::from(0)),
				access_list: Vec::new(),
			})
			.dispatch(<Runtime as frame_system::Config>::Origin::root()));

			let issuance_after = <Runtime as pallet_evm::Config>::Currency::total_issuance();
			// Fee is 1 GWEI base fee + 1 GWEI tip.
			let fee = ((2 * GIGAWEI) * 21_000) as f64;
			// 80% was burned.
			let expected_burn = (fee * 0.8) as u128;
			assert_eq!(issuance_after, issuance_before - expected_burn,);
			// 20% was sent to treasury.
			let expected_treasury = (fee * 0.2) as u128;
			assert_eq!(moonriver_runtime::Treasury::pot(), expected_treasury);
		});
}

#[test]
fn total_issuance_after_evm_transaction_without_priority_fee() {
	ExtBuilder::default()
		.with_balances(vec![(
			AccountId::from(BOB),
			(1 * MOVR) + (21_000 * (2 * GIGAWEI)),
		)])
		.build()
		.execute_with(|| {
			let issuance_before = <Runtime as pallet_evm::Config>::Currency::total_issuance();
			// EVM transfer.
			assert_ok!(Call::EVM(pallet_evm::Call::<Runtime>::call {
				source: H160::from(BOB),
				target: H160::from(ALICE),
				input: Vec::new(),
				value: (1 * MOVR).into(),
				gas_limit: 21_000u64,
				max_fee_per_gas: U256::from(1 * GIGAWEI),
				max_priority_fee_per_gas: None,
				nonce: Some(U256::from(0)),
				access_list: Vec::new(),
			})
			.dispatch(<Runtime as frame_system::Config>::Origin::root()));

			let issuance_after = <Runtime as pallet_evm::Config>::Currency::total_issuance();
			// Fee is 1 GWEI base fee.
			let fee = ((1 * GIGAWEI) * 21_000) as f64;
			// 80% was burned.
			let expected_burn = (fee * 0.8) as u128;
			assert_eq!(issuance_after, issuance_before - expected_burn,);
			// 20% was sent to treasury.
			let expected_treasury = (fee * 0.2) as u128;
			assert_eq!(moonriver_runtime::Treasury::pot(), expected_treasury);
		});
}

#[test]
fn root_can_change_default_xcm_vers() {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * MOVR),
			(AccountId::from(BOB), 1_000 * MOVR),
		])
		.with_xcm_assets(vec![XcmAssetInitialization {
			asset_type: AssetType::Xcm(MultiLocation::parent()),
			metadata: AssetRegistrarMetadata {
				name: b"RelayToken".to_vec(),
				symbol: b"Relay".to_vec(),
				decimals: 12,
				is_frozen: false,
			},
			balances: vec![(AccountId::from(ALICE), 1_000_000_000_000_000)],
			is_sufficient: true,
		}])
		.build()
		.execute_with(|| {
			let source_location = AssetType::Xcm(MultiLocation::parent());
			let dest = MultiLocation {
				parents: 1,
				interior: X1(AccountId32 {
					network: NetworkId::Any,
					id: [1u8; 32],
				}),
			};
			let source_id: moonriver_runtime::AssetId = source_location.clone().into();
			// Default XCM version is not set yet, so xtokens should fail because it does not
			// know with which version to send
			assert_noop!(
				XTokens::transfer(
					origin_of(AccountId::from(ALICE)),
					CurrencyId::ForeignAsset(source_id),
					100_000_000_000_000,
					Box::new(xcm::VersionedMultiLocation::V1(dest.clone())),
					4000000000
				),
				orml_xtokens::Error::<Runtime>::XcmExecutionFailed
			);

			// Root sets the defaultXcm
			assert_ok!(PolkadotXcm::force_default_xcm_version(
				root_origin(),
				Some(2)
			));

			// Now transferring does not fail
			assert_ok!(XTokens::transfer(
				origin_of(AccountId::from(ALICE)),
				CurrencyId::ForeignAsset(source_id),
				100_000_000_000_000,
				Box::new(xcm::VersionedMultiLocation::V1(dest)),
				4000000000
			));
		})
}

#[test]
fn asset_can_be_registered() {
	ExtBuilder::default().build().execute_with(|| {
		let source_location = AssetType::Xcm(MultiLocation::parent());
		let source_id: moonriver_runtime::AssetId = source_location.clone().into();
		let asset_metadata = AssetRegistrarMetadata {
			name: b"RelayToken".to_vec(),
			symbol: b"Relay".to_vec(),
			decimals: 12,
			is_frozen: false,
		};
		assert_ok!(AssetManager::register_foreign_asset(
			moonriver_runtime::Origin::root(),
			source_location,
			asset_metadata,
			1u128,
			true
		));
		assert!(AssetManager::asset_id_type(source_id).is_some());
	});
}

#[test]
fn local_assets_cannot_be_create_by_signed_origins() {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * MOVR * SUPPLY_FACTOR),
			(AccountId::from(BOB), 1_000 * MOVR * SUPPLY_FACTOR),
		])
		.build()
		.execute_with(|| {
			assert_noop!(
				Call::LocalAssets(pallet_assets::Call::<Runtime, LocalAssetInstance>::create {
					id: 11u128,
					admin: AccountId::from(ALICE),
					min_balance: 1u128
				})
				.dispatch(<Runtime as frame_system::Config>::Origin::signed(
					AccountId::from(ALICE)
				)),
				frame_system::Error::<Runtime>::CallFiltered
			);
		});
}

#[test]
fn asset_erc20_precompiles_supply_and_balance() {
	ExtBuilder::default()
		.with_local_assets(vec![(
			0u128,
			vec![(AccountId::from(ALICE), 1_000 * MOVR)],
			AccountId::from(ALICE),
		)])
		.build()
		.execute_with(|| {
			// Assert the asset has been created with the correct supply
			assert_eq!(LocalAssets::total_supply(0u128), 1_000 * MOVR);

			// Convert the assetId to its corresponding precompile address
			let asset_precompile_address =
				Runtime::asset_id_to_account(LOCAL_ASSET_PRECOMPILE_ADDRESS_PREFIX, 0u128);

			// The expected result for both total supply and balance of is the same, as only Alice
			// holds balance.

			// Access totalSupply through precompile.
			Precompiles::new()
				.prepare_test(
					ALICE,
					asset_precompile_address,
					EvmDataWriter::new_with_selector(AssetAction::TotalSupply).build(),
				)
				.expect_cost(1000)
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(U256::from(1000 * MOVR)).build());

			// Access balanceOf through precompile
			Precompiles::new()
				.prepare_test(
					ALICE,
					asset_precompile_address,
					EvmDataWriter::new_with_selector(AssetAction::BalanceOf)
						.write(Address(ALICE.into()))
						.build(),
				)
				.expect_cost(1000)
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(U256::from(1000 * MOVR)).build());
		});
}

#[test]
fn asset_erc20_precompiles_transfer() {
	ExtBuilder::default()
		.with_local_assets(vec![(
			0u128,
			vec![(AccountId::from(ALICE), 1_000 * MOVR)],
			AccountId::from(ALICE),
		)])
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * MOVR),
			(AccountId::from(BOB), 1_000 * MOVR),
		])
		.build()
		.execute_with(|| {
			let asset_precompile_address =
				Runtime::asset_id_to_account(LOCAL_ASSET_PRECOMPILE_ADDRESS_PREFIX, 0u128);

			// Transfer tokens from Aice to Bob, 400 MOVR.
			Precompiles::new()
				.prepare_test(
					ALICE,
					asset_precompile_address,
					EvmDataWriter::new_with_selector(AssetAction::Transfer)
						.write(Address(BOB.into()))
						.write(U256::from(400 * MOVR))
						.build(),
				)
				.expect_cost(23652u64)
				.expect_log(log3(
					asset_precompile_address,
					SELECTOR_LOG_TRANSFER,
					H160::from(ALICE),
					H160::from(BOB),
					EvmDataWriter::new().write(U256::from(400 * MOVR)).build(),
				))
				.execute_returns(EvmDataWriter::new().write(true).build());

			// Make sure BOB has 400 MOVR
			Precompiles::new()
				.prepare_test(
					BOB,
					asset_precompile_address,
					EvmDataWriter::new_with_selector(AssetAction::BalanceOf)
						.write(Address(BOB.into()))
						.build(),
				)
				.expect_cost(1000)
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(U256::from(400 * MOVR)).build());
		});
}

#[test]
fn asset_erc20_precompiles_approve() {
	ExtBuilder::default()
		.with_local_assets(vec![(
			0u128,
			vec![(AccountId::from(ALICE), 1_000 * MOVR)],
			AccountId::from(ALICE),
		)])
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * MOVR),
			(AccountId::from(BOB), 1_000 * MOVR),
		])
		.build()
		.execute_with(|| {
			let asset_precompile_address =
				Runtime::asset_id_to_account(LOCAL_ASSET_PRECOMPILE_ADDRESS_PREFIX, 0u128);

			// Aprove Bob for spending 400 MOVR from Alice
			Precompiles::new()
				.prepare_test(
					ALICE,
					asset_precompile_address,
					EvmDataWriter::new_with_selector(AssetAction::Approve)
						.write(Address(BOB.into()))
						.write(U256::from(400 * MOVR))
						.build(),
				)
				.expect_cost(14211)
				.expect_log(log3(
					asset_precompile_address,
					SELECTOR_LOG_APPROVAL,
					H160::from(ALICE),
					H160::from(BOB),
					EvmDataWriter::new().write(U256::from(400 * MOVR)).build(),
				))
				.execute_returns(EvmDataWriter::new().write(true).build());

			// Transfer tokens from Alice to Charlie by using BOB as origin
			Precompiles::new()
				.prepare_test(
					BOB,
					asset_precompile_address,
					EvmDataWriter::new_with_selector(AssetAction::TransferFrom)
						.write(Address(ALICE.into()))
						.write(Address(CHARLIE.into()))
						.write(U256::from(400 * MOVR))
						.build(),
				)
				.expect_cost(29230)
				.expect_log(log3(
					asset_precompile_address,
					SELECTOR_LOG_TRANSFER,
					H160::from(ALICE),
					H160::from(CHARLIE),
					EvmDataWriter::new().write(U256::from(400 * MOVR)).build(),
				))
				.execute_returns(EvmDataWriter::new().write(true).build());

			// Make sure CHARLIE has 400 MOVR
			Precompiles::new()
				.prepare_test(
					CHARLIE,
					asset_precompile_address,
					EvmDataWriter::new_with_selector(AssetAction::BalanceOf)
						.write(Address(CHARLIE.into()))
						.build(),
				)
				.expect_cost(1000)
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(U256::from(400 * MOVR)).build());
		});
}

#[test]
fn asset_erc20_precompiles_mint_burn() {
	ExtBuilder::default()
		.with_local_assets(vec![(
			0u128,
			vec![(AccountId::from(ALICE), 1_000 * MOVR)],
			AccountId::from(ALICE),
		)])
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * MOVR),
			(AccountId::from(BOB), 1_000 * MOVR),
		])
		.build()
		.execute_with(|| {
			let asset_precompile_address =
				Runtime::asset_id_to_account(LOCAL_ASSET_PRECOMPILE_ADDRESS_PREFIX, 0u128);

			// Mint 1000 MOVRS to BOB
			Precompiles::new()
				.prepare_test(
					ALICE,
					asset_precompile_address,
					EvmDataWriter::new_with_selector(AssetAction::Mint)
						.write(Address(BOB.into()))
						.write(U256::from(1000 * MOVR))
						.build(),
				)
				.expect_cost(12988)
				.expect_log(log3(
					asset_precompile_address,
					SELECTOR_LOG_TRANSFER,
					H160::default(),
					H160::from(BOB),
					EvmDataWriter::new().write(U256::from(1000 * MOVR)).build(),
				))
				.execute_returns(EvmDataWriter::new().write(true).build());

			// Assert the asset has been minted
			assert_eq!(LocalAssets::total_supply(0u128), 2_000 * MOVR);
			assert_eq!(
				LocalAssets::balance(0u128, AccountId::from(BOB)),
				1_000 * MOVR
			);

			// Burn tokens
			Precompiles::new()
				.prepare_test(
					ALICE,
					asset_precompile_address,
					EvmDataWriter::new_with_selector(AssetAction::Burn)
						.write(Address(BOB.into()))
						.write(U256::from(500 * MOVR))
						.build(),
				)
				.expect_cost(13164)
				.expect_log(log3(
					asset_precompile_address,
					SELECTOR_LOG_TRANSFER,
					H160::from(BOB),
					H160::default(),
					EvmDataWriter::new().write(U256::from(500 * MOVR)).build(),
				))
				.execute_returns(EvmDataWriter::new().write(true).build());

			// Assert the asset has been burnt
			assert_eq!(LocalAssets::total_supply(0u128), 1_500 * MOVR);
			assert_eq!(
				LocalAssets::balance(0u128, AccountId::from(BOB)),
				500 * MOVR
			);
		});
}

#[test]
fn asset_erc20_precompiles_freeze_thaw_account() {
	ExtBuilder::default()
		.with_local_assets(vec![(
			0u128,
			vec![(AccountId::from(ALICE), 1_000 * MOVR)],
			AccountId::from(ALICE),
		)])
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * MOVR),
			(AccountId::from(BOB), 1_000 * MOVR),
		])
		.build()
		.execute_with(|| {
			let asset_precompile_address =
				Runtime::asset_id_to_account(LOCAL_ASSET_PRECOMPILE_ADDRESS_PREFIX, 0u128);

			// Freeze Account
			Precompiles::new()
				.prepare_test(
					ALICE,
					asset_precompile_address,
					EvmDataWriter::new_with_selector(AssetAction::Freeze)
						.write(Address(ALICE.into()))
						.build(),
				)
				.expect_cost(6866)
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(true).build());

			// Assert account is frozen
			assert_eq!(
				LocalAssets::can_withdraw(0u128, &AccountId::from(ALICE), 1).into_result(),
				Err(TokenError::Frozen.into())
			);

			// Thaw Account
			Precompiles::new()
				.prepare_test(
					ALICE,
					asset_precompile_address,
					EvmDataWriter::new_with_selector(AssetAction::Thaw)
						.write(Address(ALICE.into()))
						.build(),
				)
				.expect_cost(6860)
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(true).build());

			// Assert account is not frozen
			assert!(LocalAssets::can_withdraw(0u128, &AccountId::from(ALICE), 1)
				.into_result()
				.is_ok());
		});
}

#[test]
fn asset_erc20_precompiles_freeze_thaw_asset() {
	ExtBuilder::default()
		.with_local_assets(vec![(
			0u128,
			vec![(AccountId::from(ALICE), 1_000 * MOVR)],
			AccountId::from(ALICE),
		)])
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * MOVR),
			(AccountId::from(BOB), 1_000 * MOVR),
		])
		.build()
		.execute_with(|| {
			let asset_precompile_address =
				Runtime::asset_id_to_account(LOCAL_ASSET_PRECOMPILE_ADDRESS_PREFIX, 0u128);

			// Freeze Asset
			Precompiles::new()
				.prepare_test(
					ALICE,
					asset_precompile_address,
					EvmDataWriter::new_with_selector(AssetAction::FreezeAsset).build(),
				)
				.expect_cost(5726)
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(true).build());

			// Assert account is frozen
			assert_eq!(
				LocalAssets::can_withdraw(0u128, &AccountId::from(ALICE), 1).into_result(),
				Err(TokenError::Frozen.into())
			);

			// Thaw Asset
			Precompiles::new()
				.prepare_test(
					ALICE,
					asset_precompile_address,
					EvmDataWriter::new_with_selector(AssetAction::ThawAsset).build(),
				)
				.expect_cost(5741)
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(true).build());

			// Assert account is not frozen
			assert!(LocalAssets::can_withdraw(0u128, &AccountId::from(ALICE), 1)
				.into_result()
				.is_ok());
		});
}

#[test]
fn asset_erc20_precompiles_freeze_transfer_ownership() {
	ExtBuilder::default()
		.with_local_assets(vec![(
			0u128,
			vec![(AccountId::from(ALICE), 1_000 * MOVR)],
			AccountId::from(ALICE),
		)])
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * MOVR),
			(AccountId::from(BOB), 1_000 * MOVR),
		])
		.build()
		.execute_with(|| {
			let asset_precompile_address =
				Runtime::asset_id_to_account(LOCAL_ASSET_PRECOMPILE_ADDRESS_PREFIX, 0u128);

			// Transfer ownerhsip of an asset
			Precompiles::new()
				.prepare_test(
					ALICE,
					asset_precompile_address,
					EvmDataWriter::new_with_selector(AssetAction::TransferOwnership)
						.write(Address(BOB.into()))
						.build(),
				)
				.expect_cost(6794)
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(true).build());

			// No clear way of testing BOB is new owner, other than doing a priviledged function
			// e.g., transfer_ownership again
			assert_ok!(LocalAssets::transfer_ownership(
				origin_of(AccountId::from(BOB)),
				0u128,
				AccountId::from(ALICE)
			));
		});
}

#[test]
fn asset_erc20_precompiles_freeze_set_team() {
	ExtBuilder::default()
		.with_local_assets(vec![(
			0u128,
			vec![(AccountId::from(ALICE), 1_000 * MOVR)],
			AccountId::from(ALICE),
		)])
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * MOVR),
			(AccountId::from(BOB), 1_000 * MOVR),
		])
		.build()
		.execute_with(|| {
			let asset_precompile_address =
				Runtime::asset_id_to_account(LOCAL_ASSET_PRECOMPILE_ADDRESS_PREFIX, 0u128);

			// Set Bob as issuer, admin and freezer
			Precompiles::new()
				.prepare_test(
					ALICE,
					asset_precompile_address,
					EvmDataWriter::new_with_selector(AssetAction::SetTeam)
						.write(Address(BOB.into()))
						.write(Address(BOB.into()))
						.write(Address(BOB.into()))
						.build(),
				)
				.expect_cost(5721)
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(true).build());

			// Bob should be able to mint, freeze, and thaw
			assert_ok!(LocalAssets::mint(
				origin_of(AccountId::from(BOB)),
				0u128,
				AccountId::from(BOB),
				1_000 * MOVR
			));
			assert_ok!(LocalAssets::freeze(
				origin_of(AccountId::from(BOB)),
				0u128,
				AccountId::from(ALICE)
			));
			assert_ok!(LocalAssets::thaw(
				origin_of(AccountId::from(BOB)),
				0u128,
				AccountId::from(ALICE)
			));
		});
}

#[test]
fn xcm_asset_erc20_precompiles_supply_and_balance() {
	ExtBuilder::default()
		.with_xcm_assets(vec![XcmAssetInitialization {
			asset_type: AssetType::Xcm(MultiLocation::parent()),
			metadata: AssetRegistrarMetadata {
				name: b"RelayToken".to_vec(),
				symbol: b"Relay".to_vec(),
				decimals: 12,
				is_frozen: false,
			},
			balances: vec![(AccountId::from(ALICE), 1_000 * MOVR)],
			is_sufficient: true,
		}])
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * MOVR),
			(AccountId::from(BOB), 1_000 * MOVR),
		])
		.build()
		.execute_with(|| {
			// We have the assetId that corresponds to the relay chain registered
			let relay_asset_id: AssetId = AssetType::Xcm(MultiLocation::parent()).into();

			// Its address is
			let asset_precompile_address = Runtime::asset_id_to_account(
				FOREIGN_ASSET_PRECOMPILE_ADDRESS_PREFIX,
				relay_asset_id,
			);

			// Assert the asset has been created with the correct supply
			assert_eq!(Assets::total_supply(relay_asset_id), 1_000 * MOVR);

			// Access totalSupply through precompile. Important that the context is correct
			Precompiles::new()
				.prepare_test(
					ALICE,
					asset_precompile_address,
					EvmDataWriter::new_with_selector(AssetAction::TotalSupply).build(),
				)
				.expect_cost(1000)
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(U256::from(1000 * MOVR)).build());

			// Access balanceOf through precompile
			Precompiles::new()
				.prepare_test(
					ALICE,
					asset_precompile_address,
					EvmDataWriter::new_with_selector(AssetAction::BalanceOf)
						.write(Address(ALICE.into()))
						.build(),
				)
				.expect_cost(1000)
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(U256::from(1000 * MOVR)).build());
		});
}

#[test]
fn xcm_asset_erc20_precompiles_transfer() {
	ExtBuilder::default()
		.with_xcm_assets(vec![XcmAssetInitialization {
			asset_type: AssetType::Xcm(MultiLocation::parent()),
			metadata: AssetRegistrarMetadata {
				name: b"RelayToken".to_vec(),
				symbol: b"Relay".to_vec(),
				decimals: 12,
				is_frozen: false,
			},
			balances: vec![(AccountId::from(ALICE), 1_000 * MOVR)],
			is_sufficient: true,
		}])
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * MOVR),
			(AccountId::from(BOB), 1_000 * MOVR),
		])
		.build()
		.execute_with(|| {
			// We have the assetId that corresponds to the relay chain registered
			let relay_asset_id: AssetId = AssetType::Xcm(MultiLocation::parent()).into();

			// Its address is
			let asset_precompile_address = Runtime::asset_id_to_account(
				FOREIGN_ASSET_PRECOMPILE_ADDRESS_PREFIX,
				relay_asset_id,
			);

			// Transfer tokens from Aice to Bob, 400 MOVR.
			Precompiles::new()
				.prepare_test(
					ALICE,
					asset_precompile_address,
					EvmDataWriter::new_with_selector(AssetAction::Transfer)
						.write(Address(BOB.into()))
						.write(U256::from(400 * MOVR))
						.build(),
				)
				.expect_cost(23652)
				.expect_log(log3(
					asset_precompile_address,
					SELECTOR_LOG_TRANSFER,
					H160::from(ALICE),
					H160::from(BOB),
					EvmDataWriter::new().write(U256::from(400 * MOVR)).build(),
				))
				.execute_returns(EvmDataWriter::new().write(true).build());

			// Make sure BOB has 400 MOVR
			Precompiles::new()
				.prepare_test(
					BOB,
					asset_precompile_address,
					EvmDataWriter::new_with_selector(AssetAction::BalanceOf)
						.write(Address(BOB.into()))
						.build(),
				)
				.expect_cost(1000)
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(U256::from(400 * MOVR)).build());
		});
}

#[test]
fn xcm_asset_erc20_precompiles_approve() {
	ExtBuilder::default()
		.with_xcm_assets(vec![XcmAssetInitialization {
			asset_type: AssetType::Xcm(MultiLocation::parent()),
			metadata: AssetRegistrarMetadata {
				name: b"RelayToken".to_vec(),
				symbol: b"Relay".to_vec(),
				decimals: 12,
				is_frozen: false,
			},
			balances: vec![(AccountId::from(ALICE), 1_000 * MOVR)],
			is_sufficient: true,
		}])
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * MOVR),
			(AccountId::from(BOB), 1_000 * MOVR),
		])
		.build()
		.execute_with(|| {
			// We have the assetId that corresponds to the relay chain registered
			let relay_asset_id: AssetId = AssetType::Xcm(MultiLocation::parent()).into();

			// Its address is
			let asset_precompile_address = Runtime::asset_id_to_account(
				FOREIGN_ASSET_PRECOMPILE_ADDRESS_PREFIX,
				relay_asset_id,
			);

			// Aprove Bob for spending 400 MOVR from Alice
			Precompiles::new()
				.prepare_test(
					ALICE,
					asset_precompile_address,
					EvmDataWriter::new_with_selector(AssetAction::Approve)
						.write(Address(BOB.into()))
						.write(U256::from(400 * MOVR))
						.build(),
				)
				.expect_cost(14211)
				.expect_log(log3(
					asset_precompile_address,
					SELECTOR_LOG_APPROVAL,
					H160::from(ALICE),
					H160::from(BOB),
					EvmDataWriter::new().write(U256::from(400 * MOVR)).build(),
				))
				.execute_returns(EvmDataWriter::new().write(true).build());

			// Transfer tokens from Alice to Charlie by using BOB as origin
			Precompiles::new()
				.prepare_test(
					BOB,
					asset_precompile_address,
					EvmDataWriter::new_with_selector(AssetAction::TransferFrom)
						.write(Address(ALICE.into()))
						.write(Address(CHARLIE.into()))
						.write(U256::from(400 * MOVR))
						.build(),
				)
				.expect_cost(29230)
				.expect_log(log3(
					asset_precompile_address,
					SELECTOR_LOG_TRANSFER,
					H160::from(ALICE),
					H160::from(CHARLIE),
					EvmDataWriter::new().write(U256::from(400 * MOVR)).build(),
				))
				.execute_returns(EvmDataWriter::new().write(true).build());

			// Make sure CHARLIE has 400 MOVR
			Precompiles::new()
				.prepare_test(
					CHARLIE,
					asset_precompile_address,
					EvmDataWriter::new_with_selector(AssetAction::BalanceOf)
						.write(Address(CHARLIE.into()))
						.build(),
				)
				.expect_cost(1000)
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(U256::from(400 * MOVR)).build());
		});
}

#[test]
fn xtokens_precompiles_transfer() {
	ExtBuilder::default()
		.with_xcm_assets(vec![XcmAssetInitialization {
			asset_type: AssetType::Xcm(MultiLocation::parent()),
			metadata: AssetRegistrarMetadata {
				name: b"RelayToken".to_vec(),
				symbol: b"Relay".to_vec(),
				decimals: 12,
				is_frozen: false,
			},
			balances: vec![(AccountId::from(ALICE), 1_000_000_000_000_000)],
			is_sufficient: true,
		}])
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * MOVR),
			(AccountId::from(BOB), 1_000 * MOVR),
		])
		.with_safe_xcm_version(2)
		.build()
		.execute_with(|| {
			let xtokens_precompile_address = H160::from_low_u64_be(2052);

			// We have the assetId that corresponds to the relay chain registered
			let relay_asset_id: moonriver_runtime::AssetId =
				AssetType::Xcm(MultiLocation::parent()).into();

			// Its address is
			let asset_precompile_address = Runtime::asset_id_to_account(
				FOREIGN_ASSET_PRECOMPILE_ADDRESS_PREFIX,
				relay_asset_id,
			);

			// Alice has 1000 tokens. She should be able to send through precompile
			let destination = MultiLocation::new(
				1,
				Junctions::X1(Junction::AccountId32 {
					network: NetworkId::Any,
					id: [1u8; 32],
				}),
			);

			// We use the address of the asset as an identifier of the asset we want to transferS
			Precompiles::new()
				.prepare_test(
					ALICE,
					xtokens_precompile_address,
					EvmDataWriter::new_with_selector(XtokensAction::Transfer)
						.write(Address(asset_precompile_address.into()))
						.write(U256::from(500_000_000_000_000u128))
						.write(destination.clone())
						.write(U256::from(4000000))
						.build(),
				)
				.expect_cost(20000)
				.expect_no_logs()
				.execute_returns(vec![])
		})
}

#[test]
fn xtokens_precompiles_transfer_multiasset() {
	ExtBuilder::default()
		.with_xcm_assets(vec![XcmAssetInitialization {
			asset_type: AssetType::Xcm(MultiLocation::parent()),
			metadata: AssetRegistrarMetadata {
				name: b"RelayToken".to_vec(),
				symbol: b"Relay".to_vec(),
				decimals: 12,
				is_frozen: false,
			},
			balances: vec![(AccountId::from(ALICE), 1_000_000_000_000_000)],
			is_sufficient: true,
		}])
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * MOVR),
			(AccountId::from(BOB), 1_000 * MOVR),
		])
		.with_safe_xcm_version(2)
		.build()
		.execute_with(|| {
			let xtokens_precompile_address = H160::from_low_u64_be(2052);

			// Alice has 1000 tokens. She should be able to send through precompile
			let destination = MultiLocation::new(
				1,
				Junctions::X1(Junction::AccountId32 {
					network: NetworkId::Any,
					id: [1u8; 32],
				}),
			);

			// This time we transfer it through TransferMultiAsset
			// Instead of the address, we encode directly the multilocation referencing the asset
			Precompiles::new()
				.prepare_test(
					ALICE,
					xtokens_precompile_address,
					EvmDataWriter::new_with_selector(XtokensAction::TransferMultiAsset)
						// We want to transfer the relay token
						.write(MultiLocation::parent())
						.write(U256::from(500_000_000_000_000u128))
						.write(destination)
						.write(U256::from(4000000))
						.build(),
				)
				.expect_cost(20000)
				.expect_no_logs()
				.execute_returns(vec![]);
		})
}

#[test]
fn make_sure_polkadot_xcm_cannot_be_called() {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * MOVR),
			(AccountId::from(BOB), 1_000 * MOVR),
		])
		.with_collators(vec![(AccountId::from(ALICE), 1_000 * MOVR)])
		.with_mappings(vec![(
			NimbusId::from_slice(&ALICE_NIMBUS).unwrap(),
			AccountId::from(ALICE),
		)])
		.build()
		.execute_with(|| {
			let dest = MultiLocation {
				parents: 1,
				interior: X1(AccountId32 {
					network: NetworkId::Any,
					id: [1u8; 32],
				}),
			};
			let multiassets: MultiAssets = [MultiAsset {
				id: Concrete(moonriver_runtime::xcm_config::SelfLocation::get()),
				fun: Fungible(1000),
			}]
			.to_vec()
			.into();
			assert_noop!(
				Call::PolkadotXcm(pallet_xcm::Call::<Runtime>::reserve_transfer_assets {
					dest: Box::new(VersionedMultiLocation::V1(dest.clone())),
					beneficiary: Box::new(VersionedMultiLocation::V1(dest)),
					assets: Box::new(VersionedMultiAssets::V1(multiassets)),
					fee_asset_item: 0,
				})
				.dispatch(<Runtime as frame_system::Config>::Origin::signed(
					AccountId::from(ALICE)
				)),
				frame_system::Error::<Runtime>::CallFiltered
			);
		});
}

#[test]
fn transactor_cannot_use_more_than_max_weight() {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * MOVR),
			(AccountId::from(BOB), 1_000 * MOVR),
		])
		.with_xcm_assets(vec![XcmAssetInitialization {
			asset_type: AssetType::Xcm(MultiLocation::parent()),
			metadata: AssetRegistrarMetadata {
				name: b"RelayToken".to_vec(),
				symbol: b"Relay".to_vec(),
				decimals: 12,
				is_frozen: false,
			},
			balances: vec![(AccountId::from(ALICE), 1_000_000_000_000_000)],
			is_sufficient: true,
		}])
		.build()
		.execute_with(|| {
			let source_location = AssetType::Xcm(MultiLocation::parent());
			let source_id: moonriver_runtime::AssetId = source_location.clone().into();
			assert_ok!(XcmTransactor::register(
				root_origin(),
				AccountId::from(ALICE),
				0,
			));

			// Root can set transact info
			assert_ok!(XcmTransactor::set_transact_info(
				root_origin(),
				Box::new(xcm::VersionedMultiLocation::V1(MultiLocation::parent())),
				// Relay charges 1000 for every instruction, and we have 3, so 3000
				3000,
				20000,
				None
			));

			// Root can set transact info
			assert_ok!(XcmTransactor::set_fee_per_second(
				root_origin(),
				Box::new(xcm::VersionedMultiLocation::V1(MultiLocation::parent())),
				1
			));

			assert_noop!(
				XcmTransactor::transact_through_derivative(
					origin_of(AccountId::from(ALICE)),
					moonriver_runtime::xcm_config::Transactors::Relay,
					0,
					CurrencyPayment {
						currency: Currency::AsMultiLocation(Box::new(
							xcm::VersionedMultiLocation::V1(MultiLocation::parent())
						)),
						fee_amount: None
					},
					vec![],
					// 2000 is the max
					TransactWeights {
						transact_required_weight_at_most: 17001,
						overall_weight: None
					}
				),
				pallet_xcm_transactor::Error::<Runtime>::MaxWeightTransactReached
			);
			assert_noop!(
				XcmTransactor::transact_through_derivative(
					origin_of(AccountId::from(ALICE)),
					moonriver_runtime::xcm_config::Transactors::Relay,
					0,
					CurrencyPayment {
						currency: Currency::AsCurrencyId(CurrencyId::ForeignAsset(source_id)),
						fee_amount: None
					},
					vec![],
					// 20000 is the max
					TransactWeights {
						transact_required_weight_at_most: 17001,
						overall_weight: None
					}
				),
				pallet_xcm_transactor::Error::<Runtime>::MaxWeightTransactReached
			);
		})
}

#[test]
fn transact_through_signed_precompile_not_enabled() {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * MOVR),
			(AccountId::from(BOB), 1_000 * MOVR),
		])
		.with_safe_xcm_version(2)
		.build()
		.execute_with(|| {
			// Destination
			let dest = MultiLocation::parent();

			let fee_payer_asset = MultiLocation::parent();

			let bytes: Bytes = vec![1u8, 2u8, 3u8].as_slice().into();

			let xcm_transactor_v1_precompile_address = H160::from_low_u64_be(2054);

			// Root can set transact info
			assert_ok!(XcmTransactor::set_transact_info(
				root_origin(),
				Box::new(xcm::VersionedMultiLocation::V1(MultiLocation::parent())),
				// Relay charges 1000 for every instruction, and we have 3, so 3000
				3000,
				20000,
				Some(4000)
			));
			// Root can set transact info
			assert_ok!(XcmTransactor::set_fee_per_second(
				root_origin(),
				Box::new(xcm::VersionedMultiLocation::V1(MultiLocation::parent())),
				1,
			));

			Precompiles::new()
				.prepare_test(
					ALICE,
					xcm_transactor_v1_precompile_address,
					EvmDataWriter::new_with_selector(
						XcmTransactorActionV1::TransactThroughSignedMultiLocation,
					)
					.write(dest)
					.write(fee_payer_asset)
					.write(U256::from(15000))
					.write(bytes)
					.build(),
				)
				.execute_reverts(|output| {
					from_utf8(&output)
						.unwrap()
						.contains("Dispatched call failed with error:")
						&& from_utf8(&output).unwrap().contains("CallFiltered")
				});
		});
}

#[test]
fn transact_through_signed_mult_not_enabled() {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * MOVR),
			(AccountId::from(BOB), 1_000 * MOVR),
		])
		.with_xcm_assets(vec![XcmAssetInitialization {
			asset_type: AssetType::Xcm(MultiLocation::parent()),
			metadata: AssetRegistrarMetadata {
				name: b"RelayToken".to_vec(),
				symbol: b"Relay".to_vec(),
				decimals: 12,
				is_frozen: false,
			},
			balances: vec![(AccountId::from(ALICE), 1_000_000_000_000_000)],
			is_sufficient: true,
		}])
		.build()
		.execute_with(|| {
			// Root can set transact info
			assert_ok!(XcmTransactor::set_transact_info(
				root_origin(),
				Box::new(xcm::VersionedMultiLocation::V1(MultiLocation::parent())),
				// Relay charges 1000 for every instruction, and we have 3, so 3000
				3000,
				20000,
				// lets say 1000 per instruction
				Some(4000)
			));

			// Root can set transact info
			assert_ok!(XcmTransactor::set_fee_per_second(
				root_origin(),
				Box::new(xcm::VersionedMultiLocation::V1(MultiLocation::parent())),
				1
			));

			assert_noop!(
				Call::XcmTransactor(
					pallet_xcm_transactor::Call::<Runtime>::transact_through_signed {
						dest: Box::new(xcm::VersionedMultiLocation::V1(MultiLocation::parent())),
						fee: CurrencyPayment {
							currency: Currency::AsMultiLocation(Box::new(
								xcm::VersionedMultiLocation::V1(MultiLocation::parent())
							)),
							fee_amount: None
						},
						call: vec![],
						weight_info: TransactWeights {
							transact_required_weight_at_most: 11000,
							overall_weight: None
						}
					}
				)
				.dispatch(<Runtime as frame_system::Config>::Origin::signed(
					AccountId::from(ALICE)
				)),
				frame_system::Error::<Runtime>::CallFiltered
			);
		})
}

#[test]
fn transact_through_signed_not_enabled() {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * MOVR),
			(AccountId::from(BOB), 1_000 * MOVR),
		])
		.with_xcm_assets(vec![XcmAssetInitialization {
			asset_type: AssetType::Xcm(MultiLocation::parent()),
			metadata: AssetRegistrarMetadata {
				name: b"RelayToken".to_vec(),
				symbol: b"Relay".to_vec(),
				decimals: 12,
				is_frozen: false,
			},
			balances: vec![(AccountId::from(ALICE), 1_000_000_000_000_000)],
			is_sufficient: true,
		}])
		.build()
		.execute_with(|| {
			let source_location = AssetType::Xcm(MultiLocation::parent());
			let source_id: moonriver_runtime::AssetId = source_location.clone().into();

			// Root can set transact info
			assert_ok!(XcmTransactor::set_transact_info(
				root_origin(),
				Box::new(xcm::VersionedMultiLocation::V1(MultiLocation::parent())),
				// Relay charges 1000 for every instruction, and we have 3, so 3000
				3000,
				20000,
				// lets say 1000 per instruction
				Some(4000)
			));

			// Root can set transact info
			assert_ok!(XcmTransactor::set_fee_per_second(
				root_origin(),
				Box::new(xcm::VersionedMultiLocation::V1(MultiLocation::parent())),
				1
			));

			assert_noop!(
				Call::XcmTransactor(
					pallet_xcm_transactor::Call::<Runtime>::transact_through_signed {
						dest: Box::new(xcm::VersionedMultiLocation::V1(MultiLocation::parent())),
						fee: CurrencyPayment {
							currency: Currency::AsCurrencyId(CurrencyId::ForeignAsset(source_id)),
							fee_amount: None
						},
						call: vec![],
						weight_info: TransactWeights {
							transact_required_weight_at_most: 11000,
							overall_weight: None
						}
					}
				)
				.dispatch(<Runtime as frame_system::Config>::Origin::signed(
					AccountId::from(ALICE)
				)),
				frame_system::Error::<Runtime>::CallFiltered
			);
		})
}

#[test]
fn call_xtokens_with_fee() {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * MOVR),
			(AccountId::from(BOB), 1_000 * MOVR),
		])
		.with_safe_xcm_version(2)
		.with_xcm_assets(vec![XcmAssetInitialization {
			asset_type: AssetType::Xcm(MultiLocation::parent()),
			metadata: AssetRegistrarMetadata {
				name: b"RelayToken".to_vec(),
				symbol: b"Relay".to_vec(),
				decimals: 12,
				is_frozen: false,
			},
			balances: vec![(AccountId::from(ALICE), 1_000_000_000_000_000)],
			is_sufficient: true,
		}])
		.build()
		.execute_with(|| {
			let source_location = AssetType::Xcm(MultiLocation::parent());
			let dest = MultiLocation {
				parents: 1,
				interior: X1(AccountId32 {
					network: NetworkId::Any,
					id: [1u8; 32],
				}),
			};
			let source_id: moonriver_runtime::AssetId = source_location.clone().into();

			let before_balance = Assets::balance(source_id, &AccountId::from(ALICE));

			// We are able to transfer with fee
			assert_ok!(XTokens::transfer_with_fee(
				origin_of(AccountId::from(ALICE)),
				CurrencyId::ForeignAsset(source_id),
				100_000_000_000_000,
				100,
				Box::new(xcm::VersionedMultiLocation::V1(dest.clone())),
				4000000000
			),);

			let after_balance = Assets::balance(source_id, &AccountId::from(ALICE));
			// At least these much (plus fees) should have been charged
			assert_eq!(before_balance - 100_000_000_000_000 - 100, after_balance);
		});
}

#[test]
fn test_xcm_utils_ml_to_account() {
	ExtBuilder::default().build().execute_with(|| {
		let xcm_utils_precompile_address = H160::from_low_u64_be(2060);
		let expected_address_parent: H160 =
			ParentIsPreset::<AccountId>::convert_ref(MultiLocation::parent())
				.unwrap()
				.into();

		Precompiles::new()
			.prepare_test(
				ALICE,
				xcm_utils_precompile_address,
				EvmDataWriter::new_with_selector(XcmUtilsAction::MultiLocationToAddress)
					.write(MultiLocation::parent())
					.build(),
			)
			.expect_cost(1000)
			.expect_no_logs()
			.execute_returns(
				EvmDataWriter::new()
					.write(Address(expected_address_parent))
					.build(),
			);

		let parachain_2000_multilocation = MultiLocation::new(1, X1(Parachain(2000)));
		let expected_address_parachain: H160 =
			SiblingParachainConvertsVia::<Sibling, AccountId>::convert_ref(
				parachain_2000_multilocation.clone(),
			)
			.unwrap()
			.into();

		Precompiles::new()
			.prepare_test(
				ALICE,
				xcm_utils_precompile_address,
				EvmDataWriter::new_with_selector(XcmUtilsAction::MultiLocationToAddress)
					.write(parachain_2000_multilocation)
					.build(),
			)
			.expect_cost(1000)
			.expect_no_logs()
			.execute_returns(
				EvmDataWriter::new()
					.write(Address(expected_address_parachain))
					.build(),
			);

		let alice_in_parachain_2000_multilocation = MultiLocation::new(
			1,
			X2(
				Parachain(2000),
				AccountKey20 {
					network: Any,
					key: ALICE,
				},
			),
		);

		// this should fail, this convertor is not allowed in moonriver
		Precompiles::new()
			.prepare_test(
				ALICE,
				xcm_utils_precompile_address,
				EvmDataWriter::new_with_selector(XcmUtilsAction::MultiLocationToAddress)
					.write(alice_in_parachain_2000_multilocation)
					.build(),
			)
			.expect_cost(1000)
			.expect_no_logs()
			.execute_reverts(|output| output == b"multilocation: Failed multilocation conversion");
	});
}

#[test]
fn precompile_existence() {
	ExtBuilder::default().build().execute_with(|| {
		let precompiles = Precompiles::new();
		let precompile_addresses: std::collections::BTreeSet<_> = vec![
			1, 2, 3, 4, 5, 6, 7, 8, 9, 1024, 1025, 1026, 2048, 2049, 2050, 2051, 2052, 2053, 2054,
			2055, 2056, 2058, 2060, 2062, 2063, 2064,
		]
		.into_iter()
		.map(H160::from_low_u64_be)
		.collect();

		for i in 0..3000 {
			let address = H160::from_low_u64_be(i);

			if precompile_addresses.contains(&address) {
				assert!(
					precompiles.is_precompile(address),
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
					!precompiles.is_precompile(address),
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
fn base_fee_should_default_to_associate_type_value() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(
			BaseFee::base_fee_per_gas(),
			(1 * GIGAWEI * SUPPLY_FACTOR).into()
		);
	});
}

#[test]
fn evm_revert_substrate_events() {
	ExtBuilder::default()
		.with_balances(vec![(AccountId::from(ALICE), 1_000 * MOVR)])
		.build()
		.execute_with(|| {
			let batch_precompile_address = H160::from_low_u64_be(2056);

			// Batch a transfer followed by an invalid call to batch.
			// Thus BatchAll will revert the transfer.
			assert_ok!(Call::EVM(pallet_evm::Call::call {
				source: ALICE.into(),
				target: batch_precompile_address,
				input: EvmDataWriter::new_with_selector(BatchAction::BatchAll)
					.write(vec![Address(BOB.into()), Address(batch_precompile_address)])
					.write(vec![U256::from(1 * MOVR), U256::zero()])
					.write::<Vec<Bytes>>(vec![])
					.write::<Vec<U256>>(vec![])
					.build(),
				value: U256::zero(), // No value sent in EVM
				gas_limit: 500_000,
				max_fee_per_gas: U256::from(1 * GIGAWEI),
				max_priority_fee_per_gas: None,
				nonce: Some(U256::from(0)),
				access_list: Vec::new(),
			})
			.dispatch(<Runtime as frame_system::Config>::Origin::root()));

			let transfer_count = System::events()
				.iter()
				.filter(|r| match r.event {
					Event::Balances(pallet_balances::Event::Transfer { .. }) => true,
					_ => false,
				})
				.count();

			assert_eq!(transfer_count, 0, "there should be no transfer event");
		});
}

#[test]
fn evm_success_keeps_substrate_events() {
	ExtBuilder::default()
		.with_balances(vec![(AccountId::from(ALICE), 1_000 * MOVR)])
		.build()
		.execute_with(|| {
			let batch_precompile_address = H160::from_low_u64_be(2056);

			assert_ok!(Call::EVM(pallet_evm::Call::call {
				source: ALICE.into(),
				target: batch_precompile_address,
				input: EvmDataWriter::new_with_selector(BatchAction::BatchAll)
					.write(vec![Address(BOB.into())])
					.write(vec![U256::from(1 * MOVR)])
					.write::<Vec<Bytes>>(vec![])
					.write::<Vec<U256>>(vec![])
					.build(),
				value: U256::zero(), // No value sent in EVM
				gas_limit: 500_000,
				max_fee_per_gas: U256::from(1 * GIGAWEI),
				max_priority_fee_per_gas: None,
				nonce: Some(U256::from(0)),
				access_list: Vec::new(),
			})
			.dispatch(<Runtime as frame_system::Config>::Origin::root()));

			let transfer_count = System::events()
				.iter()
				.filter(|r| match r.event {
					Event::Balances(pallet_balances::Event::Transfer { .. }) => true,
					_ => false,
				})
				.count();

			assert_eq!(transfer_count, 1, "there should be 1 transfer event");
		});
}
