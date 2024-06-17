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

//! Moonbeam Runtime Xcm Tests

mod xcm_mock;

use cumulus_primitives_core::relay_chain::HrmpChannelId;
use frame_support::{
	assert_ok,
	traits::{PalletInfo, PalletInfoAccess},
	weights::{constants::WEIGHT_REF_TIME_PER_SECOND, Weight},
	BoundedVec,
};
use pallet_xcm_transactor::{
	Currency, CurrencyPayment, HrmpInitParams, HrmpOperation, TransactWeights,
};
use sp_core::ConstU32;
use sp_runtime::traits::MaybeEquivalence;
use xcm::latest::prelude::{
	AccountId32, AccountKey20, GeneralIndex, Junction, Junctions, Limited, Location, OriginKind,
	PalletInstance, Parachain, QueryResponse, Reanchorable, Response, WeightLimit, Xcm,
};
use xcm::{VersionedLocation, WrapVersion};
use xcm_executor::traits::ConvertLocation;
use xcm_mock::parachain;
use xcm_mock::relay_chain;
use xcm_mock::*;
use xcm_primitives::{UtilityEncodeCall, DEFAULT_PROOF_SIZE};
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

	// Register relay asset in paraA
	ParaA::execute_with(|| {
		assert_ok!(AssetManager::register_foreign_asset(
			parachain::RuntimeOrigin::root(),
			source_location.clone(),
			asset_metadata,
			1u128,
			true
		));
		assert_ok!(AssetManager::set_asset_units_per_second(
			parachain::RuntimeOrigin::root(),
			source_location,
			0u128,
			0
		));
	});

	// Actually send relay asset to parachain
	let dest: Location = AccountKey20 {
		network: None,
		key: PARAALICE,
	}
	.into();

	// First send relay chain asset to Parachain
	Relay::execute_with(|| {
		assert_ok!(RelayChainPalletXcm::reserve_transfer_assets(
			relay_chain::RuntimeOrigin::signed(RELAYALICE),
			Box::new(Parachain(1).into()),
			Box::new(VersionedLocation::V4(dest).clone().into()),
			Box::new(([] /* Here */, 123).into()),
			0,
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
		assert_ok!(AssetManager::set_asset_units_per_second(
			parachain::RuntimeOrigin::root(),
			source_location,
			0u128,
			0
		));
	});

	let dest: Location = Junction::AccountKey20 {
		network: None,
		key: PARAALICE,
	}
	.into();

	// First send relay chain asset to Parachain like in previous test
	Relay::execute_with(|| {
		assert_ok!(RelayChainPalletXcm::reserve_transfer_assets(
			relay_chain::RuntimeOrigin::signed(RELAYALICE),
			Box::new(Parachain(1).into()),
			Box::new(VersionedLocation::V4(dest).clone().into()),
			Box::new(([] /* Here */, 123).into()),
			0,
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

	ParaA::execute_with(|| {
		assert_ok!(XTokens::transfer(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			parachain::CurrencyId::ForeignAsset(source_id),
			123,
			Box::new(VersionedLocation::V4(dest)),
			WeightLimit::Limited(Weight::from_parts(40000u64, DEFAULT_PROOF_SIZE))
		));
	});

	// The balances in paraAlice should have been substracted
	ParaA::execute_with(|| {
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 0);
	});

	// Balances in the relay should have been received
	Relay::execute_with(|| {
		// free execution,x	 full amount received
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
		assert_ok!(AssetManager::set_asset_units_per_second(
			parachain::RuntimeOrigin::root(),
			source_location.clone(),
			0u128,
			0
		));
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
		assert_ok!(AssetManager::set_asset_units_per_second(
			parachain::RuntimeOrigin::root(),
			source_location,
			0u128,
			0
		));
	});

	let dest: Location = Junction::AccountKey20 {
		network: None,
		key: PARAALICE,
	}
	.into();
	Relay::execute_with(|| {
		assert_ok!(RelayChainPalletXcm::reserve_transfer_assets(
			relay_chain::RuntimeOrigin::signed(RELAYALICE),
			Box::new(Parachain(1).into()),
			Box::new(VersionedLocation::V4(dest).clone().into()),
			Box::new(([] /* Here */, 123).into()),
			0,
		));
	});

	ParaA::execute_with(|| {
		// free execution, full amount received
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

	ParaA::execute_with(|| {
		assert_ok!(XTokens::transfer(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			parachain::CurrencyId::ForeignAsset(source_id),
			100,
			Box::new(VersionedLocation::V4(dest)),
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
fn send_para_a_asset_to_para_b() {
	MockNet::reset();

	// This represents the asset in paraA
	let para_a_balances = Location::new(1, [Parachain(1), PalletInstance(1u8)]);
	let source_location = parachain::AssetType::Xcm(
		xcm_builder::WithLatestLocationConverter::convert(&para_a_balances).expect("convert to v3"),
	);
	let source_id: parachain::AssetId = source_location.clone().into();

	let asset_metadata = parachain::AssetMetadata {
		name: b"ParaAToken".to_vec(),
		symbol: b"ParaA".to_vec(),
		decimals: 18,
	};

	// Register asset in paraB. Free execution
	ParaB::execute_with(|| {
		assert_ok!(AssetManager::register_foreign_asset(
			parachain::RuntimeOrigin::root(),
			source_location.clone(),
			asset_metadata,
			1u128,
			true
		));
		assert_ok!(AssetManager::set_asset_units_per_second(
			parachain::RuntimeOrigin::root(),
			source_location,
			0u128,
			0
		));
	});

	// Send para A asset from para A to para B
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

	// Native token is substracted in paraA
	ParaA::execute_with(|| {
		// Free execution, full amount received
		assert_ok!(XTokens::transfer(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			parachain::CurrencyId::SelfReserve,
			100,
			Box::new(VersionedLocation::V4(dest)),
			WeightLimit::Limited(Weight::from_parts(800000u64, DEFAULT_PROOF_SIZE))
		));
	});

	ParaA::execute_with(|| {
		assert_eq!(
			ParaBalances::free_balance(&PARAALICE.into()),
			INITIAL_BALANCE - 100
		);
	});

	// Asset is minted in paraB
	ParaB::execute_with(|| {
		// Free execution, full amount received
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 100);
	});
}

#[test]
fn send_para_a_asset_from_para_b_to_para_c() {
	MockNet::reset();

	// Represents para A asset
	let para_a_balances = Location::new(1, [Parachain(1), PalletInstance(1u8)]);
	let source_location = parachain::AssetType::Xcm(
		xcm_builder::WithLatestLocationConverter::convert(&para_a_balances).expect("convert to v3"),
	);
	let source_id: parachain::AssetId = source_location.clone().into();

	let asset_metadata = parachain::AssetMetadata {
		name: b"ParaAToken".to_vec(),
		symbol: b"ParaA".to_vec(),
		decimals: 18,
	};

	// Register para A asset in parachain B. Free execution
	ParaB::execute_with(|| {
		assert_ok!(AssetManager::register_foreign_asset(
			parachain::RuntimeOrigin::root(),
			source_location.clone(),
			asset_metadata.clone(),
			1u128,
			true
		));
		assert_ok!(AssetManager::set_asset_units_per_second(
			parachain::RuntimeOrigin::root(),
			source_location.clone(),
			0u128,
			0
		));
	});

	// Register para A asset in parachain C. Free execution
	ParaC::execute_with(|| {
		assert_ok!(AssetManager::register_foreign_asset(
			parachain::RuntimeOrigin::root(),
			source_location.clone(),
			asset_metadata,
			1u128,
			true
		));
		assert_ok!(AssetManager::set_asset_units_per_second(
			parachain::RuntimeOrigin::root(),
			source_location,
			0u128,
			0
		));
	});

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
	ParaA::execute_with(|| {
		assert_ok!(XTokens::transfer(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			parachain::CurrencyId::SelfReserve,
			100,
			Box::new(VersionedLocation::V4(dest)),
			WeightLimit::Limited(Weight::from_parts(80u64, DEFAULT_PROOF_SIZE))
		));
	});

	// Para A balances have been substracted
	ParaA::execute_with(|| {
		assert_eq!(
			ParaBalances::free_balance(&PARAALICE.into()),
			INITIAL_BALANCE - 100
		);
	});

	// Para B balances have been credited
	ParaB::execute_with(|| {
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 100);
	});

	// Send para A asset from para B to para C
	let dest = Location {
		parents: 1,
		interior: [
			Parachain(3),
			AccountKey20 {
				network: None,
				key: PARAALICE.into(),
			},
		]
		.into(),
	};

	ParaB::execute_with(|| {
		assert_ok!(XTokens::transfer(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			parachain::CurrencyId::ForeignAsset(source_id),
			100,
			Box::new(VersionedLocation::V4(dest)),
			WeightLimit::Limited(Weight::from_parts(80u64, DEFAULT_PROOF_SIZE))
		));
	});

	// The message passed through parachainA so we needed to pay since its the native token
	ParaC::execute_with(|| {
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 96);
	});
}

#[test]
fn send_para_a_asset_to_para_b_and_back_to_para_a() {
	MockNet::reset();

	// para A asset
	let para_a_balances = Location::new(1, [Parachain(1), PalletInstance(1u8)]);
	let source_location = parachain::AssetType::Xcm(
		xcm_builder::WithLatestLocationConverter::convert(&para_a_balances).expect("convert to v3"),
	);
	let source_id: parachain::AssetId = source_location.clone().into();

	let asset_metadata = parachain::AssetMetadata {
		name: b"ParaAToken".to_vec(),
		symbol: b"ParaA".to_vec(),
		decimals: 18,
	};

	// Register para A asset in para B
	ParaB::execute_with(|| {
		assert_ok!(AssetManager::register_foreign_asset(
			parachain::RuntimeOrigin::root(),
			source_location.clone(),
			asset_metadata,
			1u128,
			true
		));
		assert_ok!(AssetManager::set_asset_units_per_second(
			parachain::RuntimeOrigin::root(),
			source_location,
			0u128,
			0
		));
	});

	// Send para A asset to para B
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

	ParaA::execute_with(|| {
		assert_ok!(XTokens::transfer(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			parachain::CurrencyId::SelfReserve,
			100,
			Box::new(VersionedLocation::V4(dest)),
			WeightLimit::Limited(Weight::from_parts(80u64, DEFAULT_PROOF_SIZE))
		));
	});

	// Balances have been substracted
	ParaA::execute_with(|| {
		assert_eq!(
			ParaBalances::free_balance(&PARAALICE.into()),
			INITIAL_BALANCE - 100
		);
	});

	// Para B balances have been credited
	ParaB::execute_with(|| {
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 100);
	});

	// Send back para A asset to para A
	let dest = Location {
		parents: 1,
		interior: [
			Parachain(1),
			AccountKey20 {
				network: None,
				key: PARAALICE.into(),
			},
		]
		.into(),
	};
	ParaB::execute_with(|| {
		assert_ok!(XTokens::transfer(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			parachain::CurrencyId::ForeignAsset(source_id),
			100,
			Box::new(VersionedLocation::V4(dest)),
			WeightLimit::Limited(Weight::from_parts(80u64, DEFAULT_PROOF_SIZE))
		));
	});

	ParaA::execute_with(|| {
		// Weight used is 4
		assert_eq!(
			ParaBalances::free_balance(&PARAALICE.into()),
			INITIAL_BALANCE - 4
		);
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
		assert_ok!(AssetManager::set_asset_units_per_second(
			parachain::RuntimeOrigin::root(),
			source_location,
			2500000000000u128,
			0
		));
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
		assert_ok!(RelayChainPalletXcm::reserve_transfer_assets(
			relay_chain::RuntimeOrigin::signed(RELAYALICE),
			Box::new(Parachain(1).into()),
			Box::new(VersionedLocation::V4(dest).clone().into()),
			Box::new(([] /* Here */, 100).into()),
			0,
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
fn send_para_a_asset_to_para_b_with_trader() {
	MockNet::reset();

	let para_a_balances = Location::new(1, [Parachain(1), PalletInstance(1u8)]);
	let source_location = parachain::AssetType::Xcm(
		xcm_builder::WithLatestLocationConverter::convert(&para_a_balances).expect("convert to v3"),
	);
	let source_id: parachain::AssetId = source_location.clone().into();

	let asset_metadata = parachain::AssetMetadata {
		name: b"ParaAToken".to_vec(),
		symbol: b"ParaA".to_vec(),
		decimals: 18,
	};

	ParaB::execute_with(|| {
		assert_ok!(AssetManager::register_foreign_asset(
			parachain::RuntimeOrigin::root(),
			source_location.clone(),
			asset_metadata,
			1u128,
			true
		));
		assert_ok!(AssetManager::set_asset_units_per_second(
			parachain::RuntimeOrigin::root(),
			source_location,
			2500000000000u128,
			0
		));
	});

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

	// In destination chain, we only need 4 weight
	// We put 10 weight, 6 of which should be refunded and 4 of which should go to treasury
	ParaA::execute_with(|| {
		assert_ok!(XTokens::transfer(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			parachain::CurrencyId::SelfReserve,
			100,
			Box::new(VersionedLocation::V4(dest)),
			WeightLimit::Limited(Weight::from_parts(10u64, DEFAULT_PROOF_SIZE))
		));
	});
	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_eq!(
			ParaBalances::free_balance(&PARAALICE.into()),
			INITIAL_BALANCE - 100
		);
	});

	// We are sending 100 tokens from para A.
	// Amount spent in fees is Units per second * weight / 1_000_000_000_000 (weight per second)
	// weight is 4 since we are executing 4 instructions with a unitweightcost of 1.
	// Units per second should be 2_500_000_000_000_000
	// Since we set 10 weight in destination chain, 25 will be charged upfront
	// 15 of those will be refunded, while 10 will go to treasury as the true weight used
	// will be 4
	ParaB::execute_with(|| {
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 90);
		// Fee should have been received by treasury
		assert_eq!(Assets::balance(source_id, &Treasury::account_id()), 10);
	});
}

#[test]
fn send_para_a_asset_to_para_b_with_trader_and_fee() {
	MockNet::reset();

	let para_a_balances = Location::new(1, [Parachain(1), PalletInstance(1u8)]);
	let source_location = parachain::AssetType::Xcm(
		xcm_builder::WithLatestLocationConverter::convert(&para_a_balances).expect("convert to v3"),
	);
	let source_id: parachain::AssetId = source_location.clone().into();

	let asset_metadata = parachain::AssetMetadata {
		name: b"ParaAToken".to_vec(),
		symbol: b"ParaA".to_vec(),
		decimals: 18,
	};

	ParaB::execute_with(|| {
		assert_ok!(AssetManager::register_foreign_asset(
			parachain::RuntimeOrigin::root(),
			source_location.clone(),
			asset_metadata,
			1u128,
			true
		));
		// With these units per second, 80K weight convrets to 1 asset unit
		assert_ok!(AssetManager::set_asset_units_per_second(
			parachain::RuntimeOrigin::root(),
			source_location,
			12500000u128,
			0
		));
	});

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

	// we use transfer_with_fee
	ParaA::execute_with(|| {
		assert_ok!(XTokens::transfer_with_fee(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			parachain::CurrencyId::SelfReserve,
			100,
			1,
			Box::new(VersionedLocation::V4(dest)),
			WeightLimit::Limited(Weight::from_parts(800000u64, DEFAULT_PROOF_SIZE))
		));
	});
	ParaA::execute_with(|| {
		// 100 tokens transferred plus 1 taken from fees
		assert_eq!(
			ParaBalances::free_balance(&PARAALICE.into()),
			INITIAL_BALANCE - 100 - 1
		);
	});

	ParaB::execute_with(|| {
		// free execution, full amount received because trully the xcm instruction does not cost
		// what it is specified
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 101);
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
		assert_ok!(AssetManager::set_asset_units_per_second(
			parachain::RuntimeOrigin::root(),
			source_location,
			2500000000000u128,
			0
		));
	});

	// We are sending 100 tokens from relay.
	// If we set the dest weight to be 1e7, we know the buy_execution will spend 1e7*1e6/1e12 = 10
	// Therefore with no refund, we should receive 10 tokens less
	Relay::execute_with(|| {
		assert_ok!(RelayChainPalletXcm::reserve_transfer_assets(
			relay_chain::RuntimeOrigin::signed(RELAYALICE),
			Box::new(Parachain(1).into()),
			Box::new(VersionedLocation::V4(dest).clone().into()),
			Box::new(([] /* Here */, 5).into()),
			0,
		));
	});

	ParaA::execute_with(|| {
		// amount not received as it is not paying enough
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 0);
	});
}

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
		assert_ok!(AssetManager::set_asset_units_per_second(
			parachain::RuntimeOrigin::root(),
			source_location,
			1u128,
			0
		));

		// Root can set transact info
		assert_ok!(XcmTransactor::set_transact_info(
			parachain::RuntimeOrigin::root(),
			Box::new(xcm::VersionedLocation::V4(Location::parent())),
			// Relay charges 1000 for every instruction, and we have 3, so 3000
			3000.into(),
			20000000000.into(),
			None
		));
		// Root can set transact info
		assert_ok!(XcmTransactor::set_fee_per_second(
			parachain::RuntimeOrigin::root(),
			Box::new(xcm::VersionedLocation::V4(Location::parent())),
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
		assert_ok!(RelayChainPalletXcm::reserve_transfer_assets(
			relay_chain::RuntimeOrigin::signed(RELAYALICE),
			Box::new(Parachain(1).into()),
			Box::new(VersionedLocation::V4(dest).clone().into()),
			Box::new(([] /* Here */, 4000003100u128).into()),
			0,
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

	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_ok!(XTokens::transfer(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			parachain::CurrencyId::ForeignAsset(source_id),
			100,
			Box::new(VersionedLocation::V4(dest)),
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
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::V4(
					Location::parent()
				))),
				fee_amount: None
			},
			encoded,
			// 400000000 + 3000 we should have taken out 4000003000 tokens from the caller
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
		assert_ok!(AssetManager::set_asset_units_per_second(
			parachain::RuntimeOrigin::root(),
			source_location,
			1u128,
			0
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
		assert_ok!(RelayChainPalletXcm::reserve_transfer_assets(
			relay_chain::RuntimeOrigin::signed(RELAYALICE),
			Box::new(Parachain(1).into()),
			Box::new(VersionedLocation::V4(dest).clone().into()),
			Box::new(([] /* Here */, 4000003100u128).into()),
			0,
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

	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_ok!(XTokens::transfer(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			parachain::CurrencyId::ForeignAsset(source_id),
			100,
			Box::new(VersionedLocation::V4(dest)),
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
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::V4(
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
		assert_ok!(AssetManager::set_asset_units_per_second(
			parachain::RuntimeOrigin::root(),
			source_location,
			1u128,
			0
		));
	});

	// Let's construct the call to know how much weight it is going to require

	let dest: Location = AccountKey20 {
		network: None,
		key: PARAALICE,
	}
	.into();
	Relay::execute_with(|| {
		// 4000000000 transact + 9000 correspond to 4000009000 tokens. 100 more for the transfer call
		assert_ok!(RelayChainPalletXcm::reserve_transfer_assets(
			relay_chain::RuntimeOrigin::signed(RELAYALICE),
			Box::new(Parachain(1).into()),
			Box::new(VersionedLocation::V4(dest).clone().into()),
			Box::new(([] /* Here */, 4000009100u128).into()),
			0,
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

	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_ok!(XTokens::transfer(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			parachain::CurrencyId::ForeignAsset(source_id),
			100,
			Box::new(VersionedLocation::V4(dest)),
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
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::V4(
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
		// free execution,x	 full amount received
		// 4000005186 refunded + 100 transferred = 4000005286
		assert_eq!(RelayBalances::free_balance(&para_a_account()), 4000005286);
		assert_eq!(RelayBalances::free_balance(&registered_address), 0);
	});
}

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
		assert_ok!(AssetManager::set_asset_units_per_second(
			parachain::RuntimeOrigin::root(),
			source_location,
			1u128,
			0
		));

		// Root can set transact info
		assert_ok!(XcmTransactor::set_transact_info(
			parachain::RuntimeOrigin::root(),
			Box::new(xcm::VersionedLocation::V4(Location::parent())),
			// Relay charges 1000 for every instruction, and we have 3, so 3000
			3000.into(),
			20000000000.into(),
			None
		));
		// Root can set transact info
		assert_ok!(XcmTransactor::set_fee_per_second(
			parachain::RuntimeOrigin::root(),
			Box::new(xcm::VersionedLocation::V4(Location::parent())),
			WEIGHT_REF_TIME_PER_SECOND as u128,
		));
	});

	let dest: Location = AccountKey20 {
		network: None,
		key: PARAALICE,
	}
	.into();
	Relay::execute_with(|| {
		assert_ok!(RelayChainPalletXcm::reserve_transfer_assets(
			relay_chain::RuntimeOrigin::signed(RELAYALICE),
			Box::new(Parachain(1).into()),
			Box::new(VersionedLocation::V4(dest).clone().into()),
			Box::new(([] /* Here */, 4000003100u128).into()),
			0,
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

	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_ok!(XTokens::transfer(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			parachain::CurrencyId::ForeignAsset(source_id),
			100,
			Box::new(VersionedLocation::V4(dest)),
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
		interior: /* Here */ [].into(),
	};

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

	let utility_bytes = parachain::MockTransactors::Relay.encode_call(
		xcm_primitives::UtilityAvailableCalls::AsDerivative(0, encoded),
	);

	// Root can directly pass the execution byes to the sovereign
	ParaA::execute_with(|| {
		assert_ok!(XcmTransactor::transact_through_sovereign(
			parachain::RuntimeOrigin::root(),
			Box::new(xcm::VersionedLocation::V4(dest)),
			Some(PARAALICE.into()),
			CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::V4(
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
			Box::new(xcm::VersionedLocation::V4(Location::parent())),
			// Relay charges 1000 for every instruction, and we have 3, so 3000
			3000.into(),
			20000000000.into(),
			None
		));
		// Root can set transact info
		assert_ok!(XcmTransactor::set_fee_per_second(
			parachain::RuntimeOrigin::root(),
			Box::new(xcm::VersionedLocation::V4(Location::parent())),
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
			Box::new(xcm::VersionedLocation::V4(dest)),
			// No fee_payer here. The sovereign account will pay the fees on destination.
			None,
			CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::V4(
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
		assert_ok!(AssetManager::set_asset_units_per_second(
			parachain::RuntimeOrigin::root(),
			source_location,
			1u128,
			0
		));
	});

	let dest: Location = AccountKey20 {
		network: None,
		key: PARAALICE,
	}
	.into();
	Relay::execute_with(|| {
		assert_ok!(RelayChainPalletXcm::reserve_transfer_assets(
			relay_chain::RuntimeOrigin::signed(RELAYALICE),
			Box::new(Parachain(1).into()),
			Box::new(VersionedLocation::V4(dest).clone().into()),
			Box::new(([] /* Here */, 4000003100u128).into()),
			0,
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

	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_ok!(XTokens::transfer(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			parachain::CurrencyId::ForeignAsset(source_id),
			100,
			Box::new(VersionedLocation::V4(dest)),
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
		interior: /* Here */ [].into(),
	};

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

	let utility_bytes = parachain::MockTransactors::Relay.encode_call(
		xcm_primitives::UtilityAvailableCalls::AsDerivative(0, encoded),
	);

	let total_weight = 4000003000u64;
	// Root can directly pass the execution byes to the sovereign
	ParaA::execute_with(|| {
		assert_ok!(XcmTransactor::transact_through_sovereign(
			parachain::RuntimeOrigin::root(),
			Box::new(xcm::VersionedLocation::V4(dest)),
			Some(PARAALICE.into()),
			CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::V4(
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
		assert_ok!(AssetManager::set_asset_units_per_second(
			parachain::RuntimeOrigin::root(),
			source_location,
			1u128,
			0
		));
	});

	let dest: Location = AccountKey20 {
		network: None,
		key: PARAALICE,
	}
	.into();
	Relay::execute_with(|| {
		assert_ok!(RelayChainPalletXcm::reserve_transfer_assets(
			relay_chain::RuntimeOrigin::signed(RELAYALICE),
			Box::new(Parachain(1).into()),
			Box::new(VersionedLocation::V4(dest).clone().into()),
			Box::new(([] /* Here */, 4000009100u128).into()),
			0,
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

	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_ok!(XTokens::transfer(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			parachain::CurrencyId::ForeignAsset(source_id),
			100,
			Box::new(VersionedLocation::V4(dest)),
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
		interior: /* Here */ [].into(),
	};

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

	let utility_bytes = parachain::MockTransactors::Relay.encode_call(
		xcm_primitives::UtilityAvailableCalls::AsDerivative(0, encoded),
	);

	let total_weight = 4000009000u64;
	// Root can directly pass the execution byes to the sovereign
	ParaA::execute_with(|| {
		assert_ok!(XcmTransactor::transact_through_sovereign(
			parachain::RuntimeOrigin::root(),
			Box::new(xcm::VersionedLocation::V4(dest)),
			Some(PARAALICE.into()),
			CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::V4(
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
		// free execution,x	 full amount received
		// 4000005186 refunded + 100 transferred = 4000005286
		assert_eq!(RelayBalances::free_balance(&para_a_account()), 4000005286);

		assert_eq!(RelayBalances::free_balance(&registered_address), 0);
	});
}

#[test]
fn test_automatic_versioning_on_runtime_upgrade_with_relay() {
	MockNet::reset();

	let source_location = parachain::AssetType::Xcm(xcm::v3::Location::parent());
	let asset_metadata = parachain::AssetMetadata {
		name: b"RelayToken".to_vec(),
		symbol: b"Relay".to_vec(),
		decimals: 12,
	};
	// register relay asset in parachain A and set XCM version to 1
	ParaA::execute_with(|| {
		parachain::XcmVersioner::set_version(1);
		assert_ok!(AssetManager::register_foreign_asset(
			parachain::RuntimeOrigin::root(),
			source_location.clone(),
			asset_metadata,
			1u128,
			true
		));
		assert_ok!(AssetManager::set_asset_units_per_second(
			parachain::RuntimeOrigin::root(),
			source_location,
			0u128,
			0
		));
	});

	let response = Response::Version(2);
	let querier: Location = ([]/* Here */).into();

	// This is irrelevant, nothing will be done with this message,
	// but we need to pass a message as an argument to trigger the storage change
	let mock_message: Xcm<()> = Xcm(vec![QueryResponse {
		query_id: 0,
		response,
		max_weight: Weight::zero(),
		querier: Some(querier),
	}]);
	// The router is mocked, and we cannot use WrapVersion in ChildParachainRouter. So we will force
	// it directly here
	// Actually send relay asset to parachain
	let dest: Location = AccountKey20 {
		network: None,
		key: PARAALICE,
	}
	.into();

	Relay::execute_with(|| {
		// This sets the default version, for not known destinations
		assert_ok!(RelayChainPalletXcm::force_default_xcm_version(
			relay_chain::RuntimeOrigin::root(),
			Some(2)
		));

		// Wrap version, which sets VersionedStorage
		// This is necessary because the mock router does not use wrap_version, but
		// this is not necessary in prod
		assert_ok!(<RelayChainPalletXcm as WrapVersion>::wrap_version(
			&Parachain(1).into(),
			mock_message
		));

		// Transfer assets. Since it is an unknown destination, it will query for version
		assert_ok!(RelayChainPalletXcm::reserve_transfer_assets(
			relay_chain::RuntimeOrigin::signed(RELAYALICE),
			Box::new(Parachain(1).into()),
			Box::new(VersionedLocation::V4(dest).clone().into()),
			Box::new(([] /* Here */, 123).into()),
			0,
		));

		// Let's advance the relay. This should trigger the subscription message
		relay_chain::relay_roll_to(2);

		// queries should have been updated
		assert!(RelayChainPalletXcm::query(0).is_some());
	});

	let expected_supported_version: relay_chain::RuntimeEvent =
		pallet_xcm::Event::SupportedVersionChanged {
			location: Location {
				parents: 0,
				interior: [Parachain(1)].into(),
			},
			version: 1,
		}
		.into();

	Relay::execute_with(|| {
		// Assert that the events vector contains the version change
		assert!(relay_chain::relay_events().contains(&expected_supported_version));
	});

	// ParaA changes version to 2, and calls on_runtime_upgrade. This should notify the targets
	// of the new version change
	ParaA::execute_with(|| {
		// Set version
		parachain::XcmVersioner::set_version(2);
		// Do runtime upgrade
		parachain::on_runtime_upgrade();
		// Initialize block, to call on_initialize and notify targets
		parachain::para_roll_to(2);
		// Expect the event in the parachain
		assert!(parachain::para_events().iter().any(|e| matches!(
			e,
			parachain::RuntimeEvent::PolkadotXcm(pallet_xcm::Event::VersionChangeNotified {
				result: 2,
				..
			})
		)));
	});

	// This event should have been seen in the relay
	let expected_supported_version_2: relay_chain::RuntimeEvent =
		pallet_xcm::Event::SupportedVersionChanged {
			location: Location {
				parents: 0,
				interior: [Parachain(1)].into(),
			},
			version: 2,
		}
		.into();

	Relay::execute_with(|| {
		// Assert that the events vector contains the new version change
		assert!(relay_chain::relay_events().contains(&expected_supported_version_2));
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
		assert_ok!(AssetManager::set_asset_units_per_second(
			parachain::RuntimeOrigin::root(),
			source_location,
			0u128,
			0
		));
	});

	// Actually send relay asset to parachain
	let dest: Location = AccountKey20 {
		network: None,
		key: fresh_account,
	}
	.into();
	Relay::execute_with(|| {
		assert_ok!(RelayChainPalletXcm::reserve_transfer_assets(
			relay_chain::RuntimeOrigin::signed(RELAYALICE),
			Box::new(Parachain(1).into()),
			Box::new(VersionedLocation::V4(dest.clone()).clone().into()),
			Box::new(([] /* Here */, 123).into()),
			0,
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
		assert_ok!(RelayChainPalletXcm::reserve_transfer_assets(
			relay_chain::RuntimeOrigin::signed(RELAYALICE),
			Box::new(Parachain(1).into()),
			Box::new(VersionedLocation::V4(dest).clone().into()),
			Box::new(([] /* Here */, 123).into()),
			0,
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
		assert_ok!(AssetManager::set_asset_units_per_second(
			parachain::RuntimeOrigin::root(),
			source_location,
			0u128,
			0
		));
	});

	// Actually send relay asset to parachain
	let dest: Location = AccountKey20 {
		network: None,
		key: fresh_account,
	}
	.into();
	Relay::execute_with(|| {
		assert_ok!(RelayChainPalletXcm::reserve_transfer_assets(
			relay_chain::RuntimeOrigin::signed(RELAYALICE),
			Box::new(Parachain(1).into()),
			Box::new(VersionedLocation::V4(dest.clone()).clone().into()),
			Box::new(([] /* Here */, 123).into()),
			0,
		));
	});

	// parachain should have received assets
	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_eq!(Assets::balance(source_id, &fresh_account.into()), 123);
	});
}

#[test]
fn evm_account_receiving_assets_should_handle_sufficients_ref_count() {
	MockNet::reset();

	let mut sufficient_account = [0u8; 20];
	sufficient_account[0..20].copy_from_slice(&evm_account()[..]);

	let evm_account_id = parachain::AccountId::from(sufficient_account);

	// Evm account is self sufficient
	ParaA::execute_with(|| {
		assert_eq!(parachain::System::account(evm_account_id).sufficients, 1);
	});

	let source_location = parachain::AssetType::Xcm(xcm::v3::Location::parent());
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
		assert_ok!(AssetManager::set_asset_units_per_second(
			parachain::RuntimeOrigin::root(),
			source_location,
			0u128,
			0
		));
	});

	// Actually send relay asset to parachain
	let dest: Location = AccountKey20 {
		network: None,
		key: sufficient_account,
	}
	.into();
	Relay::execute_with(|| {
		assert_ok!(RelayChainPalletXcm::reserve_transfer_assets(
			relay_chain::RuntimeOrigin::signed(RELAYALICE),
			Box::new(Parachain(1).into()),
			Box::new(VersionedLocation::V4(dest.clone()).clone().into()),
			Box::new(([] /* Here */, 123).into()),
			0,
		));
	});

	// Evm account sufficient ref count increased by 1.
	ParaA::execute_with(|| {
		// TODO: since the suicided logic was introduced an smart contract account
		// is not deleted completely until it's data is deleted. Data deletion
		// will be implemented in a future release
		// assert_eq!(parachain::System::account(evm_account_id).sufficients, 2);
	});

	ParaA::execute_with(|| {
		// Remove the account from the evm context.
		parachain::EVM::remove_account(&evm_account());
		// Evm account sufficient ref count decreased by 1.
		// TODO: since the suicided logic was introduced an smart contract account
		// is not deleted completely until it's data is deleted. Data deletion
		// will be implemented in a future release
		// assert_eq!(parachain::System::account(evm_account_id).sufficients, 1);
	});
}

#[test]
fn empty_account_should_not_be_reset() {
	MockNet::reset();

	// Test account has nonce 1 on genesis.
	let mut sufficient_account = [0u8; 20];
	sufficient_account[0..20].copy_from_slice(&evm_account()[..]);

	let evm_account_id = parachain::AccountId::from(sufficient_account);

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
		assert_ok!(AssetManager::set_asset_units_per_second(
			parachain::RuntimeOrigin::root(),
			source_location,
			0u128,
			0
		));
	});

	// Send native token to evm_account
	ParaA::execute_with(|| {
		assert_ok!(ParaBalances::transfer_allow_death(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			evm_account_id,
			100
		));
	});

	// Actually send relay asset to parachain
	let dest: Location = AccountKey20 {
		network: None,
		key: sufficient_account,
	}
	.into();
	Relay::execute_with(|| {
		assert_ok!(RelayChainPalletXcm::reserve_transfer_assets(
			relay_chain::RuntimeOrigin::signed(RELAYALICE),
			Box::new(Parachain(1).into()),
			Box::new(VersionedLocation::V4(dest.clone()).clone().into()),
			Box::new(([] /* Here */, 123).into()),
			0,
		));
	});

	ParaA::execute_with(|| {
		// Empty the assets from the account.
		// As this makes the account go below the `min_balance`, the account is considered dead
		// at eyes of pallet-assets, and the consumer reference is decreased by 1 and is now Zero.
		assert_ok!(parachain::Assets::transfer(
			parachain::RuntimeOrigin::signed(evm_account_id),
			source_id,
			PARAALICE.into(),
			123
		));
		// Verify account asset balance is Zero.
		assert_eq!(
			parachain::Assets::balance(source_id, &evm_account_id.into()),
			0
		);
		// Because we no longer have consumer references, we can set the balance to Zero.
		// This would reset the account if our ED were to be > than Zero.
		assert_ok!(ParaBalances::force_set_balance(
			parachain::RuntimeOrigin::root(),
			evm_account_id,
			0,
		));
		// Verify account native balance is Zero.
		assert_eq!(ParaBalances::free_balance(&evm_account_id), 0);
		// Remove the account from the evm context.
		// This decreases the sufficients reference by 1 and now is Zero.
		parachain::EVM::remove_account(&evm_account());
		// Verify reference count.
		let account = parachain::System::account(evm_account_id);
		assert_eq!(account.sufficients, 0);
		assert_eq!(account.consumers, 0);
		assert_eq!(account.providers, 1);
		// We expect the account to be alive in a Zero ED context.
		assert_eq!(parachain::System::account_nonce(evm_account_id), 1);
	});
}

#[test]
fn test_statemint_like() {
	MockNet::reset();

	let dest_para = Location::new(1, [Parachain(1)]);

	let sov = xcm_builder::SiblingParachainConvertsVia::<
		polkadot_parachain::primitives::Sibling,
		statemint_like::AccountId,
	>::convert_location(&dest_para)
	.unwrap();

	let statemint_asset_a_balances = Location::new(
		1,
		[
			Parachain(4),
			PalletInstance(5),
			xcm::latest::prelude::GeneralIndex(0u128),
		],
	);
	let source_location = parachain::AssetType::Xcm(
		xcm_builder::WithLatestLocationConverter::convert(&statemint_asset_a_balances)
			.expect("convert to v3"),
	);
	let source_id: parachain::AssetId = source_location.clone().into();

	let asset_metadata = parachain::AssetMetadata {
		name: b"StatemintToken".to_vec(),
		symbol: b"StatemintToken".to_vec(),
		decimals: 12,
	};

	ParaA::execute_with(|| {
		assert_ok!(AssetManager::register_foreign_asset(
			parachain::RuntimeOrigin::root(),
			source_location.clone(),
			asset_metadata.clone(),
			1u128,
			true
		));
		assert_ok!(AssetManager::set_asset_units_per_second(
			parachain::RuntimeOrigin::root(),
			source_location,
			0u128,
			0
		));
	});

	Statemint::execute_with(|| {
		// Set new prefix
		statemint_like::PrefixChanger::set_prefix(
			PalletInstance(<StatemintAssets as PalletInfoAccess>::index() as u8).into(),
		);

		assert_ok!(StatemintAssets::create(
			statemint_like::RuntimeOrigin::signed(RELAYALICE),
			0,
			RELAYALICE,
			1
		));

		assert_ok!(StatemintAssets::mint(
			statemint_like::RuntimeOrigin::signed(RELAYALICE),
			0,
			RELAYALICE,
			300000000000000
		));

		// This is needed, since the asset is created as non-sufficient
		assert_ok!(StatemintBalances::transfer_allow_death(
			statemint_like::RuntimeOrigin::signed(RELAYALICE),
			sov,
			100000000000000
		));

		// Actually send relay asset to parachain
		let dest: Location = AccountKey20 {
			network: None,
			key: PARAALICE,
		}
		.into();

		// Send with new prefix
		assert_ok!(StatemintChainPalletXcm::reserve_transfer_assets(
			statemint_like::RuntimeOrigin::signed(RELAYALICE),
			Box::new(Location::new(1, [Parachain(1)]).into()),
			Box::new(VersionedLocation::V4(dest).clone().into()),
			Box::new(
				(
					[
						xcm::latest::prelude::PalletInstance(
							<StatemintAssets as PalletInfoAccess>::index() as u8
						),
						xcm::latest::prelude::GeneralIndex(0),
					],
					123
				)
					.into()
			),
			0,
		));
	});

	ParaA::execute_with(|| {
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 123);
	});
}

#[test]
fn send_statemint_asset_from_para_a_to_statemint_with_relay_fee() {
	MockNet::reset();

	// Relay asset
	let relay_location = parachain::AssetType::Xcm(xcm::v3::Location::parent());
	let source_relay_id: parachain::AssetId = relay_location.clone().into();

	let relay_asset_metadata = parachain::AssetMetadata {
		name: b"RelayToken".to_vec(),
		symbol: b"Relay".to_vec(),
		decimals: 12,
	};

	// Statemint asset
	let statemint_asset = Location::new(
		1,
		[Parachain(4u32), PalletInstance(5u8), GeneralIndex(10u128)],
	);
	let statemint_location_asset = parachain::AssetType::Xcm(
		xcm_builder::WithLatestLocationConverter::convert(&statemint_asset).expect("convert to v3"),
	);
	let source_statemint_asset_id: parachain::AssetId = statemint_location_asset.clone().into();

	let asset_metadata_statemint_asset = parachain::AssetMetadata {
		name: b"USDC".to_vec(),
		symbol: b"USDC".to_vec(),
		decimals: 12,
	};

	let dest_para = Location::new(1, [Parachain(1)]);

	let sov = xcm_builder::SiblingParachainConvertsVia::<
		polkadot_parachain::primitives::Sibling,
		statemint_like::AccountId,
	>::convert_location(&dest_para)
	.unwrap();

	ParaA::execute_with(|| {
		assert_ok!(AssetManager::register_foreign_asset(
			parachain::RuntimeOrigin::root(),
			relay_location.clone(),
			relay_asset_metadata,
			1u128,
			true
		));
		assert_ok!(AssetManager::set_asset_units_per_second(
			parachain::RuntimeOrigin::root(),
			relay_location,
			0u128,
			0
		));

		assert_ok!(AssetManager::register_foreign_asset(
			parachain::RuntimeOrigin::root(),
			statemint_location_asset.clone(),
			asset_metadata_statemint_asset,
			1u128,
			true
		));
		assert_ok!(AssetManager::set_asset_units_per_second(
			parachain::RuntimeOrigin::root(),
			statemint_location_asset,
			0u128,
			1
		));
	});

	let parachain_beneficiary_from_relay: Location = Junction::AccountKey20 {
		network: None,
		key: PARAALICE,
	}
	.into();

	// Send relay chain asset to Alice in Parachain A
	Relay::execute_with(|| {
		assert_ok!(RelayChainPalletXcm::reserve_transfer_assets(
			relay_chain::RuntimeOrigin::signed(RELAYALICE),
			Box::new(Parachain(1).into()),
			Box::new(
				VersionedLocation::V4(parachain_beneficiary_from_relay)
					.clone()
					.into()
			),
			Box::new(([] /* Here */, 200).into()),
			0,
		));
	});

	Statemint::execute_with(|| {
		// Set new prefix
		statemint_like::PrefixChanger::set_prefix(
			PalletInstance(<StatemintAssets as PalletInfoAccess>::index() as u8).into(),
		);

		assert_ok!(StatemintAssets::create(
			statemint_like::RuntimeOrigin::signed(RELAYALICE),
			10,
			RELAYALICE,
			1
		));

		assert_ok!(StatemintAssets::mint(
			statemint_like::RuntimeOrigin::signed(RELAYALICE),
			10,
			RELAYALICE,
			300000000000000
		));

		// Send some native statemint tokens to sovereign for fees.
		// We can't pay fees with USDC as the asset is minted as non-sufficient.
		assert_ok!(StatemintBalances::transfer_allow_death(
			statemint_like::RuntimeOrigin::signed(RELAYALICE),
			sov,
			100000000000000
		));

		// Send statemint USDC asset to Alice in Parachain A
		let parachain_beneficiary_from_statemint: Location = AccountKey20 {
			network: None,
			key: PARAALICE,
		}
		.into();

		// Send with new prefix
		assert_ok!(StatemintChainPalletXcm::reserve_transfer_assets(
			statemint_like::RuntimeOrigin::signed(RELAYALICE),
			Box::new(Location::new(1, [Parachain(1)]).into()),
			Box::new(
				VersionedLocation::V4(parachain_beneficiary_from_statemint)
					.clone()
					.into()
			),
			Box::new(
				(
					[
						xcm::latest::prelude::PalletInstance(
							<StatemintAssets as PalletInfoAccess>::index() as u8
						),
						GeneralIndex(10),
					],
					125
				)
					.into()
			),
			0,
		));
	});

	let statemint_beneficiary = Location {
		parents: 1,
		interior: [
			Parachain(4),
			AccountId32 {
				network: None,
				id: RELAYBOB.into(),
			},
		]
		.into(),
	};

	ParaA::execute_with(|| {
		// Alice has received 125 USDC
		assert_eq!(
			Assets::balance(source_statemint_asset_id, &PARAALICE.into()),
			125
		);

		// Alice has received 200 Relay assets
		assert_eq!(Assets::balance(source_relay_id, &PARAALICE.into()), 200);
	});

	Statemint::execute_with(|| {
		// Check that BOB's balance is empty before the transfer
		assert_eq!(StatemintAssets::account_balances(RELAYBOB), vec![]);
	});

	// Transfer USDC from Parachain A to Statemint using Relay asset as fee
	ParaA::execute_with(|| {
		assert_ok!(XTokens::transfer_multicurrencies(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			vec![
				(
					parachain::CurrencyId::ForeignAsset(source_statemint_asset_id),
					100
				),
				(parachain::CurrencyId::ForeignAsset(source_relay_id), 100)
			],
			1,
			Box::new(VersionedLocation::V4(statemint_beneficiary)),
			WeightLimit::Limited(Weight::from_parts(80_000_000u64, 100_000u64))
		));
	});

	ParaA::execute_with(|| {
		// Alice has 100 USDC less
		assert_eq!(
			Assets::balance(source_statemint_asset_id, &PARAALICE.into()),
			25
		);

		// Alice has 100 relay asset less
		assert_eq!(Assets::balance(source_relay_id, &PARAALICE.into()), 100);
	});

	Statemint::execute_with(|| {
		println!("STATEMINT EVENTS: {:?}", parachain::para_events());
		// Check that BOB received 100 USDC on statemint
		assert_eq!(StatemintAssets::account_balances(RELAYBOB), vec![(10, 100)]);
	});
}

#[test]
fn transact_through_signed_multilocation() {
	MockNet::reset();
	let mut ancestry = Location::parent();

	ParaA::execute_with(|| {
		// Root can set transact info
		assert_ok!(XcmTransactor::set_transact_info(
			parachain::RuntimeOrigin::root(),
			Box::new(xcm::VersionedLocation::V4(Location::parent())),
			// Relay charges 1000 for every instruction, and we have 3, so 3000
			3000.into(),
			20000000000.into(),
			// 4 instructions in transact through signed
			Some(4000.into())
		));
		// Root can set transact info
		assert_ok!(XcmTransactor::set_fee_per_second(
			parachain::RuntimeOrigin::root(),
			Box::new(xcm::VersionedLocation::V4(Location::parent())),
			WEIGHT_REF_TIME_PER_SECOND as u128,
		));
		ancestry = parachain::UniversalLocation::get().into();
	});

	// Let's construct the Junction that we will append with DescendOrigin
	let signed_origin: Junctions = [AccountKey20 {
		network: None,
		key: PARAALICE,
	}]
	.into();

	let mut descend_origin_multilocation = parachain::SelfLocation::get();
	descend_origin_multilocation
		.append_with(signed_origin)
		.unwrap();

	// To convert it to what the relay will see instead of us
	descend_origin_multilocation
		.reanchor(&Location::parent(), &ancestry.interior)
		.unwrap();

	let derived = xcm_builder::Account32Hash::<
		relay_chain::KusamaNetwork,
		relay_chain::AccountId,
	>::convert_location(&descend_origin_multilocation)
	.unwrap();

	Relay::execute_with(|| {
		// free execution, full amount received
		assert_ok!(RelayBalances::transfer_allow_death(
			relay_chain::RuntimeOrigin::signed(RELAYALICE),
			derived.clone(),
			4000004100u128,
		));
		// derived account has all funds
		assert!(RelayBalances::free_balance(&derived) == 4000004100);
		// sovereign account has 0 funds
		assert!(RelayBalances::free_balance(&para_a_account()) == 0);
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
		// 100 to sovereign
		dest: para_a_account(),
		value: 100u32.into(),
	}
	.encode();
	encoded.append(&mut call_bytes);

	ParaA::execute_with(|| {
		assert_ok!(XcmTransactor::transact_through_signed(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			Box::new(xcm::VersionedLocation::V4(Location::parent())),
			CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::V4(
					Location::parent()
				))),
				fee_amount: None
			},
			encoded,
			// 4000000000 for transfer + 4000 for XCM
			TransactWeights {
				transact_required_weight_at_most: 4000000000.into(),
				overall_weight: None
			},
			false
		));
	});

	Relay::execute_with(|| {
		assert!(RelayBalances::free_balance(&para_a_account()) == 100);

		assert!(RelayBalances::free_balance(&derived) == 0);
	});
}

#[test]
fn transact_through_signed_multilocation_custom_fee_and_weight() {
	MockNet::reset();
	let mut ancestry = Location::parent();

	ParaA::execute_with(|| {
		ancestry = parachain::UniversalLocation::get().into();
	});

	// Let's construct the Junction that we will append with DescendOrigin
	let signed_origin: Junctions = [AccountKey20 {
		network: None,
		key: PARAALICE,
	}]
	.into();

	let mut descend_origin_multilocation = parachain::SelfLocation::get();
	descend_origin_multilocation
		.append_with(signed_origin)
		.unwrap();

	// To convert it to what the relay will see instead of us
	descend_origin_multilocation
		.reanchor(&Location::parent(), &ancestry.interior)
		.unwrap();

	let derived = xcm_builder::Account32Hash::<
		relay_chain::KusamaNetwork,
		relay_chain::AccountId,
	>::convert_location(&descend_origin_multilocation)
	.unwrap();

	Relay::execute_with(|| {
		// free execution, full amount received
		assert_ok!(RelayBalances::transfer_allow_death(
			relay_chain::RuntimeOrigin::signed(RELAYALICE),
			derived.clone(),
			4000004100u128,
		));
		// derived account has all funds
		assert!(RelayBalances::free_balance(&derived) == 4000004100);
		// sovereign account has 0 funds
		assert!(RelayBalances::free_balance(&para_a_account()) == 0);
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
		// 100 to sovereign
		dest: para_a_account(),
		value: 100u32.into(),
	}
	.encode();
	encoded.append(&mut call_bytes);

	let total_weight = 4000004000u64;
	ParaA::execute_with(|| {
		assert_ok!(XcmTransactor::transact_through_signed(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			Box::new(xcm::VersionedLocation::V4(Location::parent())),
			CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::V4(
					Location::parent()
				))),
				fee_amount: Some(total_weight as u128)
			},
			encoded,
			// 4000000000 for transfer + 4000 for XCM
			TransactWeights {
				transact_required_weight_at_most: 4000000000.into(),
				overall_weight: Some(Limited(total_weight.into()))
			},
			false
		));
	});

	Relay::execute_with(|| {
		assert!(RelayBalances::free_balance(&para_a_account()) == 100);

		assert!(RelayBalances::free_balance(&derived) == 0);
	});
}

#[test]
fn transact_through_signed_multilocation_custom_fee_and_weight_refund() {
	MockNet::reset();
	let mut ancestry = Location::parent();

	ParaA::execute_with(|| {
		ancestry = parachain::UniversalLocation::get().into();
	});

	// Let's construct the Junction that we will append with DescendOrigin
	let signed_origin: Junctions = [AccountKey20 {
		network: None,
		key: PARAALICE,
	}]
	.into();

	let mut descend_origin_multilocation = parachain::SelfLocation::get();
	descend_origin_multilocation
		.append_with(signed_origin)
		.unwrap();

	// To convert it to what the relay will see instead of us
	descend_origin_multilocation
		.reanchor(&Location::parent(), &ancestry.interior)
		.unwrap();

	let derived = xcm_builder::Account32Hash::<
		relay_chain::KusamaNetwork,
		relay_chain::AccountId,
	>::convert_location(&descend_origin_multilocation)
	.unwrap();

	Relay::execute_with(|| {
		// free execution, full amount received
		assert_ok!(RelayBalances::transfer_allow_death(
			relay_chain::RuntimeOrigin::signed(RELAYALICE),
			derived.clone(),
			4000009100u128,
		));
		// derived account has all funds
		assert!(RelayBalances::free_balance(&derived) == 4000009100);
		// sovereign account has 0 funds
		assert!(RelayBalances::free_balance(&para_a_account()) == 0);
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
		// 100 to sovereign
		dest: para_a_account(),
		value: 100u32.into(),
	}
	.encode();
	encoded.append(&mut call_bytes);

	let total_weight = 4000009000u64;
	ParaA::execute_with(|| {
		assert_ok!(XcmTransactor::transact_through_signed(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			Box::new(xcm::VersionedLocation::V4(Location::parent())),
			CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::V4(
					Location::parent()
				))),
				fee_amount: Some(total_weight as u128)
			},
			encoded,
			// 4000000000 for transfer + 9000 for XCM
			TransactWeights {
				transact_required_weight_at_most: 4000000000.into(),
				overall_weight: Some(Limited(total_weight.into()))
			},
			true
		));
	});

	Relay::execute_with(|| {
		// 100 transferred
		assert_eq!(RelayBalances::free_balance(&para_a_account()), 100);

		// 4000005186 refunded
		assert_eq!(RelayBalances::free_balance(&derived), 4000005186);
	});
}

#[test]
fn transact_through_signed_multilocation_para_to_para() {
	MockNet::reset();
	let mut ancestry = Location::parent();

	let para_b_location = Location::new(1, [Parachain(2)]);

	let para_b_balances = Location::new(1, [Parachain(2), PalletInstance(1u8)]);

	ParaA::execute_with(|| {
		// Root can set transact info
		assert_ok!(XcmTransactor::set_transact_info(
			parachain::RuntimeOrigin::root(),
			// ParaB
			Box::new(xcm::VersionedLocation::V4(para_b_location.clone())),
			// Para charges 1000 for every instruction, and we have 3, so 3
			3.into(),
			20000000000.into(),
			// 4 instructions in transact through signed
			Some(4.into())
		));
		// Root can set transact info
		assert_ok!(XcmTransactor::set_fee_per_second(
			parachain::RuntimeOrigin::root(),
			Box::new(xcm::VersionedLocation::V4(para_b_balances.clone())),
			parachain::ParaTokensPerSecond::get().1 as u128,
		));
		ancestry = parachain::UniversalLocation::get().into();
	});

	// Let's construct the Junction that we will append with DescendOrigin
	let signed_origin: Junctions = [AccountKey20 {
		network: None,
		key: PARAALICE,
	}]
	.into();

	let mut descend_origin_location = parachain::SelfLocation::get();
	descend_origin_location.append_with(signed_origin).unwrap();

	// To convert it to what the paraB will see instead of us
	descend_origin_location
		.reanchor(&para_b_location, &ancestry.interior)
		.unwrap();

	let derived = xcm_builder::HashedDescription::<
		parachain::AccountId,
		xcm_builder::DescribeFamily<xcm_builder::DescribeAllTerminal>,
	>::convert_location(&descend_origin_location)
	.unwrap();

	ParaB::execute_with(|| {
		// free execution, full amount received
		assert_ok!(ParaBalances::transfer_allow_death(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			derived.clone(),
			4000000104u128,
		));
		// derived account has all funds
		assert!(ParaBalances::free_balance(&derived) == 4000000104);
		// sovereign account has 0 funds
		assert!(ParaBalances::free_balance(&para_a_account_20()) == 0);
	});

	// Encode the call. Balances transact to para_a_account
	// First index
	let mut encoded: Vec<u8> = Vec::new();
	let index =
		<parachain::Runtime as frame_system::Config>::PalletInfo::index::<parachain::Balances>()
			.unwrap() as u8;

	encoded.push(index);

	// Then call bytes
	let mut call_bytes = pallet_balances::Call::<parachain::Runtime>::transfer_allow_death {
		// 100 to sovereign
		dest: para_a_account_20(),
		value: 100u32.into(),
	}
	.encode();
	encoded.append(&mut call_bytes);

	ParaA::execute_with(|| {
		assert_ok!(XcmTransactor::transact_through_signed(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			Box::new(xcm::VersionedLocation::V4(para_b_location)),
			CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::V4(
					para_b_balances
				))),
				fee_amount: None
			},
			encoded,
			// 4000000000 for transfer + 4000 for XCM
			// 1-1 to fee
			TransactWeights {
				transact_required_weight_at_most: 4000000000.into(),
				overall_weight: None
			},
			false
		));
	});

	ParaB::execute_with(|| {
		assert!(ParaBalances::free_balance(&derived) == 0);

		assert!(ParaBalances::free_balance(&para_a_account_20()) == 100);
	});
}

#[test]
fn transact_through_signed_multilocation_para_to_para_refund() {
	MockNet::reset();
	let mut ancestry = Location::parent();

	let para_b_location = Location::new(1, [Parachain(2)]);

	let para_b_balances = Location::new(1, [Parachain(2), PalletInstance(1u8)]);

	ParaA::execute_with(|| {
		assert_ok!(XcmTransactor::set_fee_per_second(
			parachain::RuntimeOrigin::root(),
			Box::new(xcm::VersionedLocation::V4(para_b_balances.clone())),
			parachain::ParaTokensPerSecond::get().1 as u128,
		));
		ancestry = parachain::UniversalLocation::get().into();
	});

	// Let's construct the Junction that we will append with DescendOrigin
	let signed_origin: Junctions = [AccountKey20 {
		network: None,
		key: PARAALICE,
	}]
	.into();

	let mut descend_origin_location = parachain::SelfLocation::get();
	descend_origin_location.append_with(signed_origin).unwrap();

	// To convert it to what the paraB will see instead of us
	descend_origin_location
		.reanchor(&para_b_location, &ancestry.interior)
		.unwrap();

	let derived = xcm_builder::HashedDescription::<
		parachain::AccountId,
		xcm_builder::DescribeFamily<xcm_builder::DescribeAllTerminal>,
	>::convert_location(&descend_origin_location)
	.unwrap();

	ParaB::execute_with(|| {
		// free execution, full amount received
		assert_ok!(ParaBalances::transfer_allow_death(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			derived.clone(),
			4000009100u128,
		));
		// derived account has all funds
		assert!(ParaBalances::free_balance(&derived) == 4000009100);
		// sovereign account has 0 funds
		assert!(ParaBalances::free_balance(&para_a_account_20()) == 0);
	});

	// Encode the call. Balances transact to para_a_account
	// First index
	let mut encoded: Vec<u8> = Vec::new();
	let index =
		<parachain::Runtime as frame_system::Config>::PalletInfo::index::<parachain::Balances>()
			.unwrap() as u8;

	encoded.push(index);

	// Then call bytes
	let mut call_bytes = pallet_balances::Call::<parachain::Runtime>::transfer_allow_death {
		// 100 to sovereign
		dest: para_a_account_20(),
		value: 100u32.into(),
	}
	.encode();
	encoded.append(&mut call_bytes);

	let overall_weight = 4000009000u64;
	ParaA::execute_with(|| {
		assert_ok!(XcmTransactor::transact_through_signed(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			Box::new(xcm::VersionedLocation::V4(para_b_location)),
			CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::V4(
					para_b_balances
				))),
				fee_amount: Some(overall_weight as u128)
			},
			encoded,
			// 4000000000 for transfer + 9000 for XCM
			TransactWeights {
				transact_required_weight_at_most: 4000000000.into(),
				overall_weight: Some(Limited(overall_weight.into()))
			},
			true
		));
	});

	ParaB::execute_with(|| {
		// Check the derived account was refunded
		assert_eq!(ParaBalances::free_balance(&derived), 8993);

		// Check the transfer was executed
		assert_eq!(ParaBalances::free_balance(&para_a_account_20()), 100);
	});
}

#[test]
fn transact_through_signed_multilocation_para_to_para_ethereum() {
	MockNet::reset();
	let mut ancestry = Location::parent();

	let para_b_location = Location::new(1, [Parachain(2)]);

	let para_b_balances = Location::new(1, [Parachain(2), PalletInstance(1u8)]);

	ParaA::execute_with(|| {
		// Root can set transact info
		assert_ok!(XcmTransactor::set_transact_info(
			parachain::RuntimeOrigin::root(),
			// ParaB
			Box::new(xcm::VersionedLocation::V4(para_b_location.clone())),
			// Para charges 1000 for every instruction, and we have 3, so 3
			3.into(),
			20000000000.into(),
			// 4 instructions in transact through signed
			Some(4.into())
		));
		// Root can set transact info
		assert_ok!(XcmTransactor::set_fee_per_second(
			parachain::RuntimeOrigin::root(),
			Box::new(xcm::VersionedLocation::V4(para_b_balances.clone())),
			parachain::ParaTokensPerSecond::get().1 as u128,
		));
		ancestry = parachain::UniversalLocation::get().into();
	});

	// Let's construct the Junction that we will append with DescendOrigin
	let signed_origin: Junctions = [AccountKey20 {
		network: None,
		key: PARAALICE,
	}]
	.into();

	let mut descend_origin_location = parachain::SelfLocation::get();
	descend_origin_location.append_with(signed_origin).unwrap();

	// To convert it to what the paraB will see instead of us
	descend_origin_location
		.reanchor(&para_b_location, &ancestry.interior)
		.unwrap();

	let derived = xcm_builder::HashedDescription::<
		parachain::AccountId,
		xcm_builder::DescribeFamily<xcm_builder::DescribeAllTerminal>,
	>::convert_location(&descend_origin_location)
	.unwrap();

	let mut parachain_b_alice_balances_before = 0;
	ParaB::execute_with(|| {
		assert_ok!(ParaBalances::transfer_allow_death(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			derived.clone(),
			4000000104u128,
		));
		// derived account has all funds
		assert!(ParaBalances::free_balance(&derived) == 4000000104);
		// sovereign account has 0 funds
		assert!(ParaBalances::free_balance(&para_a_account_20()) == 0);

		parachain_b_alice_balances_before = ParaBalances::free_balance(&PARAALICE.into())
	});

	// Encode the call. Balances transact to para_a_account
	// First index
	let mut encoded: Vec<u8> = Vec::new();
	let index =
		<parachain::Runtime as frame_system::Config>::PalletInfo::index::<parachain::EthereumXcm>()
			.unwrap() as u8;

	encoded.push(index);

	use sp_core::U256;
	// Let's do a EVM transfer
	let eth_tx =
		xcm_primitives::EthereumXcmTransaction::V1(xcm_primitives::EthereumXcmTransactionV1 {
			gas_limit: U256::from(21000),
			fee_payment: xcm_primitives::EthereumXcmFee::Auto,
			action: pallet_ethereum::TransactionAction::Call(PARAALICE.into()),
			value: U256::from(100),
			input: BoundedVec::<
				u8,
				ConstU32<{ xcm_primitives::MAX_ETHEREUM_XCM_INPUT_SIZE }>
			>::try_from(vec![]).unwrap(),
			access_list: None,
		});

	// Then call bytes
	let mut call_bytes = pallet_ethereum_xcm::Call::<parachain::Runtime>::transact {
		xcm_transaction: eth_tx,
	}
	.encode();
	encoded.append(&mut call_bytes);

	ParaA::execute_with(|| {
		assert_ok!(XcmTransactor::transact_through_signed(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			Box::new(xcm::VersionedLocation::V4(para_b_location)),
			CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::V4(
					para_b_balances
				))),
				fee_amount: None
			},
			encoded,
			// 4000000000 for transfer + 4000 for XCM
			// 1-1 to fee
			TransactWeights {
				transact_required_weight_at_most: 4000000000.into(),
				overall_weight: None
			},
			false
		));
	});

	ParaB::execute_with(|| {
		// Make sure the EVM transfer went through
		assert!(
			ParaBalances::free_balance(&PARAALICE.into())
				== parachain_b_alice_balances_before + 100
		);
	});
}

#[test]
fn transact_through_signed_multilocation_para_to_para_ethereum_no_proxy_fails() {
	MockNet::reset();
	let mut ancestry = Location::parent();

	let para_b_location = Location::new(1, [Parachain(2)]);

	let para_b_balances = Location::new(1, [Parachain(2), PalletInstance(1u8)]);

	ParaA::execute_with(|| {
		// Root can set transact info
		assert_ok!(XcmTransactor::set_transact_info(
			parachain::RuntimeOrigin::root(),
			// ParaB
			Box::new(xcm::VersionedLocation::V4(para_b_location.clone())),
			// Para charges 1000 for every instruction, and we have 3, so 3
			3.into(),
			20000000000.into(),
			// 4 instructions in transact through signed
			Some(4.into())
		));
		// Root can set transact info
		assert_ok!(XcmTransactor::set_fee_per_second(
			parachain::RuntimeOrigin::root(),
			Box::new(xcm::VersionedLocation::V4(para_b_balances.clone())),
			parachain::ParaTokensPerSecond::get().1 as u128,
		));
		ancestry = parachain::UniversalLocation::get().into();
	});

	// Let's construct the Junction that we will append with DescendOrigin
	let signed_origin: Junctions = [AccountKey20 {
		network: None,
		key: PARAALICE,
	}]
	.into();

	let mut descend_origin_location = parachain::SelfLocation::get();
	descend_origin_location.append_with(signed_origin).unwrap();

	// To convert it to what the paraB will see instead of us
	descend_origin_location
		.reanchor(&para_b_location, &ancestry.interior)
		.unwrap();

	let derived = xcm_builder::HashedDescription::<
		parachain::AccountId,
		xcm_builder::DescribeFamily<xcm_builder::DescribeAllTerminal>,
	>::convert_location(&descend_origin_location)
	.unwrap();

	let mut parachain_b_alice_balances_before = 0;
	ParaB::execute_with(|| {
		assert_ok!(ParaBalances::transfer_allow_death(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			derived.clone(),
			4000000104u128,
		));
		// derived account has all funds
		assert!(ParaBalances::free_balance(&derived) == 4000000104);
		// sovereign account has 0 funds
		assert!(ParaBalances::free_balance(&para_a_account_20()) == 0);

		parachain_b_alice_balances_before = ParaBalances::free_balance(&PARAALICE.into())
	});

	// Encode the call. Balances transact to para_a_account
	// First index
	let mut encoded: Vec<u8> = Vec::new();
	let index =
		<parachain::Runtime as frame_system::Config>::PalletInfo::index::<parachain::EthereumXcm>()
			.unwrap() as u8;

	encoded.push(index);

	use sp_core::U256;
	// Let's do a EVM transfer
	let eth_tx =
		xcm_primitives::EthereumXcmTransaction::V1(xcm_primitives::EthereumXcmTransactionV1 {
			gas_limit: U256::from(21000),
			fee_payment: xcm_primitives::EthereumXcmFee::Auto,
			action: pallet_ethereum::TransactionAction::Call(PARAALICE.into()),
			value: U256::from(100),
			input: BoundedVec::<
				u8,
				ConstU32<{ xcm_primitives::MAX_ETHEREUM_XCM_INPUT_SIZE }>
			>::try_from(vec![]).unwrap(),
			access_list: None,
		});

	// Then call bytes
	let mut call_bytes = pallet_ethereum_xcm::Call::<parachain::Runtime>::transact_through_proxy {
		transact_as: PARAALICE.into(),
		xcm_transaction: eth_tx,
	}
	.encode();
	encoded.append(&mut call_bytes);

	ParaA::execute_with(|| {
		assert_ok!(XcmTransactor::transact_through_signed(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			Box::new(xcm::VersionedLocation::V4(para_b_location)),
			CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::V4(
					para_b_balances
				))),
				fee_amount: None
			},
			encoded,
			TransactWeights {
				transact_required_weight_at_most: 4000000000.into(),
				overall_weight: None
			},
			false
		));
	});

	ParaB::execute_with(|| {
		// Make sure the EVM transfer wasn't executed
		assert!(ParaBalances::free_balance(&PARAALICE.into()) == parachain_b_alice_balances_before);
	});
}

#[test]
fn transact_through_signed_multilocation_para_to_para_ethereum_proxy_succeeds() {
	MockNet::reset();
	let mut ancestry = Location::parent();

	let para_b_location = Location::new(1, [Parachain(2)]);

	let para_b_balances = Location::new(1, [Parachain(2), PalletInstance(1u8)]);

	ParaA::execute_with(|| {
		// Root can set transact info
		assert_ok!(XcmTransactor::set_transact_info(
			parachain::RuntimeOrigin::root(),
			// ParaB
			Box::new(xcm::VersionedLocation::V4(para_b_location.clone())),
			// Para charges 1000 for every instruction, and we have 3, so 3
			3.into(),
			20000000000.into(),
			// 4 instructions in transact through signed
			Some(4.into())
		));
		// Root can set transact info
		assert_ok!(XcmTransactor::set_fee_per_second(
			parachain::RuntimeOrigin::root(),
			Box::new(xcm::VersionedLocation::V4(para_b_balances.clone())),
			parachain::ParaTokensPerSecond::get().1 as u128,
		));
		ancestry = parachain::UniversalLocation::get().into();
	});

	// Let's construct the Junction that we will append with DescendOrigin
	let signed_origin: Junctions = [AccountKey20 {
		network: None,
		key: PARAALICE,
	}]
	.into();

	let mut descend_origin_location = parachain::SelfLocation::get();
	descend_origin_location.append_with(signed_origin).unwrap();

	// To convert it to what the paraB will see instead of us
	descend_origin_location
		.reanchor(&para_b_location, &ancestry.interior)
		.unwrap();

	let derived = xcm_builder::HashedDescription::<
		parachain::AccountId,
		xcm_builder::DescribeFamily<xcm_builder::DescribeAllTerminal>,
	>::convert_location(&descend_origin_location)
	.unwrap();

	let transfer_recipient = evm_account();
	let mut transfer_recipient_balance_before = 0;
	ParaB::execute_with(|| {
		assert_ok!(ParaBalances::transfer_allow_death(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			derived.clone(),
			4000000104u128,
		));
		// derived account has all funds
		assert!(ParaBalances::free_balance(&derived) == 4000000104);
		// sovereign account has 0 funds
		assert!(ParaBalances::free_balance(&para_a_account_20()) == 0);

		transfer_recipient_balance_before = ParaBalances::free_balance(&transfer_recipient.into());

		// Add proxy ALICE  -> derived
		let _ = parachain::Proxy::add_proxy_delegate(
			&PARAALICE.into(),
			derived,
			parachain::ProxyType::Any,
			0,
		);
	});

	// Encode the call. Balances transact to para_a_account
	// First index
	let mut encoded: Vec<u8> = Vec::new();
	let index =
		<parachain::Runtime as frame_system::Config>::PalletInfo::index::<parachain::EthereumXcm>()
			.unwrap() as u8;

	encoded.push(index);

	use sp_core::U256;
	// Let's do a EVM transfer
	let eth_tx =
		xcm_primitives::EthereumXcmTransaction::V2(xcm_primitives::EthereumXcmTransactionV2 {
			gas_limit: U256::from(21000),
			action: pallet_ethereum::TransactionAction::Call(transfer_recipient.into()),
			value: U256::from(100),
			input: BoundedVec::<
				u8,
				ConstU32<{ xcm_primitives::MAX_ETHEREUM_XCM_INPUT_SIZE }>
			>::try_from(vec![]).unwrap(),
			access_list: None,
		});

	// Then call bytes
	let mut call_bytes = pallet_ethereum_xcm::Call::<parachain::Runtime>::transact_through_proxy {
		transact_as: PARAALICE.into(),
		xcm_transaction: eth_tx,
	}
	.encode();
	encoded.append(&mut call_bytes);

	ParaA::execute_with(|| {
		assert_ok!(XcmTransactor::transact_through_signed(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			Box::new(xcm::VersionedLocation::V4(para_b_location)),
			CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::V4(
					para_b_balances
				))),
				fee_amount: None
			},
			encoded,
			TransactWeights {
				transact_required_weight_at_most: 4000000000.into(),
				overall_weight: None
			},
			false
		));
	});

	ParaB::execute_with(|| {
		// Make sure the EVM transfer was executed
		assert!(
			ParaBalances::free_balance(&transfer_recipient.into())
				== transfer_recipient_balance_before + 100
		);
	});
}

#[test]
fn hrmp_init_accept_through_root() {
	MockNet::reset();

	Relay::execute_with(|| {
		assert_ok!(RelayBalances::transfer_allow_death(
			relay_chain::RuntimeOrigin::signed(RELAYALICE),
			para_a_account(),
			1000u128
		));
		assert_ok!(RelayBalances::transfer_allow_death(
			relay_chain::RuntimeOrigin::signed(RELAYALICE),
			para_b_account(),
			1000u128
		));
	});

	ParaA::execute_with(|| {
		let total_fee = 1_000u128;
		let total_weight: u64 = 1_000_000_000;
		let tx_weight: u64 = 500_000_000;
		// Root can send hrmp init channel
		assert_ok!(XcmTransactor::hrmp_manage(
			parachain::RuntimeOrigin::root(),
			HrmpOperation::InitOpen(HrmpInitParams {
				para_id: 2u32.into(),
				proposed_max_capacity: 1,
				proposed_max_message_size: 1
			}),
			CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::V4(
					Location::parent()
				))),
				fee_amount: Some(total_fee)
			},
			TransactWeights {
				transact_required_weight_at_most: tx_weight.into(),
				overall_weight: Some(Limited(total_weight.into()))
			}
		));
	});
	Relay::execute_with(|| {
		let expected_event: relay_chain::RuntimeEvent =
			polkadot_runtime_parachains::hrmp::Event::OpenChannelRequested {
				sender: 1u32.into(),
				recipient: 2u32.into(),
				proposed_max_capacity: 1u32,
				proposed_max_message_size: 1u32,
			}
			.into();
		assert!(relay_chain::relay_events().contains(&expected_event));
	});
	ParaB::execute_with(|| {
		let total_fee = 1_000u128;
		let total_weight: u64 = 1_000_000_000;
		let tx_weight: u64 = 500_000_000;
		// Root can send hrmp accept channel
		assert_ok!(XcmTransactor::hrmp_manage(
			parachain::RuntimeOrigin::root(),
			HrmpOperation::Accept {
				para_id: 1u32.into()
			},
			CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::V4(
					Location::parent()
				))),
				fee_amount: Some(total_fee)
			},
			TransactWeights {
				transact_required_weight_at_most: tx_weight.into(),
				overall_weight: Some(Limited(total_weight.into()))
			}
		));
	});

	Relay::execute_with(|| {
		let expected_event: relay_chain::RuntimeEvent =
			polkadot_runtime_parachains::hrmp::Event::OpenChannelAccepted {
				sender: 1u32.into(),
				recipient: 2u32.into(),
			}
			.into();
		assert!(relay_chain::relay_events().contains(&expected_event));
	});
}

#[test]
fn hrmp_close_works() {
	MockNet::reset();

	Relay::execute_with(|| {
		assert_ok!(RelayBalances::transfer_allow_death(
			relay_chain::RuntimeOrigin::signed(RELAYALICE),
			para_a_account(),
			1000u128
		));
		assert_ok!(Hrmp::force_open_hrmp_channel(
			relay_chain::RuntimeOrigin::root(),
			1u32.into(),
			2u32.into(),
			1u32,
			1u32
		));
		assert_ok!(Hrmp::force_process_hrmp_open(
			relay_chain::RuntimeOrigin::root(),
			1u32
		));
	});

	ParaA::execute_with(|| {
		let total_fee = 1_000u128;
		let total_weight: u64 = 1_000_000_000;
		let tx_weight: u64 = 500_000_000;
		// Root can send hrmp close
		assert_ok!(XcmTransactor::hrmp_manage(
			parachain::RuntimeOrigin::root(),
			HrmpOperation::Close(HrmpChannelId {
				sender: 1u32.into(),
				recipient: 2u32.into()
			}),
			CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::V4(
					Location::parent()
				))),
				fee_amount: Some(total_fee)
			},
			TransactWeights {
				transact_required_weight_at_most: tx_weight.into(),
				overall_weight: Some(Limited(total_weight.into()))
			}
		));
	});
	Relay::execute_with(|| {
		let expected_event: relay_chain::RuntimeEvent =
			polkadot_runtime_parachains::hrmp::Event::ChannelClosed {
				by_parachain: 1u32.into(),
				channel_id: HrmpChannelId {
					sender: 1u32.into(),
					recipient: 2u32.into(),
				},
			}
			.into();
		assert!(relay_chain::relay_events().contains(&expected_event));
	});
}

use parity_scale_codec::{Decode, Encode};
use sp_io::hashing::blake2_256;

// Helper to derive accountIds
pub fn derivative_account_id(who: sp_runtime::AccountId32, index: u16) -> sp_runtime::AccountId32 {
	let entropy = (b"modlpy/utilisuba", who, index).using_encoded(blake2_256);
	sp_runtime::AccountId32::decode(&mut &entropy[..]).expect("valid account id")
}
