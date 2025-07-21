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

use crate::{
	xcm_mock::{parachain::PolkadotXcm, *},
	xcm_testing::{add_supported_asset, currency_to_asset},
};
use frame_support::assert_ok;
use sp_weights::Weight;
use xcm::{
	latest::prelude::{AccountId32, AccountKey20, Junction, Location, Parachain, WeightLimit},
	VersionedAssets, VersionedLocation,
};
use xcm_primitives::{split_location_into_chain_part_and_beneficiary, DEFAULT_PROOF_SIZE};
use xcm_simulator::TestExt;

// Send a relay asset (like DOT) to a parachain A
#[test]
fn receive_relay_asset_from_relay() {
	MockNet::reset();

	let source_location = parachain::AssetType::Xcm(xcm::v3::Location::parent());
	let source_id: parachain::AssetId = source_location.clone().into();
	let asset_metadata = parachain::AssetMetadata {
		name: b"RelayToken".to_vec(),
		symbol: b"Relay".to_vec(),
		decimals: 12,
	};
	// register relay asset in parachain A
	ParaA::execute_with(|| {
		assert_ok!(AssetManager::register_foreign_asset(
			parachain::RuntimeOrigin::root(),
			source_location.clone(),
			asset_metadata,
			1u128,
			true
		));
		assert_ok!(add_supported_asset(source_location.clone(), 0));
	});

	// Actually send relay asset to parachain
	let dest: Location = AccountKey20 {
		network: None,
		key: PARAALICE,
	}
	.into();
	Relay::execute_with(|| {
		assert_ok!(RelayChainPalletXcm::limited_reserve_transfer_assets(
			relay_chain::RuntimeOrigin::signed(RELAYALICE),
			Box::new(Parachain(1).into()),
			Box::new(VersionedLocation::from(dest).clone()),
			Box::new(([], 123).into()),
			0,
			WeightLimit::Unlimited
		));
	});

	// Verify that parachain received the asset
	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 123);
	});
}

// Send relay asset (like DOT) back from Parachain A to relaychain
#[test]
fn send_relay_asset_to_relay() {
	MockNet::reset();

	let source_location = parachain::AssetType::Xcm(xcm::v3::Location::parent());
	let source_id: parachain::AssetId = source_location.clone().into();

	let asset_metadata = parachain::AssetMetadata {
		name: b"RelayToken".to_vec(),
		symbol: b"Relay".to_vec(),
		decimals: 12,
	};

	// Register relay asset in paraA
	ParaA::execute_with(|| {
		assert_ok!(AssetManager::register_foreign_asset(
			parachain::RuntimeOrigin::root(),
			source_location.clone(),
			asset_metadata,
			1u128,
			true
		));
		// Free execution
		assert_ok!(add_supported_asset(source_location, 0));
	});

	let dest: Location = Junction::AccountKey20 {
		network: None,
		key: PARAALICE,
	}
	.into();

	// First send relay chain asset to Parachain like in previous test
	Relay::execute_with(|| {
		assert_ok!(RelayChainPalletXcm::limited_reserve_transfer_assets(
			relay_chain::RuntimeOrigin::signed(RELAYALICE),
			Box::new(Parachain(1).into()),
			Box::new(VersionedLocation::from(dest.clone()).into()),
			Box::new(([], 123).into()),
			0,
			WeightLimit::Unlimited
		));
	});

	ParaA::execute_with(|| {
		// Free execution, full amount received
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 123);
	});

	// Lets gather the balance before sending back money
	let mut balance_before_sending = 0;
	Relay::execute_with(|| {
		balance_before_sending = RelayBalances::free_balance(&RELAYALICE);
	});

	// We now send back some money to the relay
	let dest = Location {
		parents: 1,
		interior: [AccountId32 {
			network: None,
			id: RELAYALICE.into(),
		}]
		.into(),
	};
	let (chain_part, beneficiary) = split_location_into_chain_part_and_beneficiary(dest).unwrap();

	ParaA::execute_with(|| {
		let asset = currency_to_asset(parachain::CurrencyId::ForeignAsset(source_id), 123);
		// free execution, full amount received
		assert_ok!(PolkadotXcm::transfer_assets(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			Box::new(VersionedLocation::from(chain_part)),
			Box::new(VersionedLocation::from(beneficiary)),
			Box::new(VersionedAssets::from(asset)),
			0,
			WeightLimit::Limited(Weight::from_parts(40000u64, DEFAULT_PROOF_SIZE))
		));
	});

	// The balances in paraAlice should have been substracted
	ParaA::execute_with(|| {
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 0);
	});

	// Balances in the relay should have been received
	Relay::execute_with(|| {
		// Free execution,x	 full amount received
		assert!(RelayBalances::free_balance(&RELAYALICE) > balance_before_sending);
	});
}

#[test]
fn send_relay_asset_to_para_b() {
	MockNet::reset();

	let source_location = parachain::AssetType::Xcm(xcm::v3::Location::parent());
	let source_id: parachain::AssetId = source_location.clone().into();

	let asset_metadata = parachain::AssetMetadata {
		name: b"RelayToken".to_vec(),
		symbol: b"Relay".to_vec(),
		decimals: 12,
	};

	// Register asset in paraA. Free execution
	ParaA::execute_with(|| {
		assert_ok!(AssetManager::register_foreign_asset(
			parachain::RuntimeOrigin::root(),
			source_location.clone(),
			asset_metadata.clone(),
			1u128,
			true
		));
		assert_ok!(add_supported_asset(source_location.clone(), 0));
	});

	// Register asset in paraB. Free execution
	ParaB::execute_with(|| {
		assert_ok!(AssetManager::register_foreign_asset(
			parachain::RuntimeOrigin::root(),
			source_location.clone(),
			asset_metadata,
			1u128,
			true
		));
		assert_ok!(add_supported_asset(source_location, 0));
	});

	// First send relay chain asset to Parachain A like in previous test
	let dest: Location = Junction::AccountKey20 {
		network: None,
		key: PARAALICE,
	}
	.into();
	Relay::execute_with(|| {
		assert_ok!(RelayChainPalletXcm::limited_reserve_transfer_assets(
			relay_chain::RuntimeOrigin::signed(RELAYALICE),
			Box::new(Parachain(1).into()),
			Box::new(VersionedLocation::from(dest).clone()),
			Box::new(([], 123).into()),
			0,
			WeightLimit::Unlimited
		));
	});

	ParaA::execute_with(|| {
		// Free execution, full amount received
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 123);
	});

	// Now send relay asset from para A to para B
	let dest = Location {
		parents: 1,
		interior: [
			Parachain(2),
			AccountKey20 {
				network: None,
				key: PARAALICE.into(),
			},
		]
		.into(),
	};
	let (chain_part, beneficiary) = split_location_into_chain_part_and_beneficiary(dest).unwrap();

	ParaA::execute_with(|| {
		let asset = currency_to_asset(parachain::CurrencyId::ForeignAsset(source_id), 100);
		assert_ok!(PolkadotXcm::transfer_assets(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			Box::new(VersionedLocation::from(chain_part)),
			Box::new(VersionedLocation::from(beneficiary)),
			Box::new(VersionedAssets::from(vec![asset])),
			0,
			WeightLimit::Limited(Weight::from_parts(40000u64, DEFAULT_PROOF_SIZE))
		));
	});

	// Para A balances should have been substracted
	ParaA::execute_with(|| {
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 23);
	});

	// Para B balances should have been credited
	ParaB::execute_with(|| {
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 100);
	});
}

#[test]
fn receive_relay_asset_with_trader() {
	MockNet::reset();

	let source_location = parachain::AssetType::Xcm(xcm::v3::Location::parent());
	let source_id: parachain::AssetId = source_location.clone().into();

	let asset_metadata = parachain::AssetMetadata {
		name: b"RelayToken".to_vec(),
		symbol: b"Relay".to_vec(),
		decimals: 12,
	};

	// This time we are gonna put a rather high number of units per second
	// we know later we will divide by 1e12
	// Lets put 1e6 as units per second
	ParaA::execute_with(|| {
		assert_ok!(AssetManager::register_foreign_asset(
			parachain::RuntimeOrigin::root(),
			source_location.clone(),
			asset_metadata,
			1u128,
			true
		));
		assert_ok!(add_supported_asset(source_location, 2_500_000_000_000));
	});

	let dest: Location = Junction::AccountKey20 {
		network: None,
		key: PARAALICE,
	}
	.into();
	// We are sending 100 tokens from relay.
	// Amount spent in fees is Units per second * weight / 1_000_000_000_000 (weight per second)
	// weight is 4 since we are executing 4 instructions with a unitweightcost of 1.
	// Units per second should be 2_500_000_000_000_000
	// Therefore with no refund, we should receive 10 tokens less
	// Native trader fails for this, and we use the asset trader
	Relay::execute_with(|| {
		assert_ok!(RelayChainPalletXcm::limited_reserve_transfer_assets(
			relay_chain::RuntimeOrigin::signed(RELAYALICE),
			Box::new(Parachain(1).into()),
			Box::new(VersionedLocation::from(dest).clone()),
			Box::new(([], 100).into()),
			0,
			WeightLimit::Unlimited
		));
	});

	ParaA::execute_with(|| {
		// non-free execution, not full amount received
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 90);
		// Fee should have been received by treasury
		assert_eq!(Assets::balance(source_id, &Treasury::account_id()), 10);
	});
}

#[test]
fn error_when_not_paying_enough() {
	MockNet::reset();

	let source_location = parachain::AssetType::Xcm(xcm::v3::Location::parent());
	let source_id: parachain::AssetId = source_location.clone().into();

	let asset_metadata = parachain::AssetMetadata {
		name: b"RelayToken".to_vec(),
		symbol: b"Relay".to_vec(),
		decimals: 12,
	};

	let dest: Location = Junction::AccountKey20 {
		network: None,
		key: PARAALICE,
	}
	.into();
	// This time we are gonna put a rather high number of units per second
	// we know later we will divide by 1e12
	// Lets put 1e6 as units per second
	ParaA::execute_with(|| {
		assert_ok!(AssetManager::register_foreign_asset(
			parachain::RuntimeOrigin::root(),
			source_location.clone(),
			asset_metadata,
			1u128,
			true
		));
		assert_ok!(add_supported_asset(source_location, 2500000000000));
	});

	ParaA::execute_with(|| {
		// amount not received as it is not paying enough
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 0);
	});

	// We are sending 100 tokens from relay.
	// If we set the dest weight to be 1e7, we know the buy_execution will spend 1e7*1e6/1e12 = 10
	// Therefore with no refund, we should receive 10 tokens less
	Relay::execute_with(|| {
		assert_ok!(RelayChainPalletXcm::limited_reserve_transfer_assets(
			relay_chain::RuntimeOrigin::signed(RELAYALICE),
			Box::new(Parachain(1).into()),
			Box::new(VersionedLocation::from(dest).clone()),
			Box::new(([], 5).into()),
			0,
			WeightLimit::Unlimited
		));
	});

	ParaA::execute_with(|| {
		// amount not received as it is not paying enough
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 0);
	});
}

#[test]
fn receive_asset_with_no_sufficients_not_possible_if_non_existent_account() {
	MockNet::reset();

	let fresh_account = [2u8; 20];
	let source_location = parachain::AssetType::Xcm(xcm::v3::Location::parent());
	let source_id: parachain::AssetId = source_location.clone().into();
	let asset_metadata = parachain::AssetMetadata {
		name: b"RelayToken".to_vec(),
		symbol: b"Relay".to_vec(),
		decimals: 12,
	};
	// register relay asset in parachain A
	ParaA::execute_with(|| {
		assert_ok!(AssetManager::register_foreign_asset(
			parachain::RuntimeOrigin::root(),
			source_location.clone(),
			asset_metadata,
			1u128,
			false
		));
		assert_ok!(add_supported_asset(source_location, 0));
	});

	// Actually send relay asset to parachain
	let dest: Location = AccountKey20 {
		network: None,
		key: fresh_account,
	}
	.into();
	Relay::execute_with(|| {
		assert_ok!(RelayChainPalletXcm::limited_reserve_transfer_assets(
			relay_chain::RuntimeOrigin::signed(RELAYALICE),
			Box::new(Parachain(1).into()),
			Box::new(VersionedLocation::from(dest.clone()).clone().into()),
			Box::new(([], 123).into()),
			0,
			WeightLimit::Unlimited
		));
	});

	// parachain should not have received assets
	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_eq!(Assets::balance(source_id, &fresh_account.into()), 0);
	});

	// Send native token to fresh_account
	ParaA::execute_with(|| {
		assert_ok!(ParaBalances::transfer_allow_death(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			fresh_account.into(),
			100
		));
	});

	// Re-send tokens
	Relay::execute_with(|| {
		assert_ok!(RelayChainPalletXcm::limited_reserve_transfer_assets(
			relay_chain::RuntimeOrigin::signed(RELAYALICE),
			Box::new(Parachain(1).into()),
			Box::new(VersionedLocation::from(dest).clone()),
			Box::new(([], 123).into()),
			0,
			WeightLimit::Unlimited
		));
	});

	// parachain should have received assets
	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_eq!(Assets::balance(source_id, &fresh_account.into()), 123);
	});
}

#[test]
fn receive_assets_with_sufficients_true_allows_non_funded_account_to_receive_assets() {
	MockNet::reset();

	let fresh_account = [2u8; 20];
	let source_location = parachain::AssetType::Xcm(xcm::v3::Location::parent());
	let source_id: parachain::AssetId = source_location.clone().into();
	let asset_metadata = parachain::AssetMetadata {
		name: b"RelayToken".to_vec(),
		symbol: b"Relay".to_vec(),
		decimals: 12,
	};
	// register relay asset in parachain A
	ParaA::execute_with(|| {
		assert_ok!(AssetManager::register_foreign_asset(
			parachain::RuntimeOrigin::root(),
			source_location.clone(),
			asset_metadata,
			1u128,
			true
		));
		assert_ok!(add_supported_asset(source_location, 0));
	});

	// Actually send relay asset to parachain
	let dest: Location = AccountKey20 {
		network: None,
		key: fresh_account,
	}
	.into();
	Relay::execute_with(|| {
		assert_ok!(RelayChainPalletXcm::limited_reserve_transfer_assets(
			relay_chain::RuntimeOrigin::signed(RELAYALICE),
			Box::new(Parachain(1).into()),
			Box::new(VersionedLocation::from(dest.clone()).clone().into()),
			Box::new(([], 123).into()),
			0,
			WeightLimit::Unlimited
		));
	});

	// parachain should have received assets
	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_eq!(Assets::balance(source_id, &fresh_account.into()), 123);
	});
}
