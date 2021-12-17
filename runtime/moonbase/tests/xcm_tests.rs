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

mod xcm_mock;
use frame_support::{assert_ok, traits::PalletInfo};
use xcm::{VersionedMultiLocation, WrapVersion};
use xcm_mock::parachain;
use xcm_mock::relay_chain;
use xcm_mock::*;
use xcm_primitives::UtilityEncodeCall;

use xcm::latest::prelude::QueryResponse;
use xcm::latest::{
	Junction::{self, AccountId32, AccountKey20, PalletInstance, Parachain},
	Junctions::*,
	MultiLocation, NetworkId, Response, Xcm,
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
			Box::new(VersionedMultiLocation::V1(dest).clone().into()),
			Box::new((Here, 123).into()),
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

	let source_location = parachain::AssetType::Xcm(MultiLocation::parent());
	let source_id: parachain::AssetId = source_location.clone().into();

	let asset_metadata = parachain::AssetMetadata {
		name: b"RelayToken".to_vec(),
		symbol: b"Relay".to_vec(),
		decimals: 12,
	};

	// First send relay chain asset to Parachain like in previous test
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

	let dest: MultiLocation = Junction::AccountKey20 {
		network: NetworkId::Any,
		key: PARAALICE,
	}
	.into();
	Relay::execute_with(|| {
		assert_ok!(RelayChainPalletXcm::reserve_transfer_assets(
			relay_chain::Origin::signed(RELAYALICE),
			Box::new(Parachain(1).into().into()),
			Box::new(VersionedMultiLocation::V1(dest).clone().into()),
			Box::new((Here, 123).into()),
			0,
		));
	});

	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 123);
	});

	let mut balance_before_sending = 0;
	Relay::execute_with(|| {
		balance_before_sending = RelayBalances::free_balance(&RELAYALICE);
	});

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
			Box::new(VersionedMultiLocation::V1(dest)),
			40000
		));
	});

	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 0);
	});

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

	let dest: MultiLocation = Junction::AccountKey20 {
		network: NetworkId::Any,
		key: PARAALICE,
	}
	.into();
	Relay::execute_with(|| {
		assert_ok!(RelayChainPalletXcm::reserve_transfer_assets(
			relay_chain::Origin::signed(RELAYALICE),
			Box::new(Parachain(1).into().into()),
			Box::new(VersionedMultiLocation::V1(dest).clone().into()),
			Box::new((Here, 123).into()),
			0,
		));
	});

	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 123);
	});

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
			parachain::CurrencyId::OtherReserve(source_id),
			100,
			Box::new(VersionedMultiLocation::V1(dest)),
			40000
		));
	});

	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 23);
	});

	ParaB::execute_with(|| {
		// free execution, full amount received
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 100);
	});
}

#[test]
fn send_para_a_asset_to_para_b() {
	MockNet::reset();

	let para_a_balances = MultiLocation::new(1, X2(Parachain(1), PalletInstance(1u8)));
	let source_location = parachain::AssetType::Xcm(para_a_balances);
	let source_id: parachain::AssetId = source_location.clone().into();

	let asset_metadata = parachain::AssetMetadata {
		name: b"ParaAToken".to_vec(),
		symbol: b"ParaA".to_vec(),
		decimals: 18,
	};

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
			Box::new(VersionedMultiLocation::V1(dest)),
			800000
		));
	});
	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_eq!(
			ParaBalances::free_balance(&PARAALICE.into()),
			INITIAL_BALANCE - 100
		);
	});

	ParaB::execute_with(|| {
		// free execution, full amount received
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 100);
	});
}

#[test]
fn send_para_a_asset_from_para_b_to_para_c() {
	MockNet::reset();

	let para_a_balances = MultiLocation::new(1, X2(Parachain(1), PalletInstance(1u8)));
	let source_location = parachain::AssetType::Xcm(para_a_balances);
	let source_id: parachain::AssetId = source_location.clone().into();

	let asset_metadata = parachain::AssetMetadata {
		name: b"ParaAToken".to_vec(),
		symbol: b"ParaA".to_vec(),
		decimals: 18,
	};

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
			Box::new(VersionedMultiLocation::V1(dest)),
			80
		));
	});

	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_eq!(
			ParaBalances::free_balance(&PARAALICE.into()),
			INITIAL_BALANCE - 100
		);
	});

	ParaB::execute_with(|| {
		// free execution, full amount received
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 100);
	});

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
			Box::new(VersionedMultiLocation::V1(dest)),
			80
		));
	});

	// The message passed through parachainA so we needed to pay since its the native token
	ParaC::execute_with(|| {
		// free execution, full amount received
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 96);
	});
}

#[test]
fn send_para_a_asset_to_para_b_and_back_to_para_a() {
	MockNet::reset();

	let para_a_balances = MultiLocation::new(1, X2(Parachain(1), PalletInstance(1u8)));
	let source_location = parachain::AssetType::Xcm(para_a_balances);
	let source_id: parachain::AssetId = source_location.clone().into();

	let asset_metadata = parachain::AssetMetadata {
		name: b"ParaAToken".to_vec(),
		symbol: b"ParaA".to_vec(),
		decimals: 18,
	};

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
			Box::new(VersionedMultiLocation::V1(dest)),
			80
		));
	});

	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_eq!(
			ParaBalances::free_balance(&PARAALICE.into()),
			INITIAL_BALANCE - 100
		);
	});

	ParaB::execute_with(|| {
		// free execution, full amount received
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 100);
	});

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
		// free execution, full amount received
		assert_ok!(XTokens::transfer(
			parachain::Origin::signed(PARAALICE.into()),
			parachain::CurrencyId::OtherReserve(source_id),
			100,
			Box::new(VersionedMultiLocation::V1(dest)),
			80
		));
	});

	ParaA::execute_with(|| {
		// free execution, full amount received
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
			2500000000000u128
		));
	});

	let dest: MultiLocation = Junction::AccountKey20 {
		network: NetworkId::Any,
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
			relay_chain::Origin::signed(RELAYALICE),
			Box::new(Parachain(1).into().into()),
			Box::new(VersionedMultiLocation::V1(dest).clone().into()),
			Box::new((Here, 100).into()),
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

	let para_a_balances = MultiLocation::new(1, X2(Parachain(1), PalletInstance(1u8)));
	let source_location = parachain::AssetType::Xcm(para_a_balances);
	let source_id: parachain::AssetId = source_location.clone().into();

	let asset_metadata = parachain::AssetMetadata {
		name: b"ParaAToken".to_vec(),
		symbol: b"ParaA".to_vec(),
		decimals: 18,
	};

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
			2500000000000u128
		));
	});

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

	// In destination chain, we only need 4 weight
	// We put 10 weight, 6 of which should be refunded and 4 of which should go to treasury
	ParaA::execute_with(|| {
		assert_ok!(XTokens::transfer(
			parachain::Origin::signed(PARAALICE.into()),
			parachain::CurrencyId::SelfReserve,
			100,
			Box::new(VersionedMultiLocation::V1(dest)),
			10
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

	let para_a_balances = MultiLocation::new(1, X2(Parachain(1), PalletInstance(1u8)));
	let source_location = parachain::AssetType::Xcm(para_a_balances);
	let source_id: parachain::AssetId = source_location.clone().into();

	let asset_metadata = parachain::AssetMetadata {
		name: b"ParaAToken".to_vec(),
		symbol: b"ParaA".to_vec(),
		decimals: 18,
	};

	ParaB::execute_with(|| {
		assert_ok!(AssetManager::register_asset(
			parachain::Origin::root(),
			source_location,
			asset_metadata,
			1u128,
		));
		// With these units per second, 80K weight convrets to 1 asset unit
		assert_ok!(AssetManager::set_asset_units_per_second(
			parachain::Origin::root(),
			source_id,
			12500000u128
		));
	});

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

	// we use transfer_with_fee
	ParaA::execute_with(|| {
		assert_ok!(XTokens::transfer_with_fee(
			parachain::Origin::signed(PARAALICE.into()),
			parachain::CurrencyId::SelfReserve,
			100,
			1,
			Box::new(VersionedMultiLocation::V1(dest)),
			800000
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
			2500000000000u128
		));
	});

	// We are sending 100 tokens from relay.
	// If we set the dest weight to be 1e7, we know the buy_execution will spend 1e7*1e6/1e12 = 10
	// Therefore with no refund, we should receive 10 tokens less
	Relay::execute_with(|| {
		assert_ok!(RelayChainPalletXcm::reserve_transfer_assets(
			relay_chain::Origin::signed(RELAYALICE),
			Box::new(Parachain(1).into().into()),
			Box::new(VersionedMultiLocation::V1(dest).clone().into()),
			Box::new((Here, 5).into()),
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

	let source_location = parachain::AssetType::Xcm(MultiLocation::parent());
	let source_id: parachain::AssetId = source_location.clone().into();

	let asset_metadata = parachain::AssetMetadata {
		name: b"RelayToken".to_vec(),
		symbol: b"Relay".to_vec(),
		decimals: 12,
	};

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
			1u128
		));

		// Root can set transact info
		assert_ok!(XcmTransactor::set_transact_info(
			parachain::Origin::root(),
			xcm::VersionedMultiLocation::V1(MultiLocation::parent()),
			// Relay charges 1000 for every instruction, and we have 3, so 3000
			3000,
			0,
			0,
			1,
			0
		));
	});

	// Let's construct the call to know how much weight it is going to require

	let dest: MultiLocation = AccountKey20 {
		network: NetworkId::Any,
		key: PARAALICE,
	}
	.into();
	Relay::execute_with(|| {
		// 4000000000 transact + 3000 correspond to 4000003000 tokens. 100 more for the transfer call
		assert_ok!(RelayChainPalletXcm::reserve_transfer_assets(
			relay_chain::Origin::signed(RELAYALICE),
			Box::new(Parachain(1).into().into()),
			Box::new(VersionedMultiLocation::V1(dest).clone().into()),
			Box::new((Here, 4000003100).into()),
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
			parachain::Origin::root(),
			PARAALICE.into(),
			0,
		));
	});

	// Send to registered address

	let registered_address = derivative_account_id(para_a_account(), 0);
	let dest = MultiLocation {
		parents: 1,
		interior: X1(AccountId32 {
			network: NetworkId::Any,
			id: registered_address.clone().into(),
		}),
	};

	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_ok!(XTokens::transfer(
			parachain::Origin::signed(PARAALICE.into()),
			parachain::CurrencyId::OtherReserve(source_id),
			100,
			Box::new(VersionedMultiLocation::V1(dest)),
			40000
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
	let mut call_bytes = pallet_balances::Call::<relay_chain::Runtime>::transfer {
		dest: para_a_account(),
		value: 100u32.into(),
	}
	.encode();
	encoded.append(&mut call_bytes);

	ParaA::execute_with(|| {
		assert_ok!(XcmTransactor::transact_through_derivative_multilocation(
			parachain::Origin::signed(PARAALICE.into()),
			parachain::MockTransactors::Relay,
			0,
			xcm::VersionedMultiLocation::V1(MultiLocation::parent()),
			// 4000000000 + 3000 we should have taken out 4000003000 tokens from the caller
			4000000000,
			encoded,
		));
		let event_found: Option<parachain::Event> =
			parachain::para_events()
				.iter()
				.find_map(|event| match event.clone() {
					parachain::Event::PolkadotXcm(pallet_xcm::Event::AssetsTrapped(_, _, _)) => {
						Some(event.clone())
					}
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
fn transact_through_sovereign() {
	MockNet::reset();

	let source_location = parachain::AssetType::Xcm(MultiLocation::parent());
	let source_id: parachain::AssetId = source_location.clone().into();

	let asset_metadata = parachain::AssetMetadata {
		name: b"RelayToken".to_vec(),
		symbol: b"Relay".to_vec(),
		decimals: 12,
	};

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
			1u128
		));

		// Root can set transact info
		assert_ok!(XcmTransactor::set_transact_info(
			parachain::Origin::root(),
			xcm::VersionedMultiLocation::V1(MultiLocation::parent()),
			3000,
			0,
			0,
			1,
			0
		));
	});

	let dest: MultiLocation = AccountKey20 {
		network: NetworkId::Any,
		key: PARAALICE,
	}
	.into();
	Relay::execute_with(|| {
		assert_ok!(RelayChainPalletXcm::reserve_transfer_assets(
			relay_chain::Origin::signed(RELAYALICE),
			Box::new(Parachain(1).into().into()),
			Box::new(VersionedMultiLocation::V1(dest).clone().into()),
			Box::new((Here, 4000003100).into()),
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
			parachain::Origin::root(),
			PARAALICE.into(),
			0,
		));
	});

	// Send to registered address
	let registered_address = derivative_account_id(para_a_account(), 0);
	let dest = MultiLocation {
		parents: 1,
		interior: X1(AccountId32 {
			network: NetworkId::Any,
			id: registered_address.clone().into(),
		}),
	};

	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_ok!(XTokens::transfer(
			parachain::Origin::signed(PARAALICE.into()),
			parachain::CurrencyId::OtherReserve(source_id),
			100,
			Box::new(VersionedMultiLocation::V1(dest)),
			40000
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
	let dest = MultiLocation {
		parents: 1,
		interior: Here,
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
	let mut call_bytes = pallet_balances::Call::<relay_chain::Runtime>::transfer {
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
			parachain::Origin::root(),
			xcm::VersionedMultiLocation::V1(dest),
			PARAALICE.into(),
			xcm::VersionedMultiLocation::V1(MultiLocation::parent()),
			4000000000,
			utility_bytes,
		));
	});

	Relay::execute_with(|| {
		// free execution,x	 full amount received
		assert!(RelayBalances::free_balance(&para_a_account()) == 100);

		assert!(RelayBalances::free_balance(&registered_address) == 0);
	});
}

#[test]
fn test_automatic_versioning_on_runtime_upgrade_with_relay() {
	MockNet::reset();

	let source_location = parachain::AssetType::Xcm(MultiLocation::parent());
	let source_id: parachain::AssetId = source_location.clone().into();
	let asset_metadata = parachain::AssetMetadata {
		name: b"RelayToken".to_vec(),
		symbol: b"Relay".to_vec(),
		decimals: 12,
	};
	// register relay asset in parachain A and set XCM version to 1
	ParaA::execute_with(|| {
		parachain::XcmVersioner::set_version(1);
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

	let response = Response::Version(2);

	// This is irrelevant, nothing will be done with this message,
	// but we need to pass a message as an argument to trigger the storage change
	let mock_message: Xcm<()> = Xcm(vec![QueryResponse {
		query_id: 0,
		response,
		max_weight: 0,
	}]);
	// The router is mocked, and we cannot use WrapVersion in ChildParachainRouter. So we will force
	// it directly here
	// Actually send relay asset to parachain
	let dest: MultiLocation = AccountKey20 {
		network: NetworkId::Any,
		key: PARAALICE,
	}
	.into();

	Relay::execute_with(|| {
		// This sets the default version, for not known destinations
		assert_ok!(RelayChainPalletXcm::force_default_xcm_version(
			relay_chain::Origin::root(),
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
			relay_chain::Origin::signed(RELAYALICE),
			Box::new(Parachain(1).into().into()),
			Box::new(VersionedMultiLocation::V1(dest).clone().into()),
			Box::new((Here, 123).into()),
			0,
		));

		// Let's advance the relay. This should trigger the subscription message
		relay_chain::relay_roll_to(2);

		// queries should have been updated
		assert!(RelayChainPalletXcm::query(0).is_some());
	});

	let expected_supported_version: relay_chain::Event =
		pallet_xcm::Event::SupportedVersionChanged(
			MultiLocation {
				parents: 0,
				interior: X1(Parachain(1)),
			},
			1,
		)
		.into();

	Relay::execute_with(|| {
		// Assert that the events vector contains the version change
		assert!(relay_chain::relay_events().contains(&expected_supported_version));
	});

	let expected_version_notified: parachain::Event = pallet_xcm::Event::VersionChangeNotified(
		MultiLocation {
			parents: 1,
			interior: Here,
		},
		2,
	)
	.into();

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
		assert!(parachain::para_events().contains(&expected_version_notified));
	});

	// This event should have been seen in the relay
	let expected_supported_version_2: relay_chain::Event =
		pallet_xcm::Event::SupportedVersionChanged(
			MultiLocation {
				parents: 0,
				interior: X1(Parachain(1)),
			},
			2,
		)
		.into();

	Relay::execute_with(|| {
		// Assert that the events vector contains the new version change
		assert!(relay_chain::relay_events().contains(&expected_supported_version_2));
	});
}

#[test]
fn test_automatic_versioning_on_runtime_upgrade_with_para_b() {
	MockNet::reset();

	let para_a_balances = MultiLocation::new(1, X2(Parachain(1), PalletInstance(1u8)));
	let source_location = parachain::AssetType::Xcm(para_a_balances);
	let source_id: parachain::AssetId = source_location.clone().into();

	let asset_metadata = parachain::AssetMetadata {
		name: b"ParaAToken".to_vec(),
		symbol: b"ParaA".to_vec(),
		decimals: 18,
	};
	let response = Response::Version(2);

	// This is irrelevant, nothing will be done with this message,
	// but we need to pass a message as an argument to trigger the storage change
	let mock_message: Xcm<()> = Xcm(vec![QueryResponse {
		query_id: 0,
		response,
		max_weight: 0,
	}]);

	ParaA::execute_with(|| {
		// advertised version
		parachain::XcmVersioner::set_version(2);
	});

	ParaB::execute_with(|| {
		// Let's try with v0
		parachain::XcmVersioner::set_version(0);

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

	ParaA::execute_with(|| {
		// This sets the default version, for not known destinations
		assert_ok!(ParachainPalletXcm::force_default_xcm_version(
			parachain::Origin::root(),
			Some(2)
		));
		// Wrap version, which sets VersionedStorage
		assert_ok!(<ParachainPalletXcm as WrapVersion>::wrap_version(
			&MultiLocation::new(1, X1(Parachain(2))).into(),
			mock_message
		));

		parachain::para_roll_to(2);

		// queries should have been updated
		assert!(ParachainPalletXcm::query(0).is_some());
	});

	let expected_supported_version: parachain::Event = pallet_xcm::Event::SupportedVersionChanged(
		MultiLocation {
			parents: 1,
			interior: X1(Parachain(2)),
		},
		0,
	)
	.into();

	ParaA::execute_with(|| {
		// Assert that the events vector contains the version change
		assert!(parachain::para_events().contains(&expected_supported_version));
	});

	// Let's ensure talking in v0 works
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
			Box::new(VersionedMultiLocation::V1(dest)),
			80
		));
		// free execution, full amount received
		assert_eq!(
			ParaBalances::free_balance(&PARAALICE.into()),
			INITIAL_BALANCE - 100
		);
	});

	ParaB::execute_with(|| {
		// free execution, full amount received
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 100);
	});

	let expected_version_notified: parachain::Event = pallet_xcm::Event::VersionChangeNotified(
		MultiLocation {
			parents: 1,
			interior: X1(Parachain(1)),
		},
		2,
	)
	.into();

	// ParaB changes version to 2, and calls on_runtime_upgrade. This should notify the targets
	// of the new version change
	ParaB::execute_with(|| {
		// Set version
		parachain::XcmVersioner::set_version(2);
		// Do runtime upgrade
		parachain::on_runtime_upgrade();
		// Initialize block, to call on_initialize and notify targets
		parachain::para_roll_to(2);
		// Expect the event in the parachain
		assert!(parachain::para_events().contains(&expected_version_notified));
	});

	// This event should have been seen in para A
	let expected_supported_version_2: parachain::Event =
		pallet_xcm::Event::SupportedVersionChanged(
			MultiLocation {
				parents: 1,
				interior: X1(Parachain(2)),
			},
			2,
		)
		.into();

	// Para A should have received the version change
	ParaA::execute_with(|| {
		// Assert that the events vector contains the new version change
		assert!(parachain::para_events().contains(&expected_supported_version_2));
	});
}

use parity_scale_codec::{Decode, Encode};
use sp_io::hashing::blake2_256;

// Helper to derive accountIds
pub fn derivative_account_id(who: sp_runtime::AccountId32, index: u16) -> sp_runtime::AccountId32 {
	let entropy = (b"modlpy/utilisuba", who, index).using_encoded(blake2_256);
	sp_runtime::AccountId32::decode(&mut &entropy[..]).unwrap_or_default()
}
