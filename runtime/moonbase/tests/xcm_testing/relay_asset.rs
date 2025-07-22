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
	xcm_testing::{currency_to_asset, helpers::*},
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
	reset_test_environment();

	// Register relay asset using helper
	let relay_asset_id = register_relay_asset();

	// Send relay asset to parachain (keep original transfer logic for now)
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

	// Verify that parachain received the asset using helper
	assert_asset_balance(&PARAALICE, relay_asset_id, 123);
}

// Send relay asset (like DOT) back from Parachain A to relaychain
#[test]
fn send_relay_asset_to_relay() {
	reset_test_environment();

	// Register relay asset using helper
	let relay_asset_id = register_relay_asset();

	// First send relay chain asset to Parachain
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
	assert_asset_balance(&PARAALICE, relay_asset_id, 123);

	// Get initial relay balance
	let balance_before_sending = {
		let mut balance = 0;
		Relay::execute_with(|| {
			balance = RelayBalances::free_balance(&RELAYALICE);
		});
		balance
	};

	// Send relay asset back to relay using builder for custom params
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
		let asset = currency_to_asset(parachain::CurrencyId::ForeignAsset(relay_asset_id), 123);
		assert_ok!(PolkadotXcm::transfer_assets(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			Box::new(VersionedLocation::from(chain_part)),
			Box::new(VersionedLocation::from(beneficiary)),
			Box::new(VersionedAssets::from(asset)),
			0,
			WeightLimit::Limited(Weight::from_parts(40000u64, DEFAULT_PROOF_SIZE))
		));
	});

	// Verify balances
	assert_asset_balance(&PARAALICE, relay_asset_id, 0);
	Relay::execute_with(|| {
		assert!(RelayBalances::free_balance(&RELAYALICE) > balance_before_sending);
	});
}

#[test]
fn send_relay_asset_to_para_b() {
	reset_test_environment();

	// Register relay asset in both parachains using helpers
	let relay_asset_id = register_relay_asset();
	register_relay_asset_in_para_b();

	// Send relay asset to Para A first
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
	assert_asset_balance(&PARAALICE, relay_asset_id, 123);

	// Send relay asset from Para A to Para B
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
		let asset = currency_to_asset(parachain::CurrencyId::ForeignAsset(relay_asset_id), 100);
		assert_ok!(PolkadotXcm::transfer_assets(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			Box::new(VersionedLocation::from(chain_part)),
			Box::new(VersionedLocation::from(beneficiary)),
			Box::new(VersionedAssets::from(vec![asset])),
			0,
			WeightLimit::Limited(Weight::from_parts(40000u64, DEFAULT_PROOF_SIZE))
		));
	});

	// Verify balances
	assert_asset_balance(&PARAALICE, relay_asset_id, 23);
	assert_asset_balance_para_b(&PARAALICE, relay_asset_id, 100);
}

#[test]
fn receive_relay_asset_with_trader() {
	reset_test_environment();

	// Use helper for high units_per_second registration
	let relay_asset_id = register_relay_asset_with_units_per_second(2_500_000_000_000);

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

	// Use assertion
	assert_asset_balance(&PARAALICE, relay_asset_id, 90);
	assert_treasury_asset_balance(relay_asset_id, 10);
}

#[test]
fn error_when_not_paying_enough() {
	reset_test_environment();

	let relay_asset_id = register_relay_asset_with_units_per_second(2500000000000);

	let dest: Location = Junction::AccountKey20 {
		network: None,
		key: PARAALICE,
	}
	.into();

	// Initial state verification - should be zero
	assert_asset_balance(&PARAALICE, relay_asset_id, 0);

	// We are sending 5 tokens from relay - not enough to pay fees
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

	// Use helper for assertion - amount not received as it is not paying enough
	assert_asset_balance(&PARAALICE, relay_asset_id, 0);
}

#[test]
fn receive_asset_with_no_sufficients_not_possible_if_non_existent_account() {
	reset_test_environment();

	let fresh_account = [2u8; 20];
	let relay_asset_id = register_relay_asset_non_sufficient();

	// Actually send relay asset to parachain - should fail for non-existent account
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

	// parachain should not have received assets (non-sufficient asset to non-existent account)
	assert_asset_balance(&fresh_account, relay_asset_id, 0);

	// Fund fresh account with native tokens using helper
	fund_account_native(&fresh_account, 100);

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

	// parachain should have received assets (account now exists)
	assert_asset_balance(&fresh_account, relay_asset_id, 123);
}

#[test]
fn receive_assets_with_sufficients_true_allows_non_funded_account_to_receive_assets() {
	reset_test_environment();

	let fresh_account = [2u8; 20];
	let relay_asset_id = register_relay_asset(); // Uses sufficient=true by default

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

	// parachain should have received assets (sufficient asset allows non-funded account)
	assert_asset_balance(&fresh_account, relay_asset_id, 123);
}
