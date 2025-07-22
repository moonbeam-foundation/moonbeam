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
use crate::xcm_testing::{
	add_supported_asset, currency_to_asset, derivative_account_id, helpers::*,
};
use frame_support::{
	assert_ok, traits::PalletInfo, weights::constants::WEIGHT_REF_TIME_PER_SECOND, weights::Weight,
};
use pallet_xcm_transactor::{Currency, CurrencyPayment, TransactWeights};
use sp_std::boxed::Box;
use xcm::VersionedLocation;
use xcm::{
	latest::prelude::{AccountId32, Limited, Location, OriginKind, Parachain, WeightLimit},
	VersionedAssets,
};
use xcm_primitives::{
	split_location_into_chain_part_and_beneficiary, UtilityEncodeCall, DEFAULT_PROOF_SIZE,
};
use xcm_simulator::{Encode, TestExt};

#[test]
fn transact_through_sovereign() {
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

	let dest = account_key20_location(PARAALICE);
	Relay::execute_with(|| {
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
		0
	});

	// We send the xcm transact operation to parent
	let dest = Location {
		parents: 1,
		interior: [].into(),
	};

	// Encode the call. Balances transact to para_a_account
	// First index
	let encoded = encode_relay_balance_transfer_call(para_a_account(), 100u128);

	let utility_bytes = parachain::MockTransactors::Relay.encode_call(
		xcm_primitives::UtilityAvailableCalls::AsDerivative(0, encoded),
	);

	// Root can directly pass the execution byes to the sovereign
	ParaA::execute_with(|| {
		assert_ok!(XcmTransactor::transact_through_sovereign(
			parachain::RuntimeOrigin::root(),
			Box::new(xcm::VersionedLocation::from(dest)),
			Some(PARAALICE.into()),
			CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::from(
					Location::parent()
				))),
				fee_amount: None
			},
			utility_bytes,
			OriginKind::SovereignAccount,
			TransactWeights {
				transact_required_weight_at_most: 4000000000.into(),
				overall_weight: None
			},
			false
		));
	});

	Relay::execute_with(|| {
		// free execution,x	 full amount received
		assert!(RelayBalances::free_balance(&para_a_account()) == 100);

		assert!(RelayBalances::free_balance(&registered_address) == 0);
	});
}

#[test]
fn transact_through_sovereign_fee_payer_none() {
	MockNet::reset();

	ParaA::execute_with(|| {
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

	let derivative_address = derivative_account_id(para_a_account(), 0);

	Relay::execute_with(|| {
		// Transfer 100 tokens to derivative_address on the relay
		assert_ok!(RelayBalances::transfer_keep_alive(
			relay_chain::RuntimeOrigin::signed(RELAYALICE),
			derivative_address.clone(),
			100u128
		));

		// Transfer the XCM execution fee amount to ParaA's sovereign account
		assert_ok!(RelayBalances::transfer_keep_alive(
			relay_chain::RuntimeOrigin::signed(RELAYALICE),
			para_a_account(),
			4000003000u128
		));
	});

	// Check balances before the transact call
	Relay::execute_with(|| {
		assert_eq!(RelayBalances::free_balance(&para_a_account()), 4000003000);
		assert_eq!(RelayBalances::free_balance(&derivative_address), 100);
		assert_eq!(RelayBalances::free_balance(&RELAYBOB), 0);
	});

	// Encode the call. Balances transfer of 100 relay tokens to RELAYBOB
	let mut encoded: Vec<u8> = Vec::new();
	let index = <relay_chain::Runtime as frame_system::Config>::PalletInfo::index::<
		relay_chain::Balances,
	>()
	.unwrap() as u8;

	encoded.push(index);

	let mut call_bytes = pallet_balances::Call::<relay_chain::Runtime>::transfer_allow_death {
		dest: RELAYBOB,
		value: 100u32.into(),
	}
	.encode();
	encoded.append(&mut call_bytes);

	// The final call will be an AsDerivative using index 0
	let utility_bytes = parachain::MockTransactors::Relay.encode_call(
		xcm_primitives::UtilityAvailableCalls::AsDerivative(0, encoded),
	);

	// We send the xcm transact operation to parent
	let dest = Location {
		parents: 1,
		interior: /* Here */ [].into(),
	};

	// Root can directly pass the execution byes to the sovereign
	ParaA::execute_with(|| {
		assert_ok!(XcmTransactor::transact_through_sovereign(
			parachain::RuntimeOrigin::root(),
			Box::new(xcm::VersionedLocation::from(dest)),
			// No fee_payer here. The sovereign account will pay the fees on destination.
			None,
			CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::from(
					Location::parent()
				))),
				fee_amount: None
			},
			utility_bytes,
			OriginKind::SovereignAccount,
			TransactWeights {
				transact_required_weight_at_most: 4000000000.into(),
				overall_weight: None
			},
			false
		));
	});

	// Check balances after the transact call are correct
	Relay::execute_with(|| {
		assert_eq!(RelayBalances::free_balance(&para_a_account()), 0);
		assert_eq!(RelayBalances::free_balance(&derivative_address), 0);
		assert_eq!(RelayBalances::free_balance(&RELAYBOB), 100);
	});
}

#[test]
fn transact_through_sovereign_with_custom_fee_weight() {
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

	let dest = account_key20_location(PARAALICE);
	Relay::execute_with(|| {
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
		0
	});

	// We send the xcm transact operation to parent
	let dest = Location {
		parents: 1,
		interior: [].into(),
	};

	// Encode the call. Balances transact to para_a_account
	// First index
	let encoded = encode_relay_balance_transfer_call(para_a_account(), 100u128);

	let utility_bytes = parachain::MockTransactors::Relay.encode_call(
		xcm_primitives::UtilityAvailableCalls::AsDerivative(0, encoded),
	);

	let total_weight = 4000003000u64;
	// Root can directly pass the execution byes to the sovereign
	ParaA::execute_with(|| {
		assert_ok!(XcmTransactor::transact_through_sovereign(
			parachain::RuntimeOrigin::root(),
			Box::new(xcm::VersionedLocation::from(dest)),
			Some(PARAALICE.into()),
			CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::from(
					Location::parent()
				))),
				// 1-1 fee-weight mapping
				fee_amount: Some(total_weight as u128)
			},
			utility_bytes,
			OriginKind::SovereignAccount,
			TransactWeights {
				transact_required_weight_at_most: 4000000000.into(),
				overall_weight: Some(Limited(total_weight.into()))
			},
			false
		));
	});

	Relay::execute_with(|| {
		// free execution,x	 full amount received
		assert!(RelayBalances::free_balance(&para_a_account()) == 100);

		assert!(RelayBalances::free_balance(&registered_address) == 0);
	});
}

#[test]
fn transact_through_sovereign_with_custom_fee_weight_refund() {
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

	let dest = account_key20_location(PARAALICE);
	Relay::execute_with(|| {
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
		0
	});

	// We send the xcm transact operation to parent
	let dest = Location {
		parents: 1,
		interior: [].into(),
	};

	// Encode the call. Balances transact to para_a_account
	// First index
	let encoded = encode_relay_balance_transfer_call(para_a_account(), 100u128);

	let utility_bytes = parachain::MockTransactors::Relay.encode_call(
		xcm_primitives::UtilityAvailableCalls::AsDerivative(0, encoded),
	);

	let total_weight = 4000009000u64;
	// Root can directly pass the execution byes to the sovereign
	ParaA::execute_with(|| {
		assert_ok!(XcmTransactor::transact_through_sovereign(
			parachain::RuntimeOrigin::root(),
			Box::new(xcm::VersionedLocation::from(dest)),
			Some(PARAALICE.into()),
			CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::from(
					Location::parent()
				))),
				// 1-1 fee-weight mapping
				fee_amount: Some(total_weight as u128)
			},
			utility_bytes,
			OriginKind::SovereignAccount,
			TransactWeights {
				transact_required_weight_at_most: 4000000000.into(),
				overall_weight: Some(Limited(total_weight.into()))
			},
			true
		));
	});

	Relay::execute_with(|| {
		// free execution, full amount received
		// 4000009000 refunded + 100 transferred = 4000009100
		assert_eq!(RelayBalances::free_balance(&para_a_account()), 4000009100);

		assert_eq!(RelayBalances::free_balance(&registered_address), 0);
	});
}
