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

//! Integration tests for the ERC-20 teleport feature exposed by
//! `pallet_erc20_xcm_bridge`.
//!
//! These tests exercise the actual `moonbase-runtime` (not the simulator mock) and verify
//! that:
//!
//! 1. `pallet_xcm::limited_teleport_assets` is rejected by `XcmTeleportFilter` when carrying
//!    native DEV (regression guard against accidentally enabling native teleport).
//! 2. `pallet_xcm::limited_teleport_assets` is rejected when carrying a non-whitelisted
//!    ERC-20 location.
//! 3. Root can whitelist a contract via `pallet_erc20_xcm_bridge::add_teleportable_erc20`,
//!    and after that the `IsTeleportableErc20` filter accepts it (so the same call would no
//!    longer be filtered by `XcmTeleportFilter`).
//!
//! Actual cross-chain delivery and EVM checking-account accounting are exercised by the
//! zombienet smoke test under `tools/zombienet/`.

mod common;
use common::*;

use frame_support::{
	assert_noop, assert_ok,
	traits::{Contains, ContainsPair},
};
use moonbase_runtime::{
	xcm_config::{Erc20XcmBridgePalletLocation, SelfReserve},
	Erc20XcmBridge, PolkadotXcm,
};
use sp_core::H160;
use sp_runtime::DispatchError;
use xcm::{
	latest::prelude::{AccountKey20, Asset, Junction, Location, PalletInstance},
	VersionedAssets, VersionedLocation,
};

/// Build a relative ERC-20 location using the runtime's pallet-instance prefix.
fn erc20_location(contract: H160) -> Location {
	let prefix = Erc20XcmBridgePalletLocation::get();
	let pallet_instance = match prefix.interior().first() {
		Some(Junction::PalletInstance(idx)) => *idx,
		_ => panic!("Erc20XcmBridgePalletLocation must start with PalletInstance"),
	};
	Location {
		parents: 0,
		interior: [
			PalletInstance(pallet_instance),
			AccountKey20 {
				key: contract.0,
				network: None,
			},
		]
		.into(),
	}
}

fn dest_assethub() -> Location {
	moonbase_runtime::xcm_config::AssetHubLocation::get()
}

#[test]
fn teleport_filter_rejects_native_dev() {
	ExtBuilder::default()
		.with_balances(vec![(AccountId::from(ALICE), 1_000 * UNIT)])
		.build()
		.execute_with(|| {
			let dev_asset: Asset = (SelfReserve::get(), 10u128 * UNIT).into();

			// `XcmTeleportFilter = IsTeleportableErc20<Runtime>` rejects DEV at the call gate.
			assert_noop!(
				PolkadotXcm::limited_teleport_assets(
					origin_of(AccountId::from(ALICE)),
					Box::new(VersionedLocation::from(dest_assethub())),
					Box::new(VersionedLocation::from(Location::new(
						0,
						[AccountKey20 {
							network: None,
							key: ALICE,
						}],
					))),
					Box::new(VersionedAssets::from(vec![dev_asset])),
					0,
					xcm::v5::WeightLimit::Unlimited,
				),
				pallet_xcm::Error::<Runtime>::Filtered,
			);
		});
}

#[test]
fn teleport_filter_rejects_non_whitelisted_erc20() {
	ExtBuilder::default()
		.with_balances(vec![(AccountId::from(ALICE), 1_000 * UNIT)])
		.build()
		.execute_with(|| {
			let contract = H160([0xab; 20]);
			let asset: Asset = (erc20_location(contract), 1_000u128).into();

			assert_noop!(
				PolkadotXcm::limited_teleport_assets(
					origin_of(AccountId::from(ALICE)),
					Box::new(VersionedLocation::from(dest_assethub())),
					Box::new(VersionedLocation::from(Location::new(
						0,
						[AccountKey20 {
							network: None,
							key: ALICE,
						}],
					))),
					Box::new(VersionedAssets::from(vec![asset])),
					0,
					xcm::v5::WeightLimit::Unlimited,
				),
				pallet_xcm::Error::<Runtime>::Filtered,
			);
		});
}

#[test]
fn whitelist_admin_extrinsics_are_root_only() {
	ExtBuilder::default()
		.with_balances(vec![(AccountId::from(ALICE), 1_000 * UNIT)])
		.build()
		.execute_with(|| {
			let contract = H160([0xcd; 20]);

			// Non-root signed origin is rejected.
			assert_noop!(
				Erc20XcmBridge::add_teleportable_erc20(origin_of(AccountId::from(ALICE)), contract,),
				DispatchError::BadOrigin,
			);

			// Root works and is idempotent-safe (duplicate add fails).
			assert_ok!(Erc20XcmBridge::add_teleportable_erc20(
				root_origin(),
				contract
			));
			assert_noop!(
				Erc20XcmBridge::add_teleportable_erc20(root_origin(), contract),
				pallet_erc20_xcm_bridge::Error::<Runtime>::Erc20AlreadyTeleportable,
			);

			// Removing back to clean state works.
			assert_ok!(Erc20XcmBridge::remove_teleportable_erc20(
				root_origin(),
				contract
			));
			assert_noop!(
				Erc20XcmBridge::remove_teleportable_erc20(root_origin(), contract),
				pallet_erc20_xcm_bridge::Error::<Runtime>::Erc20NotTeleportable,
			);
		});
}

#[test]
fn whitelisted_erc20_passes_teleport_filters() {
	ExtBuilder::default()
		.with_balances(vec![(AccountId::from(ALICE), 1_000 * UNIT)])
		.build()
		.execute_with(|| {
			let contract = H160([0xef; 20]);
			let asset_loc = erc20_location(contract);
			let asset: Asset = (asset_loc.clone(), 1_000u128).into();
			let dest = dest_assethub();
			let origin_loc = Location::new(
				0,
				[AccountKey20 {
					network: None,
					key: ALICE,
				}],
			);

			type Filter = pallet_erc20_xcm_bridge::IsTeleportableErc20<Runtime>;

			// Before whitelisting: both filter forms reject.
			assert!(!<Filter as ContainsPair<Asset, Location>>::contains(
				&asset, &dest,
			));
			assert!(!<Filter as Contains<(Location, Vec<Asset>)>>::contains(&(
				origin_loc.clone(),
				vec![asset.clone()]
			)));

			assert_ok!(Erc20XcmBridge::add_teleportable_erc20(
				root_origin(),
				contract
			));

			// After whitelisting: both filter forms accept.
			assert!(<Filter as ContainsPair<Asset, Location>>::contains(
				&asset, &dest,
			));
			assert!(<Filter as Contains<(Location, Vec<Asset>)>>::contains(&(
				origin_loc,
				vec![asset]
			)));
		});
}
