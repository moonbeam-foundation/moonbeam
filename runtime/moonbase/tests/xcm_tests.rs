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

//! Moonbase Runtime Xcm Tets

mod xcm_mock;
use frame_support::assert_ok;
use xcm_mock::parachain;
use xcm_mock::relay_chain;
use xcm_mock::*;

use xcm::v1::{
	Junction::{self, AccountId32, AccountKey20, PalletInstance, Parachain},
	Junctions::*,
	MultiLocation, NetworkId,
};
use xcm_simulator::TestExt;

// Send a relay asset (like DOT) to a parachain A
#[test]
fn receive_relay_asset_from_relay() {
	MockNet::reset();

	let source_location = parachain::AssetType::Xcm(MultiLocation::parent());
	let source_id: parachain::AssetId = source_location.clone().into();
	let asset_metadata = parachain::AssetMetadata {
		name: b"RelayToken".to_vec(),
		symbol: b"Relay".to_vec(),
		decimals: 12,
	};
	// register relay asset in parachain A
	ParaA::execute_with(|| {
		assert_ok!(AssetManager::register_asset(
			parachain::Origin::root(),
			source_location,
			asset_metadata,
			1u128,
		));
		assert_ok!(AssetManager::set_asset_units_per_second(
			parachain::Origin::root(),
			source_id,
			0u128
		));
	});

	// Actually send relay asset to parachain
	let dest: MultiLocation = AccountKey20 {
		network: NetworkId::Any,
		key: PARAALICE,
	}
	.into();
	Relay::execute_with(|| {
		assert_ok!(RelayChainPalletXcm::reserve_transfer_assets(
			relay_chain::Origin::signed(RELAYALICE),
			Box::new(Parachain(1).into().into()),
			Box::new(dest.clone().into()),
			Box::new((Here, 123).into()),
			0,
			123,
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

	let source_location = parachain::AssetType::Xcm(MultiLocation::parent());
	let source_id: parachain::AssetId = source_location.clone().into();

	let asset_metadata = parachain::AssetMetadata {
		name: b"RelayToken".to_vec(),
		symbol: b"Relay".to_vec(),
		decimals: 12,
	};

	// Register relay asset in paraA
	ParaA::execute_with(|| {
		assert_ok!(AssetManager::register_asset(
			parachain::Origin::root(),
			source_location,
			asset_metadata,
			1u128,
		));
		// free execution
		assert_ok!(AssetManager::set_asset_units_per_second(
			parachain::Origin::root(),
			source_id,
			0u128
		));
	});

	let dest: MultiLocation = Junction::AccountKey20 {
		network: NetworkId::Any,
		key: PARAALICE,
	}
	.into();

	// First send relay chain asset to Parachain like in previous test
	Relay::execute_with(|| {
		assert_ok!(RelayChainPalletXcm::reserve_transfer_assets(
			relay_chain::Origin::signed(RELAYALICE),
			Box::new(Parachain(1).into().into()),
			Box::new(dest.clone().into()),
			Box::new((Here, 123).into()),
			0,
			123,
		));
	});

	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 123);
	});

	// lets gather the balance before sending back money
	let mut balance_before_sending = 0;
	Relay::execute_with(|| {
		balance_before_sending = RelayBalances::free_balance(&RELAYALICE);
	});

	// We now send back some money to the relay
	let dest = MultiLocation {
		parents: 1,
		interior: X1(AccountId32 {
			network: NetworkId::Any,
			id: RELAYALICE.into(),
		}),
	};

	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_ok!(XTokens::transfer(
			parachain::Origin::signed(PARAALICE.into()),
			parachain::CurrencyId::OtherReserve(source_id),
			123,
			Box::new(dest),
			40000
		));
	});

	// the balances in paraAlice should have been substracted
	ParaA::execute_with(|| {
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 0);
	});

	// balances in the relay should have been received
	Relay::execute_with(|| {
		// free execution,x	 full amount received
		assert!(RelayBalances::free_balance(&RELAYALICE) > balance_before_sending);
	});
}

#[test]
fn send_relay_asset_to_para_b() {
	MockNet::reset();

	let source_location = parachain::AssetType::Xcm(MultiLocation::parent());
	let source_id: parachain::AssetId = source_location.clone().into();

	let asset_metadata = parachain::AssetMetadata {
		name: b"RelayToken".to_vec(),
		symbol: b"Relay".to_vec(),
		decimals: 12,
	};

	// Register asset in paraA. Free execution
	ParaA::execute_with(|| {
		assert_ok!(AssetManager::register_asset(
			parachain::Origin::root(),
			source_location.clone(),
			asset_metadata.clone(),
			1u128,
		));
		assert_ok!(AssetManager::set_asset_units_per_second(
			parachain::Origin::root(),
			source_id,
			0u128
		));
	});

	// Register asset in paraB. Free execution
	ParaB::execute_with(|| {
		assert_ok!(AssetManager::register_asset(
			parachain::Origin::root(),
			source_location,
			asset_metadata,
			1u128,
		));
		assert_ok!(AssetManager::set_asset_units_per_second(
			parachain::Origin::root(),
			source_id,
			0u128
		));
	});

	// First send relay chain asset to Parachain A like in previous test
	let dest: MultiLocation = Junction::AccountKey20 {
		network: NetworkId::Any,
		key: PARAALICE,
	}
	.into();
	Relay::execute_with(|| {
		assert_ok!(RelayChainPalletXcm::reserve_transfer_assets(
			relay_chain::Origin::signed(RELAYALICE),
			Box::new(Parachain(1).into().into()),
			Box::new(dest.clone().into()),
			Box::new((Here, 123).into()),
			0,
			123,
		));
	});

	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 123);
	});

	// Now send relay asset from para A to para B
	let dest = MultiLocation {
		parents: 1,
		interior: X2(
			Parachain(2),
			AccountKey20 {
				network: NetworkId::Any,
				key: PARAALICE.into(),
			},
		),
	};

	ParaA::execute_with(|| {
		assert_ok!(XTokens::transfer(
			parachain::Origin::signed(PARAALICE.into()),
			parachain::CurrencyId::OtherReserve(source_id),
			100,
			Box::new(dest),
			40000
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

	// this represents the asset in paraA
	let para_a_balances = MultiLocation::new(1, X2(Parachain(1), PalletInstance(1u8)));
	let source_location = parachain::AssetType::Xcm(para_a_balances);
	let source_id: parachain::AssetId = source_location.clone().into();

	let asset_metadata = parachain::AssetMetadata {
		name: b"ParaAToken".to_vec(),
		symbol: b"ParaA".to_vec(),
		decimals: 18,
	};

	// Register asset in paraB. Free execution
	ParaB::execute_with(|| {
		assert_ok!(AssetManager::register_asset(
			parachain::Origin::root(),
			source_location,
			asset_metadata,
			1u128,
		));
		assert_ok!(AssetManager::set_asset_units_per_second(
			parachain::Origin::root(),
			source_id,
			0u128
		));
	});

	// Send para A asset from para A to para B
	let dest = MultiLocation {
		parents: 1,
		interior: X2(
			Parachain(2),
			AccountKey20 {
				network: NetworkId::Any,
				key: PARAALICE.into(),
			},
		),
	};

	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_ok!(XTokens::transfer(
			parachain::Origin::signed(PARAALICE.into()),
			parachain::CurrencyId::SelfReserve,
			100,
			Box::new(dest),
			800000
		));
	});

	// Native token is substracted in paraA
	ParaA::execute_with(|| {
		assert_eq!(
			ParaBalances::free_balance(&PARAALICE.into()),
			INITIAL_BALANCE - 100
		);
	});

	// Asset is minted in paraB
	ParaB::execute_with(|| {
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 100);
	});
}

#[test]
fn send_para_a_asset_from_para_b_to_para_c() {
	MockNet::reset();

	// Represents para A asset
	let para_a_balances = MultiLocation::new(1, X2(Parachain(1), PalletInstance(1u8)));
	let source_location = parachain::AssetType::Xcm(para_a_balances);
	let source_id: parachain::AssetId = source_location.clone().into();

	let asset_metadata = parachain::AssetMetadata {
		name: b"ParaAToken".to_vec(),
		symbol: b"ParaA".to_vec(),
		decimals: 18,
	};

	// Register para A asset in parachain B. Free execution
	ParaB::execute_with(|| {
		assert_ok!(AssetManager::register_asset(
			parachain::Origin::root(),
			source_location.clone(),
			asset_metadata.clone(),
			1u128,
		));
		assert_ok!(AssetManager::set_asset_units_per_second(
			parachain::Origin::root(),
			source_id,
			0u128
		));
	});

	// Register para A asset in parachain C. Free execution
	ParaC::execute_with(|| {
		assert_ok!(AssetManager::register_asset(
			parachain::Origin::root(),
			source_location,
			asset_metadata,
			1u128,
		));
		assert_ok!(AssetManager::set_asset_units_per_second(
			parachain::Origin::root(),
			source_id,
			0u128
		));
	});

	// Send para A asset to para B
	let dest = MultiLocation {
		parents: 1,
		interior: X2(
			Parachain(2),
			AccountKey20 {
				network: NetworkId::Any,
				key: PARAALICE.into(),
			},
		),
	};
	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_ok!(XTokens::transfer(
			parachain::Origin::signed(PARAALICE.into()),
			parachain::CurrencyId::SelfReserve,
			100,
			Box::new(dest),
			800000
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
	let dest = MultiLocation {
		parents: 1,
		interior: X2(
			Parachain(3),
			AccountKey20 {
				network: NetworkId::Any,
				key: PARAALICE.into(),
			},
		),
	};

	ParaB::execute_with(|| {
		// free execution, full amount received
		assert_ok!(XTokens::transfer(
			parachain::Origin::signed(PARAALICE.into()),
			parachain::CurrencyId::OtherReserve(source_id),
			100,
			Box::new(dest),
			800000
		));
	});

	// Para B balances have been substracted
	ParaB::execute_with(|| {
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 0);
	});

	// Para C balances have been credited
	ParaC::execute_with(|| {
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 100);
	});
}

#[test]
fn send_para_a_asset_to_para_b_and_back_to_para_a() {
	MockNet::reset();

	// para A asset
	let para_a_balances = MultiLocation::new(1, X2(Parachain(1), PalletInstance(1u8)));
	let source_location = parachain::AssetType::Xcm(para_a_balances);
	let source_id: parachain::AssetId = source_location.clone().into();

	let asset_metadata = parachain::AssetMetadata {
		name: b"ParaAToken".to_vec(),
		symbol: b"ParaA".to_vec(),
		decimals: 18,
	};

	// Register para A asset in para B
	ParaB::execute_with(|| {
		assert_ok!(AssetManager::register_asset(
			parachain::Origin::root(),
			source_location,
			asset_metadata,
			1u128,
		));
		assert_ok!(AssetManager::set_asset_units_per_second(
			parachain::Origin::root(),
			source_id,
			0u128
		));
	});

	// Send para A asset to para B
	let dest = MultiLocation {
		parents: 1,
		interior: X2(
			Parachain(2),
			AccountKey20 {
				network: NetworkId::Any,
				key: PARAALICE.into(),
			},
		),
	};
	ParaA::execute_with(|| {
		assert_ok!(XTokens::transfer(
			parachain::Origin::signed(PARAALICE.into()),
			parachain::CurrencyId::SelfReserve,
			100,
			Box::new(dest),
			4000
		));
	});

	// Balances have been substracted
	ParaA::execute_with(|| {
		assert_eq!(
			ParaBalances::free_balance(&PARAALICE.into()),
			INITIAL_BALANCE - 100
		);
	});

	// para B balances have been credited
	ParaB::execute_with(|| {
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 100);
	});

	// Send back para A asset to para A
	let dest = MultiLocation {
		parents: 1,
		interior: X2(
			Parachain(1),
			AccountKey20 {
				network: NetworkId::Any,
				key: PARAALICE.into(),
			},
		),
	};
	ParaB::execute_with(|| {
		assert_ok!(XTokens::transfer(
			parachain::Origin::signed(PARAALICE.into()),
			parachain::CurrencyId::OtherReserve(source_id),
			100,
			Box::new(dest),
			4000
		));
	});

	// Para B balances have been substracted
	ParaB::execute_with(|| {
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 0);
	});

	// Para A asset has been credited
	ParaA::execute_with(|| {
		assert_eq!(
			ParaBalances::free_balance(&PARAALICE.into()),
			INITIAL_BALANCE
		);
	});
}

#[test]
fn receive_relay_asset_with_trader() {
	MockNet::reset();

	let source_location = parachain::AssetType::Xcm(MultiLocation::parent());
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
		assert_ok!(AssetManager::register_asset(
			parachain::Origin::root(),
			source_location,
			asset_metadata,
			1u128,
		));
		assert_ok!(AssetManager::set_asset_units_per_second(
			parachain::Origin::root(),
			source_id,
			1_000_000u128
		));
	});

	let dest: MultiLocation = Junction::AccountKey20 {
		network: NetworkId::Any,
		key: PARAALICE,
	}
	.into();
	// We are sending 100 tokens from relay.
	// If we set the dest weight to be 1e7, we know the buy_execution will spend 1e7*1e6/1e12 = 10
	// Therefore with no refund, we should receive 10 tokens less
	// Native trader fails for this, and we use the asset trader
	Relay::execute_with(|| {
		assert_ok!(RelayChainPalletXcm::reserve_transfer_assets(
			relay_chain::Origin::signed(RELAYALICE),
			Box::new(Parachain(1).into().into()),
			Box::new(dest.clone().into()),
			Box::new((Here, 100).into()),
			0,
			10_000_000u64,
		));
	});

	ParaA::execute_with(|| {
		// non-free execution, not full amount received
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 90);
	});
}

#[test]
fn error_when_not_paying_enough() {
	MockNet::reset();

	let source_location = parachain::AssetType::Xcm(MultiLocation::parent());
	let source_id: parachain::AssetId = source_location.clone().into();

	let asset_metadata = parachain::AssetMetadata {
		name: b"RelayToken".to_vec(),
		symbol: b"Relay".to_vec(),
		decimals: 12,
	};

	let dest: MultiLocation = Junction::AccountKey20 {
		network: NetworkId::Any,
		key: PARAALICE,
	}
	.into();
	// This time we are gonna put a rather high number of units per second
	// we know later we will divide by 1e12
	// Lets put 1e6 as units per second
	ParaA::execute_with(|| {
		assert_ok!(AssetManager::register_asset(
			parachain::Origin::root(),
			source_location,
			asset_metadata,
			1u128,
		));
		assert_ok!(AssetManager::set_asset_units_per_second(
			parachain::Origin::root(),
			source_id,
			1_000_000u128
		));
	});

	// We are sending 100 tokens from relay.
	// If we set the dest weight to be 1e7, we know the buy_execution will spend 1e7*1e6/1e12 = 10
	// Therefore with no refund, we should receive 10 tokens less
	Relay::execute_with(|| {
		assert_ok!(RelayChainPalletXcm::reserve_transfer_assets(
			relay_chain::Origin::signed(RELAYALICE),
			Box::new(Parachain(1).into().into()),
			Box::new(dest.clone().into()),
			Box::new((Here, 5).into()),
			0,
			10_000_000u64,
		));
	});

	ParaA::execute_with(|| {
		// amount not received as it is not paying enough
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 0);
	});
}
