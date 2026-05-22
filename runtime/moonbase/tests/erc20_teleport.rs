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

			// Both extrinsics reject signed origins.
			assert_noop!(
				Erc20XcmBridge::add_teleportable_erc20(origin_of(AccountId::from(ALICE)), contract,),
				DispatchError::BadOrigin,
			);
			assert_noop!(
				Erc20XcmBridge::remove_teleportable_erc20(
					origin_of(AccountId::from(ALICE)),
					contract,
				),
				DispatchError::BadOrigin,
			);

			// Root adds → contract is Active. Adding again while `Active` is the
			// duplicate no-op case.
			assert_ok!(Erc20XcmBridge::add_teleportable_erc20(
				root_origin(),
				contract
			));
			assert_noop!(
				Erc20XcmBridge::add_teleportable_erc20(root_origin(), contract),
				pallet_erc20_xcm_bridge::Error::<Runtime>::Erc20AlreadyTeleportable,
			);

			// Active → InboundOnly via a single `remove_teleportable_erc20`. The entry
			// stays in storage; it just flips state so the inbound (return) path keeps
			// admitting users that still hold a foreign-asset twin on Asset Hub.
			assert_ok!(Erc20XcmBridge::remove_teleportable_erc20(
				root_origin(),
				contract
			));

			// Removing again is rejected: `InboundOnly` is the terminal `remove` state.
			assert_noop!(
				Erc20XcmBridge::remove_teleportable_erc20(root_origin(), contract),
				pallet_erc20_xcm_bridge::Error::<Runtime>::Erc20AlreadyRemoved,
			);

			// But `add_teleportable_erc20` revives an `InboundOnly` entry back to
			// `Active`. This is the operator escape hatch from a removal that should be
			// reversed.
			assert_ok!(Erc20XcmBridge::add_teleportable_erc20(
				root_origin(),
				contract
			));

			// And once `Active` again, calling `add` is a no-op as before.
			assert_noop!(
				Erc20XcmBridge::add_teleportable_erc20(root_origin(), contract),
				pallet_erc20_xcm_bridge::Error::<Runtime>::Erc20AlreadyTeleportable,
			);
		});
}

/// Pinning the core invariant of the simplified lifecycle: after
/// `remove_teleportable_erc20`, the whitelist entry is **kept** in `InboundOnly` state
/// so the inbound teleport-back path that returns supply from
/// `Erc20TeleportCheckingAccount` to users stays open indefinitely, while new outbound
/// teleports are refused immediately by both runtime filter forms.
#[test]
fn inbound_only_contract_keeps_inbound_open_and_blocks_outbound_via_runtime_filters() {
	ExtBuilder::default()
		.with_balances(vec![(AccountId::from(ALICE), 1_000 * UNIT)])
		.build()
		.execute_with(|| {
			let contract = H160([0xdd; 20]);
			let asset: Asset = (erc20_location(contract), 1_000u128).into();
			let ah = dest_assethub();
			let origin_loc = Location::new(
				0,
				[AccountKey20 {
					network: None,
					key: ALICE,
				}],
			);
			type Filter = pallet_erc20_xcm_bridge::IsTeleportableErc20<Runtime>;

			assert_ok!(Erc20XcmBridge::add_teleportable_erc20(
				root_origin(),
				contract
			));
			// One-step transition: Active → InboundOnly.
			assert_ok!(Erc20XcmBridge::remove_teleportable_erc20(
				root_origin(),
				contract
			));

			// Inbound trust gate (`IsTeleporter`): still accepts AH-originated teleports
			// for this contract so `Erc20TeleportCheckingAccount` can keep returning
			// supply to users — this is the whole point of preserving the whitelist
			// entry after removal.
			assert!(<Filter as ContainsPair<Asset, Location>>::contains(
				&asset, &ah,
			));

			// User-facing outbound gate (`XcmTeleportFilter`): rejects fresh outbound
			// teleports for this contract.
			assert!(!<Filter as Contains<(Location, Vec<Asset>)>>::contains(&(
				origin_loc.clone(),
				vec![asset.clone()],
			)));

			// And the actual user extrinsic is filtered: `limited_teleport_assets` for
			// an `InboundOnly` contract returns `Filtered` cleanly, before any EVM
			// state is touched.
			assert_noop!(
				PolkadotXcm::limited_teleport_assets(
					origin_of(AccountId::from(ALICE)),
					Box::new(VersionedLocation::from(ah.clone())),
					Box::new(VersionedLocation::from(origin_loc)),
					Box::new(VersionedAssets::from(vec![asset])),
					0,
					xcm::v5::WeightLimit::Unlimited,
				),
				pallet_xcm::Error::<Runtime>::Filtered,
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

/// Regression test for the `IsTeleporter` security gate.
///
/// `pallet_xcm` and `xcm-executor` both consult `IsTeleporter::contains(asset, location)` —
/// inbound on `ReceiveTeleportedAsset` (location = message origin) and outbound on
/// `pallet_xcm::limited_teleport_assets` (location = destination). If the filter accepted a
/// whitelisted ERC-20 from any location, any sibling parachain or the relay could deliver a
/// `ReceiveTeleportedAsset` followed by `DepositAsset` and drain `Erc20TeleportCheckingAccount`.
///
/// We bind the filter to `TeleportTrustedLocation = AssetHubLocation` in
/// `pallet_erc20_xcm_bridge::Config`, so this test enumerates a few non-AH locations and
/// asserts every one is rejected even after the contract is whitelisted.
#[test]
fn whitelisted_erc20_rejected_from_untrusted_locations() {
	use xcm::latest::prelude::Parachain;

	ExtBuilder::default()
		.with_balances(vec![(AccountId::from(ALICE), 1_000 * UNIT)])
		.build()
		.execute_with(|| {
			let contract = H160([0xa1; 20]);
			let asset: Asset = (erc20_location(contract), 1_000u128).into();
			type Filter = pallet_erc20_xcm_bridge::IsTeleportableErc20<Runtime>;

			assert_ok!(Erc20XcmBridge::add_teleportable_erc20(
				root_origin(),
				contract
			));

			// AH (the trusted peer) is admitted — sanity check.
			assert!(<Filter as ContainsPair<Asset, Location>>::contains(
				&asset,
				&dest_assethub(),
			));

			// Hostile sibling parachains: rejected.
			let hostile_sibling = Location::new(1, [Parachain(2042)]);
			assert!(!<Filter as ContainsPair<Asset, Location>>::contains(
				&asset,
				&hostile_sibling,
			));

			// Relay chain: rejected.
			assert!(!<Filter as ContainsPair<Asset, Location>>::contains(
				&asset,
				&Location::parent(),
			));

			// `Here`: rejected (would otherwise allow self-origin loops).
			assert!(!<Filter as ContainsPair<Asset, Location>>::contains(
				&asset,
				&Location::here(),
			));
		});
}

/// Regression test: outbound teleports with a non-AH destination must error before any
/// EVM state mutation, even when the contract is whitelisted.
///
/// `pallet_xcm::XcmTeleportFilter` (the outer gate at the user-facing extrinsic) cannot
/// see `dest` — its signature is `Contains<(Location, Vec<Asset>)>` where `Location` is
/// the local caller. The destination check therefore lives in the next gate down,
/// `IsTeleporter::contains(asset, dest)`, which `pallet_xcm::teleport_assets_program`
/// runs *before* building the local `WithdrawAsset` instruction. As long as that gate is
/// bound to `TeleportTrustedLocation = AssetHubLocation`, the user's ERC-20s cannot end
/// up locked in `Erc20TeleportCheckingAccount` for any destination other than AH — the
/// call fails with `Filtered` and never reaches the EVM. This test pins that contract.
#[test]
fn limited_teleport_assets_rejects_non_ah_destination() {
	use xcm::latest::prelude::Parachain;

	ExtBuilder::default()
		.with_balances(vec![(AccountId::from(ALICE), 1_000 * UNIT)])
		.build()
		.execute_with(|| {
			let contract = H160([0xb2; 20]);
			assert_ok!(Erc20XcmBridge::add_teleportable_erc20(
				root_origin(),
				contract
			));

			let beneficiary = VersionedLocation::from(Location::new(
				0,
				[AccountKey20 {
					network: None,
					key: ALICE,
				}],
			));

			// A hostile sibling parachain must be rejected even though the contract is
			// whitelisted. `assert_noop!` also asserts no storage was mutated.
			let asset: Asset = (erc20_location(contract), 1_000u128).into();
			assert_noop!(
				PolkadotXcm::limited_teleport_assets(
					origin_of(AccountId::from(ALICE)),
					Box::new(VersionedLocation::from(Location::new(1, [Parachain(2042)]))),
					Box::new(beneficiary.clone()),
					Box::new(VersionedAssets::from(vec![asset])),
					0,
					xcm::v5::WeightLimit::Unlimited,
				),
				pallet_xcm::Error::<Runtime>::Filtered,
			);

			// Same for the relay chain — only AH is trusted to teleport whitelisted
			// ERC-20s in/out of this runtime.
			let asset: Asset = (erc20_location(contract), 1_000u128).into();
			assert_noop!(
				PolkadotXcm::limited_teleport_assets(
					origin_of(AccountId::from(ALICE)),
					Box::new(VersionedLocation::from(Location::parent())),
					Box::new(beneficiary),
					Box::new(VersionedAssets::from(vec![asset])),
					0,
					xcm::v5::WeightLimit::Unlimited,
				),
				pallet_xcm::Error::<Runtime>::Filtered,
			);
		});
}
