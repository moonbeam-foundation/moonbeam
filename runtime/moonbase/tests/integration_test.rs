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

//! Moonbase Runtime Integration Tests

mod common;
use common::*;

use precompile_utils::{Address as EvmAddress, EvmDataWriter, LogsBuilder};

use evm::{executor::PrecompileOutput, ExitSucceed};
use frame_support::{
	assert_noop, assert_ok,
	dispatch::Dispatchable,
	traits::{fungible::Inspect, PalletInfo, StorageInfo, StorageInfoTrait},
	weights::{DispatchClass, Weight},
	StorageHasher, Twox128,
};
use moonbase_runtime::{
	currency::UNIT, AccountId, AssetId, AssetManager, AssetRegistrarMetadata, AssetType, Assets,
	Balances, BlockWeights, Call, CrowdloanRewards, Event, ParachainStaking, Precompiles, Runtime,
	System,
};
use nimbus_primitives::NimbusId;
use pallet_evm::PrecompileSet;
use pallet_evm_precompile_assets_erc20::{
	AccountIdAssetIdConversion, Action as AssetAction, SELECTOR_LOG_APPROVAL, SELECTOR_LOG_TRANSFER,
};
use xtokens_precompiles::Action as XtokensAction;

use pallet_transaction_payment::Multiplier;
use parachain_staking::Bond;
use parity_scale_codec::Encode;
use sha3::{Digest, Keccak256};
use sp_core::Pair;
use sp_core::{Public, H160, U256};
use sp_runtime::{
	traits::{Convert, One},
	DispatchError,
};
use xcm::latest::prelude::*;
use xcm::v1::{Junctions, MultiLocation};

#[test]
fn fast_track_available() {
	assert!(<moonbase_runtime::Runtime as pallet_democracy::Config>::InstantAllowed::get());
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
	is_pallet_prefix::<moonbase_runtime::RandomnessCollectiveFlip>("RandomnessCollectiveFlip");
	is_pallet_prefix::<moonbase_runtime::ParachainSystem>("ParachainSystem");
	is_pallet_prefix::<moonbase_runtime::TransactionPayment>("TransactionPayment");
	is_pallet_prefix::<moonbase_runtime::ParachainInfo>("ParachainInfo");
	is_pallet_prefix::<moonbase_runtime::EthereumChainId>("EthereumChainId");
	is_pallet_prefix::<moonbase_runtime::EVM>("EVM");
	is_pallet_prefix::<moonbase_runtime::Ethereum>("Ethereum");
	is_pallet_prefix::<moonbase_runtime::ParachainStaking>("ParachainStaking");
	is_pallet_prefix::<moonbase_runtime::Scheduler>("Scheduler");
	is_pallet_prefix::<moonbase_runtime::Democracy>("Democracy");
	is_pallet_prefix::<moonbase_runtime::CouncilCollective>("CouncilCollective");
	is_pallet_prefix::<moonbase_runtime::TechCommitteeCollective>("TechCommitteeCollective");
	is_pallet_prefix::<moonbase_runtime::Treasury>("Treasury");
	is_pallet_prefix::<moonbase_runtime::AuthorInherent>("AuthorInherent");
	is_pallet_prefix::<moonbase_runtime::AuthorFilter>("AuthorFilter");
	is_pallet_prefix::<moonbase_runtime::CrowdloanRewards>("CrowdloanRewards");
	is_pallet_prefix::<moonbase_runtime::AuthorMapping>("AuthorMapping");
	is_pallet_prefix::<moonbase_runtime::MaintenanceMode>("MaintenanceMode");
	let prefix = |pallet_name, storage_name| {
		let mut res = [0u8; 32];
		res[0..16].copy_from_slice(&Twox128::hash(pallet_name));
		res[16..32].copy_from_slice(&Twox128::hash(storage_name));
		res.to_vec()
	};
	assert_eq!(
		<moonbase_runtime::Timestamp as StorageInfoTrait>::storage_info(),
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
				storage_name: b"Account".to_vec(),
				prefix: prefix(b"Balances", b"Account"),
				max_values: Some(300_000),
				max_size: Some(100),
			},
			StorageInfo {
				pallet_name: b"Balances".to_vec(),
				storage_name: b"Locks".to_vec(),
				prefix: prefix(b"Balances", b"Locks"),
				max_values: Some(300_000),
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
			max_size: Some(1),
		},]
	);
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
	is_pallet_index::<moonbase_runtime::Timestamp>(2);
	is_pallet_index::<moonbase_runtime::Balances>(3);
	is_pallet_index::<moonbase_runtime::Sudo>(4);
	is_pallet_index::<moonbase_runtime::RandomnessCollectiveFlip>(5);
	is_pallet_index::<moonbase_runtime::ParachainSystem>(6);
	is_pallet_index::<moonbase_runtime::TransactionPayment>(7);
	is_pallet_index::<moonbase_runtime::ParachainInfo>(8);
	is_pallet_index::<moonbase_runtime::EthereumChainId>(9);
	is_pallet_index::<moonbase_runtime::EVM>(10);
	is_pallet_index::<moonbase_runtime::Ethereum>(11);
	is_pallet_index::<moonbase_runtime::ParachainStaking>(12);
	is_pallet_index::<moonbase_runtime::Scheduler>(13);
	is_pallet_index::<moonbase_runtime::Democracy>(14);
	is_pallet_index::<moonbase_runtime::CouncilCollective>(15);
	is_pallet_index::<moonbase_runtime::TechCommitteeCollective>(16);
	is_pallet_index::<moonbase_runtime::Treasury>(17);
	is_pallet_index::<moonbase_runtime::AuthorInherent>(18);
	is_pallet_index::<moonbase_runtime::AuthorFilter>(19);
	is_pallet_index::<moonbase_runtime::CrowdloanRewards>(20);
	is_pallet_index::<moonbase_runtime::AuthorMapping>(21);
	is_pallet_index::<moonbase_runtime::Proxy>(22);
	is_pallet_index::<moonbase_runtime::MaintenanceMode>(23);
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
		.with_nominations(vec![
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
				parachain_staking::Error::<Runtime>::CandidateExists
			);
			assert_noop!(
				ParachainStaking::join_candidates(
					origin_of(AccountId::from(CHARLIE)),
					1_000 * UNIT,
					2u32
				),
				parachain_staking::Error::<Runtime>::NominatorExists
			);
			assert!(System::events().is_empty());
			assert_ok!(ParachainStaking::join_candidates(
				origin_of(AccountId::from(DAVE)),
				1_000 * UNIT,
				2u32
			));
			assert_eq!(
				last_event(),
				Event::ParachainStaking(parachain_staking::Event::JoinedCollatorCandidates(
					AccountId::from(DAVE),
					1_000 * UNIT,
					3_100 * UNIT
				))
			);
			let candidates = ParachainStaking::candidate_pool();
			assert_eq!(
				candidates.0[0],
				Bond {
					owner: AccountId::from(ALICE),
					amount: 1_050 * UNIT
				}
			);
			assert_eq!(
				candidates.0[1],
				Bond {
					owner: AccountId::from(BOB),
					amount: 1_050 * UNIT
				}
			);
			assert_eq!(
				candidates.0[2],
				Bond {
					owner: AccountId::from(DAVE),
					amount: 1_000 * UNIT
				}
			);
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
				DispatchError::Module {
					index: 3,
					error: 2,
					message: Some("InsufficientBalance")
				}
			);

			// Alice transfer from free balance 2000 UNIT to Bob
			assert_ok!(Balances::transfer(
				origin_of(AccountId::from(ALICE)),
				AccountId::from(BOB),
				2_000 * UNIT,
			));
			assert_eq!(Balances::free_balance(AccountId::from(BOB)), 2_000 * UNIT);

			let gas_limit = 100000u64;
			let gas_price: U256 = 1_000_000_000.into();
			// Bob transfers 1000 UNIT to Charlie via EVM
			assert_ok!(Call::EVM(pallet_evm::Call::<Runtime>::call {
				source: AccountId::from(BOB),
				target: AccountId::from(CHARLIE),
				input: Vec::new(),
				value: (1_000 * UNIT).into(),
				gas_limit,
				gas_price,
				nonce: None
			})
			.dispatch(<Runtime as frame_system::Config>::Origin::root()));
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
			assert_eq!(
				candidates.0[0],
				Bond {
					owner: AccountId::from(CHARLIE),
					amount: 1_000 * UNIT
				}
			);
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
		.with_nominations(vec![(
			AccountId::from(BOB),
			AccountId::from(ALICE),
			500 * UNIT,
		)])
		.with_mappings(vec![(
			NimbusId::from_slice(&ALICE_NIMBUS),
			AccountId::from(ALICE),
		)])
		.build()
		.execute_with(|| {
			set_parachain_inherent_data();
			for x in 2..599 {
				set_author(NimbusId::from_slice(&ALICE_NIMBUS));
				run_to_block(x);
			}
			// no rewards doled out yet
			assert_eq!(Balances::free_balance(AccountId::from(ALICE)), 1_000 * UNIT,);
			assert_eq!(Balances::free_balance(AccountId::from(BOB)), 500 * UNIT,);
			set_author(NimbusId::from_slice(&ALICE_NIMBUS));
			run_to_block(600);
			// rewards minted and distributed
			assert_eq!(
				Balances::free_balance(AccountId::from(ALICE)),
				1113666666584000000000,
			);
			assert_eq!(
				Balances::free_balance(AccountId::from(BOB)),
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
		.with_nominations(vec![(
			AccountId::from(BOB),
			AccountId::from(ALICE),
			500 * UNIT,
		)])
		.with_mappings(vec![(
			NimbusId::from_slice(&ALICE_NIMBUS),
			AccountId::from(ALICE),
		)])
		.build()
		.execute_with(|| {
			set_parachain_inherent_data();
			assert_ok!(ParachainStaking::set_parachain_bond_account(
				root_origin(),
				AccountId::from(CHARLIE),
			),);
			for x in 2..599 {
				set_author(NimbusId::from_slice(&ALICE_NIMBUS));
				run_to_block(x);
			}
			// no rewards doled out yet
			assert_eq!(Balances::free_balance(AccountId::from(ALICE)), 1_000 * UNIT,);
			assert_eq!(Balances::free_balance(AccountId::from(BOB)), 500 * UNIT,);
			assert_eq!(Balances::free_balance(AccountId::from(CHARLIE)), UNIT,);
			set_author(NimbusId::from_slice(&ALICE_NIMBUS));
			run_to_block(600);
			// rewards minted and distributed
			assert_eq!(
				Balances::free_balance(AccountId::from(ALICE)),
				1082693333281650000000,
			);
			assert_eq!(
				Balances::free_balance(AccountId::from(BOB)),
				525841666640825000000,
			);
			// 30% reserved for parachain bond
			assert_eq!(
				Balances::free_balance(AccountId::from(CHARLIE)),
				47515000000000000000,
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
			NimbusId::from_slice(&ALICE_NIMBUS),
			AccountId::from(ALICE),
		)])
		.with_crowdloan_fund(3_000_000 * UNIT)
		.build()
		.execute_with(|| {
			// set parachain inherent data
			set_parachain_inherent_data();
			set_author(NimbusId::from_slice(&ALICE_NIMBUS));
			for x in 1..3 {
				run_to_block(x);
			}
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
								1_500_000 * UNIT
							)]
						}
					),
					Call::CrowdloanRewards(
						pallet_crowdloan_rewards::Call::<Runtime>::initialize_reward_vec {
							rewards: vec![(
								[5u8; 32].into(),
								Some(AccountId::from(DAVE)),
								1_500_000 * UNIT
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
			assert_eq!(Balances::balance(&AccountId::from(CHARLIE)), 450_000 * UNIT);
			// 30 percent initial payout
			assert_eq!(Balances::balance(&AccountId::from(DAVE)), 450_000 * UNIT);
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
			let expected_fail = Event::Utility(pallet_utility::Event::BatchInterrupted(
				0,
				DispatchError::Module {
					index: 20,
					error: 8,
					message: None,
				},
			));
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
			NimbusId::from_slice(&ALICE_NIMBUS),
			AccountId::from(ALICE),
		)])
		.with_crowdloan_fund(3_000_000 * UNIT)
		.build()
		.execute_with(|| {
			// set parachain inherent data
			set_parachain_inherent_data();
			set_author(NimbusId::from_slice(&ALICE_NIMBUS));
			for x in 1..3 {
				run_to_block(x);
			}
			let init_block = CrowdloanRewards::init_vesting_block();
			// This matches the previous vesting
			let end_block = init_block + 4 * WEEKS;

			let (pair1, _) = sp_core::sr25519::Pair::generate();
			let (pair2, _) = sp_core::sr25519::Pair::generate();

			let public1 = pair1.public();
			let public2 = pair2.public();

			// signature is new_account || previous_account
			let mut message = pallet_crowdloan_rewards::WRAPPED_BYTES_PREFIX.to_vec();
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
									1_500_000 * UNIT
								)]
							}
						),
						Call::CrowdloanRewards(
							pallet_crowdloan_rewards::Call::<Runtime>::initialize_reward_vec {
								rewards: vec![(
									public2.into(),
									Some(AccountId::from(CHARLIE)),
									1_500_000 * UNIT
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
			NimbusId::from_slice(&ALICE_NIMBUS),
			AccountId::from(ALICE),
		)])
		.with_crowdloan_fund(3_000_000 * UNIT)
		.build()
		.execute_with(|| {
			// set parachain inherent data
			set_parachain_inherent_data();
			set_author(NimbusId::from_slice(&ALICE_NIMBUS));
			for x in 1..3 {
				run_to_block(x);
			}
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
								1_500_000 * UNIT
							)]
						}
					),
					Call::CrowdloanRewards(
						pallet_crowdloan_rewards::Call::<Runtime>::initialize_reward_vec {
							rewards: vec![(
								[5u8; 32].into(),
								Some(AccountId::from(DAVE)),
								1_500_000 * UNIT
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
			assert_eq!(Balances::balance(&AccountId::from(CHARLIE)), 450_000 * UNIT);
			// 30 percent initial payout
			assert_eq!(Balances::balance(&AccountId::from(DAVE)), 450_000 * UNIT);

			let crowdloan_precompile_address = H160::from_low_u64_be(2049);

			// Alice uses the crowdloan precompile to claim through the EVM
			let gas_limit = 100000u64;
			let gas_price: U256 = 1_000_000_000.into();

			// Construct the call data (selector, amount)
			let mut call_data = Vec::<u8>::from([0u8; 4]);
			call_data[0..4].copy_from_slice(&Keccak256::digest(b"claim()")[0..4]);

			assert_ok!(Call::EVM(pallet_evm::Call::<Runtime>::call {
				source: AccountId::from(CHARLIE),
				target: crowdloan_precompile_address,
				input: call_data,
				value: U256::zero(), // No value sent in EVM
				gas_limit,
				gas_price,
				nonce: None, // Use the next nonce
			})
			.dispatch(<Runtime as frame_system::Config>::Origin::root()));

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
			NimbusId::from_slice(&ALICE_NIMBUS),
			AccountId::from(ALICE),
		)])
		.with_crowdloan_fund(3_000_000 * UNIT)
		.build()
		.execute_with(|| {
			// set parachain inherent data
			set_parachain_inherent_data();
			set_author(NimbusId::from_slice(&ALICE_NIMBUS));
			for x in 1..3 {
				run_to_block(x);
			}
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
								1_500_000 * UNIT
							)]
						}
					),
					Call::CrowdloanRewards(
						pallet_crowdloan_rewards::Call::<Runtime>::initialize_reward_vec {
							rewards: vec![(
								[5u8; 32].into(),
								Some(AccountId::from(DAVE)),
								1_500_000 * UNIT
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

			// Construct the input data to check if Bob is a contributor
			let mut bob_input_data = Vec::<u8>::from([0u8; 36]);
			bob_input_data[0..4]
				.copy_from_slice(&Keccak256::digest(b"is_contributor(address)")[0..4]);
			bob_input_data[16..36].copy_from_slice(&BOB);

			// Expected result is an EVM boolean false which is 256 bits long.
			let mut expected_bytes = Vec::from([0u8; 32]);
			expected_bytes[31] = 0;
			let expected_false_result = Some(Ok(PrecompileOutput {
				exit_status: ExitSucceed::Returned,
				output: expected_bytes,
				cost: 1000,
				logs: Default::default(),
			}));

			// Assert precompile reports Bob is not a contributor
			assert_eq!(
				Precompiles::execute(
					crowdloan_precompile_address,
					&bob_input_data,
					None, // target_gas is not necessary right now because consumed none now
					&evm_test_context(),
				),
				expected_false_result
			);

			// Construct the input data to check if Charlie is a contributor
			let mut charlie_input_data = Vec::<u8>::from([0u8; 36]);
			charlie_input_data[0..4]
				.copy_from_slice(&Keccak256::digest(b"is_contributor(address)")[0..4]);
			charlie_input_data[16..36].copy_from_slice(&CHARLIE);

			// Expected result is an EVM boolean true which is 256 bits long.
			let mut expected_bytes = Vec::from([0u8; 32]);
			expected_bytes[31] = 1;
			let expected_true_result = Some(Ok(PrecompileOutput {
				exit_status: ExitSucceed::Returned,
				output: expected_bytes,
				cost: 1000,
				logs: Default::default(),
			}));

			// Assert precompile reports Bob is a nominator
			assert_eq!(
				Precompiles::execute(
					crowdloan_precompile_address,
					&charlie_input_data,
					None, // target_gas is not necessary right now because consumed none now
					&evm_test_context(),
				),
				expected_true_result
			);
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
			NimbusId::from_slice(&ALICE_NIMBUS),
			AccountId::from(ALICE),
		)])
		.with_crowdloan_fund(3_000_000 * UNIT)
		.build()
		.execute_with(|| {
			// set parachain inherent data
			set_parachain_inherent_data();
			set_author(NimbusId::from_slice(&ALICE_NIMBUS));
			for x in 1..3 {
				run_to_block(x);
			}
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
								1_500_000 * UNIT
							)]
						}
					),
					Call::CrowdloanRewards(
						pallet_crowdloan_rewards::Call::<Runtime>::initialize_reward_vec {
							rewards: vec![(
								[5u8; 32].into(),
								Some(AccountId::from(DAVE)),
								1_500_000 * UNIT
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

			// Construct the input data to check if Bob is a contributor
			let mut charlie_input_data = Vec::<u8>::from([0u8; 36]);
			charlie_input_data[0..4]
				.copy_from_slice(&Keccak256::digest(b"reward_info(address)")[0..4]);
			charlie_input_data[16..36].copy_from_slice(&CHARLIE);

			let expected_total: U256 = (1_500_000 * UNIT).into();
			let expected_claimed: U256 = (450_000 * UNIT).into();

			// Expected result is two EVM u256 false which are 256 bits long.
			let mut expected_bytes = Vec::from([0u8; 64]);
			expected_total.to_big_endian(&mut expected_bytes[0..32]);
			expected_claimed.to_big_endian(&mut expected_bytes[32..64]);
			let expected_result = Some(Ok(PrecompileOutput {
				exit_status: ExitSucceed::Returned,
				output: expected_bytes,
				cost: 1000,
				logs: Default::default(),
			}));

			// Assert precompile reports Bob is not a contributor
			assert_eq!(
				Precompiles::execute(
					crowdloan_precompile_address,
					&charlie_input_data,
					None, // target_gas is not necessary right now because consumed none now
					&evm_test_context(),
				),
				expected_result
			);
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
			NimbusId::from_slice(&ALICE_NIMBUS),
			AccountId::from(ALICE),
		)])
		.with_crowdloan_fund(3_000_000 * UNIT)
		.build()
		.execute_with(|| {
			// set parachain inherent data
			set_parachain_inherent_data();
			set_author(NimbusId::from_slice(&ALICE_NIMBUS));
			for x in 1..3 {
				run_to_block(x);
			}
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
								1_500_000 * UNIT
							)]
						}
					),
					Call::CrowdloanRewards(
						pallet_crowdloan_rewards::Call::<Runtime>::initialize_reward_vec {
							rewards: vec![(
								[5u8; 32].into(),
								Some(AccountId::from(DAVE)),
								1_500_000 * UNIT
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
			let gas_price: U256 = 1_000_000_000.into();

			// Construct the input data to check if Bob is a contributor
			let mut call_data = Vec::<u8>::from([0u8; 36]);
			call_data[0..4]
				.copy_from_slice(&Keccak256::digest(b"update_reward_address(address)")[0..4]);
			call_data[16..36].copy_from_slice(&ALICE);

			assert_ok!(Call::EVM(pallet_evm::Call::<Runtime>::call {
				source: AccountId::from(CHARLIE),
				target: crowdloan_precompile_address,
				input: call_data,
				value: U256::zero(), // No value sent in EVM
				gas_limit,
				gas_price,
				nonce: None, // Use the next nonce
			})
			.dispatch(<Runtime as frame_system::Config>::Origin::root()));

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
fn asset_can_be_registered() {
	ExtBuilder::default().build().execute_with(|| {
		let source_location = AssetType::Xcm(MultiLocation::parent());
		let source_id: moonbase_runtime::AssetId = source_location.clone().into();
		let asset_metadata = moonbase_runtime::AssetRegistrarMetadata {
			name: b"RelayToken".to_vec(),
			symbol: b"Relay".to_vec(),
			decimals: 12,
			is_frozen: false,
		};
		assert_ok!(AssetManager::register_asset(
			moonbase_runtime::Origin::root(),
			source_location,
			asset_metadata,
			1u128,
		));
		assert!(AssetManager::asset_id_type(source_id).is_some());
	});
}

#[test]
fn asset_erc20_precompiles_supply_and_balance() {
	ExtBuilder::default()
		.with_assets(vec![(0u128, vec![(AccountId::from(ALICE), 1_000 * UNIT)])])
		.build()
		.execute_with(|| {
			// Assert the asset has been created with the correct supply
			assert_eq!(Assets::total_supply(0u128), 1_000 * UNIT);

			// Convert the assetId to its corresponding precompile address
			let asset_precompile_address = Runtime::asset_id_to_account(0u128);

			// The expected result for both total supply and balance of is the same, as only Alice
			// holds balance
			let expected_result = Some(Ok(PrecompileOutput {
				exit_status: ExitSucceed::Returned,
				output: EvmDataWriter::new().write(U256::from(1000 * UNIT)).build(),
				cost: 1000,
				logs: Default::default(),
			}));

			// Access totalSupply through precompile. Important that the context is correct
			assert_eq!(
				Precompiles::execute(
					asset_precompile_address,
					&EvmDataWriter::new_with_selector(AssetAction::TotalSupply).build(),
					None,
					&evm::Context {
						address: asset_precompile_address,
						caller: ALICE.into(),
						apparent_value: From::from(0),
					},
				),
				expected_result
			);

			// Access balanceOf through precompile
			assert_eq!(
				Precompiles::execute(
					asset_precompile_address,
					&EvmDataWriter::new_with_selector(AssetAction::BalanceOf)
						.write(EvmAddress(ALICE.into()))
						.build(),
					None,
					&evm::Context {
						address: asset_precompile_address,
						caller: ALICE.into(),
						apparent_value: From::from(0),
					},
				),
				expected_result
			);
		});
}

#[test]
fn asset_erc20_precompiles_transfer() {
	ExtBuilder::default()
		.with_assets(vec![(0u128, vec![(AccountId::from(ALICE), 1_000 * UNIT)])])
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * UNIT),
			(AccountId::from(BOB), 1_000 * UNIT),
		])
		.build()
		.execute_with(|| {
			let asset_precompile_address = Runtime::asset_id_to_account(0u128);

			// Expected result for a transfer
			let expected_result = Some(Ok(PrecompileOutput {
				exit_status: ExitSucceed::Returned,
				output: Default::default(),
				cost: 25084u64,
				logs: LogsBuilder::new(asset_precompile_address)
					.log3(
						SELECTOR_LOG_TRANSFER,
						H160::from(ALICE),
						H160::from(BOB),
						EvmDataWriter::new().write(U256::from(400 * UNIT)).build(),
					)
					.build(),
			}));

			// Transfer tokens from Aice to Bob, 400 unit.
			assert_eq!(
				Precompiles::execute(
					asset_precompile_address,
					&EvmDataWriter::new_with_selector(AssetAction::Transfer)
						.write(EvmAddress(BOB.into()))
						.write(U256::from(400 * UNIT))
						.build(),
					None,
					&evm::Context {
						address: asset_precompile_address,
						caller: ALICE.into(),
						apparent_value: From::from(0),
					},
				),
				expected_result
			);

			// Expected result for balanceOf BOB
			let expected_result = Some(Ok(PrecompileOutput {
				exit_status: ExitSucceed::Returned,
				output: EvmDataWriter::new().write(U256::from(400 * UNIT)).build(),
				cost: 1000,
				logs: Default::default(),
			}));

			// Make sure BOB has 400 unit
			assert_eq!(
				Precompiles::execute(
					asset_precompile_address,
					&EvmDataWriter::new_with_selector(AssetAction::BalanceOf)
						.write(EvmAddress(BOB.into()))
						.build(),
					None,
					&evm::Context {
						address: asset_precompile_address,
						caller: BOB.into(),
						apparent_value: From::from(0),
					},
				),
				expected_result
			);
		});
}

#[test]
fn asset_erc20_precompiles_approve() {
	ExtBuilder::default()
		.with_assets(vec![(0u128, vec![(AccountId::from(ALICE), 1_000 * UNIT)])])
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * UNIT),
			(AccountId::from(BOB), 1_000 * UNIT),
		])
		.build()
		.execute_with(|| {
			let asset_precompile_address = Runtime::asset_id_to_account(0u128);

			// Expected result for approve
			let expected_result = Some(Ok(PrecompileOutput {
				exit_status: ExitSucceed::Returned,
				output: Default::default(),
				cost: 16035u64,
				logs: LogsBuilder::new(asset_precompile_address)
					.log3(
						SELECTOR_LOG_APPROVAL,
						H160::from(ALICE),
						H160::from(BOB),
						EvmDataWriter::new().write(U256::from(400 * UNIT)).build(),
					)
					.build(),
			}));

			// Aprove Bob for spending 400 unit from Alice
			assert_eq!(
				Precompiles::execute(
					asset_precompile_address,
					&EvmDataWriter::new_with_selector(AssetAction::Approve)
						.write(EvmAddress(BOB.into()))
						.write(U256::from(400 * UNIT))
						.build(),
					None,
					&evm::Context {
						address: asset_precompile_address,
						caller: ALICE.into(),
						apparent_value: From::from(0),
					},
				),
				expected_result
			);

			// Expected result for transfer_from
			let expected_result = Some(Ok(PrecompileOutput {
				exit_status: ExitSucceed::Returned,
				output: Default::default(),
				cost: 31042u64,
				logs: LogsBuilder::new(asset_precompile_address)
					.log3(
						SELECTOR_LOG_TRANSFER,
						H160::from(ALICE),
						H160::from(CHARLIE),
						EvmDataWriter::new().write(U256::from(400 * UNIT)).build(),
					)
					.build(),
			}));

			// Transfer tokens from Alice to Charlie by using BOB as origin
			assert_eq!(
				Precompiles::execute(
					asset_precompile_address,
					&EvmDataWriter::new_with_selector(AssetAction::TransferFrom)
						.write(EvmAddress(ALICE.into()))
						.write(EvmAddress(CHARLIE.into()))
						.write(U256::from(400 * UNIT))
						.build(),
					None,
					&evm::Context {
						address: asset_precompile_address,
						caller: BOB.into(),
						apparent_value: From::from(0),
					},
				),
				expected_result
			);

			// Expected result for balance of CHARLIE
			let expected_result = Some(Ok(PrecompileOutput {
				exit_status: ExitSucceed::Returned,
				output: EvmDataWriter::new().write(U256::from(400 * UNIT)).build(),
				cost: 1000,
				logs: Default::default(),
			}));

			// Make sure CHARLIE has 400 unit
			assert_eq!(
				Precompiles::execute(
					asset_precompile_address,
					&EvmDataWriter::new_with_selector(AssetAction::BalanceOf)
						.write(EvmAddress(CHARLIE.into()))
						.build(),
					None,
					&evm::Context {
						address: asset_precompile_address,
						caller: CHARLIE.into(),
						apparent_value: From::from(0),
					},
				),
				expected_result
			);
		});
}

#[test]
fn xtokens_precompiles_transfer() {
	ExtBuilder::default()
		.with_xcm_assets(vec![(
			AssetType::Xcm(MultiLocation::parent()),
			AssetRegistrarMetadata {
				name: b"RelayToken".to_vec(),
				symbol: b"Relay".to_vec(),
				decimals: 12,
				is_frozen: false,
			},
			vec![(AccountId::from(ALICE), 1_000_000_000_000_000)],
		)])
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * UNIT),
			(AccountId::from(BOB), 1_000 * UNIT),
		])
		.build()
		.execute_with(|| {
			let xtokens_precompile_address = H160::from_low_u64_be(2052);

			// We have the assetId that corresponds to the relay chain registered
			let relay_asset_id: AssetId = AssetType::Xcm(MultiLocation::parent()).into();

			// Its address is
			let asset_precompile_address = Runtime::asset_id_to_account(relay_asset_id);

			// Alice has 1000 tokens. She should be able to send through precompile
			let destination = MultiLocation::new(
				1,
				Junctions::X1(Junction::AccountId32 {
					network: NetworkId::Any,
					id: [1u8; 32],
				}),
			);

			// We use the address of the asset as an identifier of the asset we want to transferS
			assert_eq!(
				Precompiles::execute(
					xtokens_precompile_address,
					&EvmDataWriter::new_with_selector(XtokensAction::Transfer)
						.write(EvmAddress(asset_precompile_address))
						.write(U256::from(500_000_000_000_000u128))
						.write(destination.clone())
						.write(U256::from(4000000))
						.build(),
					None,
					&evm::Context {
						address: xtokens_precompile_address,
						caller: ALICE.into(),
						apparent_value: From::from(0),
					},
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					cost: 12000,
					output: vec![],
					logs: vec![]
				}))
			);
		})
}

#[test]
fn xtokens_precompiles_transfer_multiasset() {
	ExtBuilder::default()
		.with_xcm_assets(vec![(
			AssetType::Xcm(MultiLocation::parent()),
			AssetRegistrarMetadata {
				name: b"RelayToken".to_vec(),
				symbol: b"Relay".to_vec(),
				decimals: 12,
				is_frozen: false,
			},
			vec![(AccountId::from(ALICE), 1_000_000_000_000_000)],
		)])
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * UNIT),
			(AccountId::from(BOB), 1_000 * UNIT),
		])
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
			assert_eq!(
				Precompiles::execute(
					xtokens_precompile_address,
					&EvmDataWriter::new_with_selector(XtokensAction::TransferMultiAsset)
						// We want to transfer the relay token
						.write(MultiLocation::parent())
						.write(U256::from(500_000_000_000_000u128))
						.write(destination)
						.write(U256::from(4000000))
						.build(),
					None,
					&evm::Context {
						address: xtokens_precompile_address,
						caller: ALICE.into(),
						apparent_value: From::from(0),
					},
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					cost: 12000,
					output: vec![],
					logs: vec![]
				}))
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
fn multiplier_can_grow_from_zero() {
	let minimum_multiplier = moonbase_runtime::MinimumMultiplier::get();
	let target = moonbase_runtime::TargetBlockFullness::get()
		* BlockWeights::get()
			.get(DispatchClass::Normal)
			.max_total
			.unwrap();
	// if the min is too small, then this will not change, and we are doomed forever.
	// the weight is 1/100th bigger than target.
	run_with_system_weight(target * 101 / 100, || {
		let next = moonbase_runtime::SlowAdjustingFeeUpdate::<Runtime>::convert(minimum_multiplier);
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
	let mut multiplier = moonbase_runtime::MinimumMultiplier::get();
	let block_weight = moonbase_runtime::TargetBlockFullness::get()
		* BlockWeights::get()
			.get(DispatchClass::Normal)
			.max_total
			.unwrap()
		* 2;
	let mut blocks = 0;
	while multiplier <= Multiplier::one() {
		run_with_system_weight(block_weight, || {
			let next = moonbase_runtime::SlowAdjustingFeeUpdate::<Runtime>::convert(multiplier);
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
