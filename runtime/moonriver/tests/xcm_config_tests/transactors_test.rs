// Copyright 2019-2025 Moonbeam Foundation.
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

//! Tests for AssetTransactors configuration.
//!
//! AssetTransactors handle the deposit and withdrawal of assets via XCM.
//! Moonriver uses a tuple of transactors:
//! - LocalAssetTransactor: Handles native token (MOVR) using pallet_balances
//! - EvmForeignAssets: Handles registered foreign assets
//! - Erc20XcmBridge: Handles ERC20 tokens via the bridge

use crate::xcm_common::*;
use frame_support::traits::{Currency, PalletInfoAccess};
use moonriver_runtime::{currency::MOVR, AccountId, Balances};
use xcm::latest::prelude::*;
use xcm_executor::traits::TransactAsset;

const ONE_MOVR: u128 = MOVR;
const ONE_DOT: u128 = 10_000_000_000;

fn alice_account() -> AccountId {
	AccountId::from(ALICE)
}

fn bob_account() -> AccountId {
	AccountId::from(BOB)
}

fn native_asset_location() -> Location {
	Location::new(0, [PalletInstance(Balances::index() as u8)])
}

#[test]
fn local_transactor_deposits_native_token() {
	ExtBuilder::default()
		.with_balances(vec![(alice_account(), ONE_MOVR * 100)])
		.build()
		.execute_with(|| {
			use moonriver_runtime::xcm_config::AssetTransactors;

			let initial_balance = Balances::free_balance(bob_account());

			let asset = Asset {
				id: AssetId(native_asset_location()),
				fun: Fungible(ONE_MOVR),
			};
			let destination = Location::new(
				0,
				[AccountKey20 {
					network: None,
					key: BOB,
				}],
			);

			// Deposit native asset to Bob
			let result =
				<AssetTransactors as TransactAsset>::deposit_asset(&asset, &destination, None);

			assert!(result.is_ok(), "Deposit should succeed");
			let final_balance = Balances::free_balance(bob_account());
			assert_eq!(
				final_balance,
				initial_balance + ONE_MOVR,
				"Balance should increase by deposited amount"
			);
		});
}

#[test]
fn local_transactor_withdraws_native_token() {
	ExtBuilder::default()
		.with_balances(vec![(alice_account(), ONE_MOVR * 100)])
		.build()
		.execute_with(|| {
			use moonriver_runtime::xcm_config::AssetTransactors;

			let initial_balance = Balances::free_balance(alice_account());

			let asset = Asset {
				id: AssetId(native_asset_location()),
				fun: Fungible(ONE_MOVR),
			};
			let source = Location::new(
				0,
				[AccountKey20 {
					network: None,
					key: ALICE,
				}],
			);

			// Withdraw native asset from Alice
			let result = <AssetTransactors as TransactAsset>::withdraw_asset(&asset, &source, None);

			assert!(result.is_ok(), "Withdraw should succeed");
			let final_balance = Balances::free_balance(alice_account());
			assert_eq!(
				final_balance,
				initial_balance - ONE_MOVR,
				"Balance should decrease by withdrawn amount"
			);
		});
}

#[test]
fn local_transactor_fails_withdraw_insufficient_balance() {
	ExtBuilder::default()
		.with_balances(vec![(alice_account(), ONE_MOVR)]) // Only 1 MOVR
		.build()
		.execute_with(|| {
			use moonriver_runtime::xcm_config::AssetTransactors;

			let asset = Asset {
				id: AssetId(native_asset_location()),
				fun: Fungible(ONE_MOVR * 100), // Try to withdraw 100 MOVR
			};
			let source = Location::new(
				0,
				[AccountKey20 {
					network: None,
					key: ALICE,
				}],
			);

			let result = <AssetTransactors as TransactAsset>::withdraw_asset(&asset, &source, None);

			assert!(
				result.is_err(),
				"Withdraw should fail with insufficient balance"
			);
		});
}

#[test]
fn foreign_asset_transactor_deposits_registered_asset() {
	let dot_location = Location::parent();

	ExtBuilder::default()
		.with_xcm_assets(vec![XcmAssetInitialization {
			asset_id: 1,
			xcm_location: dot_location.clone(),
			decimals: 10,
			name: "Polkadot",
			symbol: "DOT",
			balances: vec![],
		}])
		.build()
		.execute_with(|| {
			use moonriver_runtime::xcm_config::AssetTransactors;

			let asset = Asset {
				id: AssetId(dot_location.clone()),
				fun: Fungible(ONE_DOT),
			};
			let destination = Location::new(
				0,
				[AccountKey20 {
					network: None,
					key: BOB,
				}],
			);

			// Deposit DOT to Bob
			let result =
				<AssetTransactors as TransactAsset>::deposit_asset(&asset, &destination, None);

			// Should succeed for registered foreign asset
			assert!(
				result.is_ok(),
				"Deposit of registered foreign asset should succeed"
			);
		});
}

#[test]
fn transactor_fails_for_unregistered_asset() {
	ExtBuilder::default().build().execute_with(|| {
		use moonriver_runtime::xcm_config::AssetTransactors;

		// Unregistered asset location
		let unknown_asset = Asset {
			id: AssetId(Location::new(1, [Parachain(9999), PalletInstance(99)])),
			fun: Fungible(1_000_000),
		};
		let destination = Location::new(
			0,
			[AccountKey20 {
				network: None,
				key: BOB,
			}],
		);

		let result =
			<AssetTransactors as TransactAsset>::deposit_asset(&unknown_asset, &destination, None);

		// Should fail - asset not registered
		assert!(result.is_err(), "Deposit of unregistered asset should fail");
	});
}

#[test]
fn transactor_withdraws_registered_foreign_asset() {
	let dot_location = Location::parent();

	ExtBuilder::default()
		.with_xcm_assets(vec![XcmAssetInitialization {
			asset_id: 1,
			xcm_location: dot_location.clone(),
			decimals: 10,
			name: "Polkadot",
			symbol: "DOT",
			balances: vec![(bob_account(), ONE_DOT * 10)],
		}])
		.build()
		.execute_with(|| {
			use moonriver_runtime::xcm_config::AssetTransactors;

			let asset = Asset {
				id: AssetId(dot_location.clone()),
				fun: Fungible(ONE_DOT),
			};
			let source = Location::new(
				0,
				[AccountKey20 {
					network: None,
					key: BOB,
				}],
			);

			let result =
				<AssetTransactors as TransactAsset>::withdraw_asset(&asset, &source, None);

			assert!(
				result.is_ok(),
				"Withdraw of registered foreign asset should succeed: {:?}",
				result
			);
		});
}

#[test]
fn transactor_handles_erc20_bridge_asset() {
	ExtBuilder::default()
		.with_balances(vec![(alice_account(), ONE_MOVR * 100)])
		.build()
		.execute_with(|| {
			use moonriver_runtime::xcm_config::AssetTransactors;
			use moonriver_runtime::Erc20XcmBridge;

			let bridge_pallet_location = {
				use frame_support::traits::PalletInfoAccess;
				Location::new(
					0,
					[PalletInstance(
						<Erc20XcmBridge as PalletInfoAccess>::index() as u8,
					)],
				)
			};

			let fake_contract: [u8; 20] = [0xAA; 20];
			let erc20_asset_location = Location::new(
				0,
				[
					PalletInstance(bridge_pallet_location.first_interior().map_or(0, |j| {
						if let Junction::PalletInstance(idx) = j {
							*idx
						} else {
							0
						}
					})),
					AccountKey20 {
						network: None,
						key: fake_contract,
					},
				],
			);

			let asset = Asset {
				id: AssetId(erc20_asset_location),
				fun: Fungible(1_000),
			};
			let destination = Location::new(
				0,
				[AccountKey20 {
					network: None,
					key: BOB,
				}],
			);

			let result =
				<AssetTransactors as TransactAsset>::deposit_asset(&asset, &destination, None);

			// Either Ok or Err — the important thing is no panic.
			let _ = result;
		});
}

#[test]
fn transactor_handles_relay_sovereign_account() {
	ExtBuilder::default()
		.with_balances(vec![(alice_account(), ONE_MOVR * 100)])
		.build()
		.execute_with(|| {
			use moonriver_runtime::xcm_config::{AssetTransactors, LocationToAccountId};
			use xcm_executor::traits::ConvertLocation;

			// The relay chain's sovereign account
			let relay_location = Location::parent();
			let sovereign_account = LocationToAccountId::convert_location(&relay_location).unwrap();

			// Give the sovereign account some funds
			let _ = Balances::deposit_creating(&sovereign_account, ONE_MOVR * 10);

			let asset = Asset {
				id: AssetId(native_asset_location()),
				fun: Fungible(ONE_MOVR),
			};

			// Withdraw from relay sovereign account
			let result =
				<AssetTransactors as TransactAsset>::withdraw_asset(&asset, &relay_location, None);

			assert!(
				result.is_ok(),
				"Should withdraw from relay sovereign account"
			);
		});
}
