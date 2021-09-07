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
use xcm_mock::parachain;
use xcm_mock::relay_chain;
use xcm_mock::*;

use frame_support::assert_ok;

use xcm::v0::{
	Junction::{self, PalletInstance, Parachain, Parent},
	MultiAsset::*,
	MultiLocation::*,
	NetworkId,
};
use xcm_simulator::MultiLocation;
use xcm_simulator::TestExt;

#[test]
fn receive_relay_asset_from_relay() {
	MockNet::reset();

	let source_location = parachain::AssetType::Xcm(X1(Junction::Parent));
	let source_id: parachain::AssetId = source_location.clone().into();
	ParaA::execute_with(|| {
		assert_ok!(AssetManager::xcm_asset_register(
			parachain::Origin::root(),
			source_location,
			1u128,
			0u128
		));
	});

	Relay::execute_with(|| {
		assert_ok!(RelayChainPalletXcm::reserve_transfer_assets(
			relay_chain::Origin::signed(RELAYALICE),
			X1(Parachain(1)),
			X1(Junction::AccountKey20 {
				network: NetworkId::Any,
				key: PARAALICE
			}),
			vec![ConcreteFungible {
				id: MultiLocation::Here,
				amount: 123
			}],
			123,
		));
	});

	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 123);
	});
}

#[test]
fn send_relay_asset_to_relay() {
	MockNet::reset();

	let source_location = parachain::AssetType::Xcm(X1(Junction::Parent));
	let source_id: parachain::AssetId = source_location.clone().into();
	ParaA::execute_with(|| {
		assert_ok!(AssetManager::xcm_asset_register(
			parachain::Origin::root(),
			source_location,
			1u128,
			0u128
		));
	});

	Relay::execute_with(|| {
		assert_ok!(RelayChainPalletXcm::reserve_transfer_assets(
			relay_chain::Origin::signed(RELAYALICE),
			X1(Parachain(1)),
			X1(Junction::AccountKey20 {
				network: NetworkId::Any,
				key: PARAALICE
			}),
			vec![ConcreteFungible {
				id: Null,
				amount: 123
			}],
			123,
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

	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_ok!(XTokens::transfer(
			parachain::Origin::signed(PARAALICE.into()),
			parachain::CurrencyId::OtherReserve(source_id),
			100,
			X2(
				Junction::Parent,
				Junction::AccountId32 {
					network: NetworkId::Any,
					id: RELAYALICE.into()
				}
			),
			4000
		));
	});

	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 23);
	});

	Relay::execute_with(|| {
		// free execution,x	 full amount received
		assert!(RelayBalances::free_balance(&RELAYALICE) > balance_before_sending);
	});
}

#[test]
fn send_relay_asset_to_para_b() {
	MockNet::reset();

	let source_location = parachain::AssetType::Xcm(X1(Junction::Parent));
	let source_id: parachain::AssetId = source_location.clone().into();
	ParaA::execute_with(|| {
		assert_ok!(AssetManager::xcm_asset_register(
			parachain::Origin::root(),
			source_location.clone(),
			1u128,
			1u128
		));
	});

	ParaB::execute_with(|| {
		assert_ok!(AssetManager::xcm_asset_register(
			parachain::Origin::root(),
			source_location,
			1u128,
			0u128
		));
	});

	Relay::execute_with(|| {
		assert_ok!(RelayChainPalletXcm::reserve_transfer_assets(
			relay_chain::Origin::signed(RELAYALICE),
			X1(Parachain(1)),
			X1(Junction::AccountKey20 {
				network: NetworkId::Any,
				key: PARAALICE
			}),
			vec![ConcreteFungible {
				id: Null,
				amount: 123
			}],
			123,
		));
	});

	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 123);
	});

	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_ok!(XTokens::transfer(
			parachain::Origin::signed(PARAALICE.into()),
			parachain::CurrencyId::OtherReserve(source_id),
			100,
			X3(
				Junction::Parent,
				Junction::Parachain(2),
				Junction::AccountKey20 {
					network: NetworkId::Any,
					key: PARAALICE.into()
				}
			),
			4000
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

	let para_a_balances = X3(Parent, Parachain(1).into(), PalletInstance(1u8));
	let source_location = parachain::AssetType::Xcm(para_a_balances);
	let source_id: parachain::AssetId = source_location.clone().into();

	ParaB::execute_with(|| {
		assert_ok!(AssetManager::xcm_asset_register(
			parachain::Origin::root(),
			source_location,
			1u128,
			0u128
		));
	});

	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_ok!(XTokens::transfer(
			parachain::Origin::signed(PARAALICE.into()),
			parachain::CurrencyId::SelfReserve,
			100,
			X3(
				Junction::Parent,
				Junction::Parachain(2),
				Junction::AccountKey20 {
					network: NetworkId::Any,
					key: PARAALICE.into()
				}
			),
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
fn send_para_a_asset_to_para_b_and_back_to_para_a() {
	MockNet::reset();

	let para_a_balances = X3(Parent, Parachain(1).into(), PalletInstance(1u8));
	let source_location = parachain::AssetType::Xcm(para_a_balances);
	let source_id: parachain::AssetId = source_location.clone().into();

	ParaB::execute_with(|| {
		assert_ok!(AssetManager::xcm_asset_register(
			parachain::Origin::root(),
			source_location,
			1u128,
			0u128
		));
	});

	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_ok!(XTokens::transfer(
			parachain::Origin::signed(PARAALICE.into()),
			parachain::CurrencyId::SelfReserve,
			100,
			X3(
				Junction::Parent,
				Junction::Parachain(2),
				Junction::AccountKey20 {
					network: NetworkId::Any,
					key: PARAALICE.into()
				}
			),
			4000
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

	ParaB::execute_with(|| {
		// free execution, full amount received
		assert_ok!(XTokens::transfer(
			parachain::Origin::signed(PARAALICE.into()),
			parachain::CurrencyId::OtherReserve(source_id),
			100,
			X3(
				Junction::Parent,
				Junction::Parachain(1),
				Junction::AccountKey20 {
					network: NetworkId::Any,
					key: PARAALICE.into()
				}
			),
			4000
		));
	});

	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_eq!(
			ParaBalances::free_balance(&PARAALICE.into()),
			INITIAL_BALANCE
		);
	});
}

#[test]
fn receive_relay_asset_with_trader() {
	MockNet::reset();

	let source_location = parachain::AssetType::Xcm(X1(Junction::Parent));
	let source_id: parachain::AssetId = source_location.clone().into();

	// This time we are gonna put a rather high number of units per second
	// we know later we will divide by 1e12
	// Lets put 1e6 as units per second
	ParaA::execute_with(|| {
		assert_ok!(AssetManager::xcm_asset_register(
			parachain::Origin::root(),
			source_location,
			1u128,
			1_000_000u128
		));
	});

	// We are sending 100 tokens from relay.
	// If we set the dest weight to be 1e7, we know the buy_execution will spend 1e7*1e6/1e12 = 10
	// Therefore with no refund, we should receive 10 tokens less
	Relay::execute_with(|| {
		assert_ok!(RelayChainPalletXcm::reserve_transfer_assets(
			relay_chain::Origin::signed(RELAYALICE),
			X1(Parachain(1)),
			X1(Junction::AccountKey20 {
				network: NetworkId::Any,
				key: PARAALICE
			}),
			vec![ConcreteFungible {
				id: Null,
				amount: 100
			}],
			10_000_000u64,
		));
	});

	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 90);
	});
}
