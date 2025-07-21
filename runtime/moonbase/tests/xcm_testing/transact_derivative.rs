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

use crate::xcm_mock::parachain::PolkadotXcm;
use crate::xcm_mock::*;
use crate::xcm_testing::{add_supported_asset, currency_to_asset, derivative_account_id};
use frame_support::{
	assert_ok, traits::PalletInfo, weights::constants::WEIGHT_REF_TIME_PER_SECOND, weights::Weight,
};
use pallet_xcm_transactor::{Currency, CurrencyPayment, TransactWeights};
use sp_std::boxed::Box;
use xcm::VersionedLocation;
use xcm::{
	latest::prelude::{AccountId32, AccountKey20, Limited, Location, Parachain, WeightLimit},
	VersionedAssets,
};
use xcm_primitives::{split_location_into_chain_part_and_beneficiary, DEFAULT_PROOF_SIZE};
use xcm_simulator::{Encode, TestExt};

#[test]
fn transact_through_derivative_multilocation() {
	MockNet::reset();

	let source_location = parachain::AssetType::Xcm(xcm::v3::Location::parent());
	let source_id: parachain::AssetId = source_location.clone().into();

	let asset_metadata = parachain::AssetMetadata {
		name: b"RelayToken".to_vec(),
		symbol: b"Relay".to_vec(),
		decimals: 12,
	};

	ParaA::execute_with(|| {
		assert_ok!(AssetManager::register_foreign_asset(
			parachain::RuntimeOrigin::root(),
			source_location.clone(),
			asset_metadata,
			1u128,
			true
		));
		assert_ok!(add_supported_asset(source_location, 1));

		// Root can set transact info
		assert_ok!(XcmTransactor::set_transact_info(
			parachain::RuntimeOrigin::root(),
			Box::new(xcm::VersionedLocation::from(Location::parent())),
			// Relay charges 1000 for every instruction, and we have 3, so 3000
			3000.into(),
			20000000000.into(),
			None
		));
		// Root can set transact info
		assert_ok!(XcmTransactor::set_fee_per_second(
			parachain::RuntimeOrigin::root(),
			Box::new(xcm::VersionedLocation::from(Location::parent())),
			WEIGHT_REF_TIME_PER_SECOND as u128,
		));
	});

	// Let's construct the call to know how much weight it is going to require

	let dest: Location = AccountKey20 {
		network: None,
		key: PARAALICE,
	}
	.into();
	Relay::execute_with(|| {
		// 4000000000 transact + 3000 correspond to 4000003000 tokens. 100 more for the transfer call
		assert_ok!(RelayChainPalletXcm::limited_reserve_transfer_assets(
			relay_chain::RuntimeOrigin::signed(RELAYALICE),
			Box::new(Parachain(1).into()),
			Box::new(VersionedLocation::from(dest).clone()),
			Box::new(([], 4000003100u128).into()),
			0,
			WeightLimit::Unlimited
		));
	});

	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 4000003100);
	});

	// Register address
	ParaA::execute_with(|| {
		assert_ok!(XcmTransactor::register(
			parachain::RuntimeOrigin::root(),
			PARAALICE.into(),
			0,
		));
	});

	// Send to registered address

	let registered_address = derivative_account_id(para_a_account(), 0);
	let dest = Location {
		parents: 1,
		interior: [AccountId32 {
			network: None,
			id: registered_address.clone().into(),
		}]
		.into(),
	};
	let (chain_part, beneficiary) = split_location_into_chain_part_and_beneficiary(dest).unwrap();

	ParaA::execute_with(|| {
		let asset = currency_to_asset(parachain::CurrencyId::ForeignAsset(source_id), 100);
		// free execution, full amount received
		assert_ok!(PolkadotXcm::transfer_assets(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			Box::new(VersionedLocation::from(chain_part)),
			Box::new(VersionedLocation::from(beneficiary)),
			Box::new(VersionedAssets::from(vec![asset])),
			0,
			WeightLimit::Limited(Weight::from_parts(40000u64, DEFAULT_PROOF_SIZE))
		));
	});

	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 4000003000);
	});

	// What we will do now is transfer this relay tokens from the derived account to the sovereign
	// again
	Relay::execute_with(|| {
		// free execution,x	 full amount received
		assert!(RelayBalances::free_balance(&para_a_account()) == 4000003000);
	});

	// Encode the call. Balances transact to para_a_account
	// First index
	let mut encoded: Vec<u8> = Vec::new();
	let index = <relay_chain::Runtime as frame_system::Config>::PalletInfo::index::<
		relay_chain::Balances,
	>()
	.unwrap() as u8;

	encoded.push(index);

	// Then call bytes
	let mut call_bytes = pallet_balances::Call::<relay_chain::Runtime>::transfer_allow_death {
		dest: para_a_account(),
		value: 100u32.into(),
	}
	.encode();
	encoded.append(&mut call_bytes);

	ParaA::execute_with(|| {
		assert_ok!(XcmTransactor::transact_through_derivative(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			parachain::MockTransactors::Relay,
			0,
			CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::from(
					Location::parent()
				))),
				fee_amount: None
			},
			// 4000000000 + 3000 we should have taken out 4000003000 tokens from the caller
			encoded,
			TransactWeights {
				transact_required_weight_at_most: 4000000000.into(),
				overall_weight: None
			},
			false
		));
		let event_found: Option<parachain::RuntimeEvent> = parachain::para_events()
			.iter()
			.find_map(|event| match event.clone() {
				parachain::RuntimeEvent::PolkadotXcm(pallet_xcm::Event::AssetsTrapped {
					..
				}) => Some(event.clone()),
				_ => None,
			});
		// Assert that the events do not contain the assets being trapped
		assert!(event_found.is_none());
	});

	Relay::execute_with(|| {
		// free execution,x	 full amount received
		assert!(RelayBalances::free_balance(&para_a_account()) == 100);

		assert!(RelayBalances::free_balance(&registered_address) == 0);
	});
}

#[test]
fn transact_through_derivative_with_custom_fee_weight() {
	MockNet::reset();

	let source_location = parachain::AssetType::Xcm(xcm::v3::Location::parent());
	let source_id: parachain::AssetId = source_location.clone().into();

	let asset_metadata = parachain::AssetMetadata {
		name: b"RelayToken".to_vec(),
		symbol: b"Relay".to_vec(),
		decimals: 12,
	};

	ParaA::execute_with(|| {
		assert_ok!(AssetManager::register_foreign_asset(
			parachain::RuntimeOrigin::root(),
			source_location.clone(),
			asset_metadata,
			1u128,
			true
		));
		assert_ok!(add_supported_asset(source_location, 1));
	});

	// Let's construct the call to know how much weight it is going to require

	let dest: Location = AccountKey20 {
		network: None,
		key: PARAALICE,
	}
	.into();
	Relay::execute_with(|| {
		// 4000000000 transact + 3000 correspond to 4000003000 tokens. 100 more for the transfer call
		assert_ok!(RelayChainPalletXcm::limited_reserve_transfer_assets(
			relay_chain::RuntimeOrigin::signed(RELAYALICE),
			Box::new(Parachain(1).into()),
			Box::new(VersionedLocation::from(dest).clone()),
			Box::new(([], 4000003100u128).into()),
			0,
			WeightLimit::Unlimited
		));
	});

	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 4000003100);
	});

	// Register address
	ParaA::execute_with(|| {
		assert_ok!(XcmTransactor::register(
			parachain::RuntimeOrigin::root(),
			PARAALICE.into(),
			0,
		));
	});

	// Send to registered address

	let registered_address = derivative_account_id(para_a_account(), 0);
	let dest = Location {
		parents: 1,
		interior: [AccountId32 {
			network: None,
			id: registered_address.clone().into(),
		}]
		.into(),
	};
	let (chain_part, beneficiary) = split_location_into_chain_part_and_beneficiary(dest).unwrap();

	ParaA::execute_with(|| {
		let asset = currency_to_asset(parachain::CurrencyId::ForeignAsset(source_id), 100);
		// free execution, full amount received
		assert_ok!(PolkadotXcm::transfer_assets(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			Box::new(VersionedLocation::from(chain_part)),
			Box::new(VersionedLocation::from(beneficiary)),
			Box::new(VersionedAssets::from(vec![asset])),
			0,
			WeightLimit::Limited(Weight::from_parts(40000u64, DEFAULT_PROOF_SIZE))
		));
	});

	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 4000003000);
	});

	// What we will do now is transfer this relay tokens from the derived account to the sovereign
	// again
	Relay::execute_with(|| {
		// free execution,x	 full amount received
		assert!(RelayBalances::free_balance(&para_a_account()) == 4000003000);
	});

	// Encode the call. Balances transact to para_a_account
	// First index
	let mut encoded: Vec<u8> = Vec::new();
	let index = <relay_chain::Runtime as frame_system::Config>::PalletInfo::index::<
		relay_chain::Balances,
	>()
	.unwrap() as u8;

	encoded.push(index);

	// Then call bytes
	let mut call_bytes = pallet_balances::Call::<relay_chain::Runtime>::transfer_allow_death {
		dest: para_a_account(),
		value: 100u32.into(),
	}
	.encode();
	encoded.append(&mut call_bytes);

	let overall_weight = 4000003000u64;
	ParaA::execute_with(|| {
		assert_ok!(XcmTransactor::transact_through_derivative(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			parachain::MockTransactors::Relay,
			0,
			CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::from(
					Location::parent()
				))),
				// 1-1 fee weight mapping
				fee_amount: Some(overall_weight as u128)
			},
			// 4000000000 + 3000 we should have taken out 4000003000 tokens from the caller
			encoded,
			TransactWeights {
				transact_required_weight_at_most: 4000000000.into(),
				overall_weight: Some(Limited(overall_weight.into()))
			},
			false
		));
		let event_found: Option<parachain::RuntimeEvent> = parachain::para_events()
			.iter()
			.find_map(|event| match event.clone() {
				parachain::RuntimeEvent::PolkadotXcm(pallet_xcm::Event::AssetsTrapped {
					..
				}) => Some(event.clone()),
				_ => None,
			});
		// Assert that the events do not contain the assets being trapped
		assert!(event_found.is_none());
	});

	Relay::execute_with(|| {
		// free execution,x	 full amount received
		assert!(RelayBalances::free_balance(&para_a_account()) == 100);

		assert!(RelayBalances::free_balance(&registered_address) == 0);
	});
}

#[test]
fn transact_through_derivative_with_custom_fee_weight_refund() {
	MockNet::reset();

	let source_location = parachain::AssetType::Xcm(xcm::v3::Location::parent());
	let source_id: parachain::AssetId = source_location.clone().into();

	let asset_metadata = parachain::AssetMetadata {
		name: b"RelayToken".to_vec(),
		symbol: b"Relay".to_vec(),
		decimals: 12,
	};

	ParaA::execute_with(|| {
		assert_ok!(AssetManager::register_foreign_asset(
			parachain::RuntimeOrigin::root(),
			source_location.clone(),
			asset_metadata,
			1u128,
			true
		));
		assert_ok!(add_supported_asset(source_location, 1));
	});

	// Let's construct the call to know how much weight it is going to require

	let dest: Location = AccountKey20 {
		network: None,
		key: PARAALICE,
	}
	.into();
	Relay::execute_with(|| {
		// 4000000000 transact + 9000 correspond to 4000009000 tokens. 100 more for the transfer call
		assert_ok!(RelayChainPalletXcm::limited_reserve_transfer_assets(
			relay_chain::RuntimeOrigin::signed(RELAYALICE),
			Box::new(Parachain(1).into()),
			Box::new(VersionedLocation::from(dest).clone()),
			Box::new(([], 4000009100u128).into()),
			0,
			WeightLimit::Unlimited
		));
	});

	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 4000009100);
	});

	// Register address
	ParaA::execute_with(|| {
		assert_ok!(XcmTransactor::register(
			parachain::RuntimeOrigin::root(),
			PARAALICE.into(),
			0,
		));
	});

	// Send to registered address

	let registered_address = derivative_account_id(para_a_account(), 0);
	let dest = Location {
		parents: 1,
		interior: [AccountId32 {
			network: None,
			id: registered_address.clone().into(),
		}]
		.into(),
	};
	let (chain_part, beneficiary) = split_location_into_chain_part_and_beneficiary(dest).unwrap();

	ParaA::execute_with(|| {
		let asset = currency_to_asset(parachain::CurrencyId::ForeignAsset(source_id), 100);
		// free execution, full amount received
		assert_ok!(PolkadotXcm::transfer_assets(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			Box::new(VersionedLocation::from(chain_part)),
			Box::new(VersionedLocation::from(beneficiary)),
			Box::new(VersionedAssets::from(vec![asset])),
			0,
			WeightLimit::Limited(Weight::from_parts(40000u64, DEFAULT_PROOF_SIZE))
		));
	});

	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 4000009000);
	});

	// What we will do now is transfer this relay tokens from the derived account to the sovereign
	// again
	Relay::execute_with(|| {
		// free execution,x	 full amount received
		assert!(RelayBalances::free_balance(&para_a_account()) == 4000009000);
	});

	// Encode the call. Balances transact to para_a_account
	// First index
	let mut encoded: Vec<u8> = Vec::new();
	let index = <relay_chain::Runtime as frame_system::Config>::PalletInfo::index::<
		relay_chain::Balances,
	>()
	.unwrap() as u8;

	encoded.push(index);

	// Then call bytes
	let mut call_bytes = pallet_balances::Call::<relay_chain::Runtime>::transfer_allow_death {
		dest: para_a_account(),
		value: 100u32.into(),
	}
	.encode();
	encoded.append(&mut call_bytes);

	let overall_weight = 4000009000u64;
	ParaA::execute_with(|| {
		assert_ok!(XcmTransactor::transact_through_derivative(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			parachain::MockTransactors::Relay,
			0,
			CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::from(
					Location::parent()
				))),
				// 1-1 fee weight mapping
				fee_amount: Some(overall_weight as u128)
			},
			encoded,
			TransactWeights {
				transact_required_weight_at_most: 4000000000.into(),
				overall_weight: Some(Limited(overall_weight.into()))
			},
			true
		));
		let event_found: Option<parachain::RuntimeEvent> = parachain::para_events()
			.iter()
			.find_map(|event| match event.clone() {
				parachain::RuntimeEvent::PolkadotXcm(pallet_xcm::Event::AssetsTrapped {
					..
				}) => Some(event.clone()),
				_ => None,
			});
		// Assert that the events do not contain the assets being trapped
		assert!(event_found.is_none());
	});

	Relay::execute_with(|| {
		// free execution, full amount received
		// 4000009000 refunded + 100 transferred = 4000009100
		assert_eq!(RelayBalances::free_balance(&para_a_account()), 4000009100);
		assert_eq!(RelayBalances::free_balance(&registered_address), 0);
	});
}
