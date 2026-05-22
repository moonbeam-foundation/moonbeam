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
//! - **Filter (`XcmTeleportFilter`).** Native DEV is rejected (regression guard
//!   against accidentally enabling native teleport); a non-whitelisted ERC-20 is
//!   rejected; a whitelisted ERC-20 passes both filter forms.
//! - **Location bind (`IsTeleporter`).** Whitelisted ERC-20s are admitted only
//!   from `TeleportTrustedLocation = AssetHubLocation`; sibling parachains, the
//!   relay, and `Here` are all rejected.
//! - **Outbound destination.** `pallet_xcm::limited_teleport_assets` rejects
//!   non-AH destinations with `Filtered` before any EVM state mutation.
//! - **Whitelist lifecycle.** End-to-end exercise of the three-state state
//!   machine (`Registered → Active → Deregistered`) and the dual-purpose
//!   `remove_teleportable_erc20` (admin-only purge from `Registered`,
//!   permissionless purge from `Active`/`Deregistered` once `LockedSupply == 0`,
//!   admin flip to `Deregistered` while `LockedSupply > 0`, revival via
//!   `add_teleportable_erc20`).
//! - **Admin escape hatch.** `force_remove_teleportable_erc20` purges any state
//!   regardless of `LockedSupply`.
//! - **Deregistered semantics.** A `Deregistered` contract still admits inbound
//!   teleports from AH (so users can recover supply still locked in the
//!   `TeleportCheckingAccount`) while every outbound gate refuses new locks.
//!

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

/// Integration coverage of the three-state lifecycle and the dual-purpose
/// `remove_teleportable_erc20` semantics. Pinning at the runtime layer because
/// signed-vs-root origin handling is runtime-defined (the `EnsureRoot` config of
/// `pallet_erc20_xcm_bridge`'s `TeleportAdminOrigin`).
#[test]
fn whitelist_admin_extrinsics_lifecycle() {
	ExtBuilder::default()
		.with_balances(vec![(AccountId::from(ALICE), 1_000 * UNIT)])
		.build()
		.execute_with(|| {
			use pallet_erc20_xcm_bridge::TeleportableErc20Status;

			let contract = H160([0xcd; 20]);
			let signed = origin_of(AccountId::from(ALICE));

			// `add` requires admin; signed user is rejected.
			assert_noop!(
				Erc20XcmBridge::add_teleportable_erc20(signed.clone(), contract),
				DispatchError::BadOrigin,
			);

			// `add` on an unknown contract: `(none) → Registered`. `LockedSupply` stays
			// at zero because no flow has happened yet.
			assert_ok!(Erc20XcmBridge::add_teleportable_erc20(
				root_origin(),
				contract
			));
			assert_eq!(
				pallet_erc20_xcm_bridge::TeleportableErc20s::<Runtime>::get(&contract),
				Some(TeleportableErc20Status::Registered),
			);
			assert!(pallet_erc20_xcm_bridge::LockedSupply::<Runtime>::get(&contract).is_zero());

			// Calling `add` again on `Registered` is the duplicate-no-op case.
			assert_noop!(
				Erc20XcmBridge::add_teleportable_erc20(root_origin(), contract),
				pallet_erc20_xcm_bridge::Error::<Runtime>::Erc20AlreadyTeleportable,
			);

			// `remove` on `Registered + count == 0` is admin-only and PURGES the entry
			// (no `Deregistered` intermediate state because there's no obligation).
			// Signed user gets `BadOrigin`; admin sweeps it cleanly.
			assert_noop!(
				Erc20XcmBridge::remove_teleportable_erc20(signed.clone(), contract),
				DispatchError::BadOrigin,
			);
			assert_ok!(Erc20XcmBridge::remove_teleportable_erc20(
				root_origin(),
				contract
			));
			assert!(
				!pallet_erc20_xcm_bridge::TeleportableErc20s::<Runtime>::contains_key(&contract),
				"Registered + zero count must purge cleanly",
			);

			// Subsequent `remove` returns `NotTeleportable`.
			assert_noop!(
				Erc20XcmBridge::remove_teleportable_erc20(root_origin(), contract),
				pallet_erc20_xcm_bridge::Error::<Runtime>::Erc20NotTeleportable,
			);

			// Re-add (fresh, no leftover state) lands at `Registered` again.
			assert_ok!(Erc20XcmBridge::add_teleportable_erc20(
				root_origin(),
				contract
			));

			// Simulate an outstanding obligation: pre-set `Active + count > 0` so the
			// other `remove` branch fires. (The promotion + counter increment is
			// covered end-to-end by the pallet unit tests; here we only need the
			// state machine integrated against the runtime's `EnsureRoot` origin.)
			pallet_erc20_xcm_bridge::TeleportableErc20s::<Runtime>::insert(
				&contract,
				TeleportableErc20Status::Active,
			);
			pallet_erc20_xcm_bridge::LockedSupply::<Runtime>::insert(
				&contract,
				sp_core::U256::from(500u128),
			);

			// `remove` on `Active + count > 0` is admin-only and FLIPS to `Deregistered`,
			// preserving the counter so users can keep teleporting their twin home.
			assert_noop!(
				Erc20XcmBridge::remove_teleportable_erc20(signed.clone(), contract),
				DispatchError::BadOrigin,
			);
			assert_ok!(Erc20XcmBridge::remove_teleportable_erc20(
				root_origin(),
				contract
			));
			assert_eq!(
				pallet_erc20_xcm_bridge::TeleportableErc20s::<Runtime>::get(&contract),
				Some(TeleportableErc20Status::Deregistered),
			);
			assert_eq!(
				pallet_erc20_xcm_bridge::LockedSupply::<Runtime>::get(&contract),
				sp_core::U256::from(500u128),
			);

			// Calling `remove` again while still `Deregistered + count > 0` yields the
			// no-op error.
			assert_noop!(
				Erc20XcmBridge::remove_teleportable_erc20(root_origin(), contract),
				pallet_erc20_xcm_bridge::Error::<Runtime>::Erc20AlreadyRemoved,
			);

			// Revive: `Deregistered → Registered` via `add`. The counter survives the
			// round-trip so subsequent inbound legs unwind it normally.
			assert_ok!(Erc20XcmBridge::add_teleportable_erc20(
				root_origin(),
				contract
			));
			assert_eq!(
				pallet_erc20_xcm_bridge::TeleportableErc20s::<Runtime>::get(&contract),
				Some(TeleportableErc20Status::Registered),
			);
			assert_eq!(
				pallet_erc20_xcm_bridge::LockedSupply::<Runtime>::get(&contract),
				sp_core::U256::from(500u128),
			);

			// Drain the counter (simulating the inbound teleport-back unwinding it),
			// then prove the permissionless purge from `Active`. We pre-set `Active`
			// because the actual auto-promotion fires from the asset transactor; a
			// pure-Substrate `remove` test isolates the lifecycle logic.
			pallet_erc20_xcm_bridge::TeleportableErc20s::<Runtime>::insert(
				&contract,
				TeleportableErc20Status::Active,
			);
			pallet_erc20_xcm_bridge::LockedSupply::<Runtime>::insert(
				&contract,
				sp_core::U256::zero(),
			);

			// Now `Active + count == 0` is the permissionless purge case — any signed
			// user (including ALICE, not admin) can sweep the entry.
			assert_ok!(Erc20XcmBridge::remove_teleportable_erc20(signed, contract));
			assert!(
				!pallet_erc20_xcm_bridge::TeleportableErc20s::<Runtime>::contains_key(&contract)
			);
			assert!(!pallet_erc20_xcm_bridge::LockedSupply::<Runtime>::contains_key(&contract));
		});
}

/// Cover the admin escape hatch: `force_remove_teleportable_erc20` purges any state
/// regardless of `LockedSupply`, and is admin-only at the runtime layer too.
#[test]
fn force_remove_teleportable_erc20_admin_escape_hatch() {
	ExtBuilder::default()
		.with_balances(vec![(AccountId::from(ALICE), 1_000 * UNIT)])
		.build()
		.execute_with(|| {
			use pallet_erc20_xcm_bridge::TeleportableErc20Status;

			let contract = H160([0xfe; 20]);

			// Pre-seed an outstanding obligation: `Deregistered + count > 0` (the
			// scenario where regular `remove` would refuse with `Erc20AlreadyRemoved`).
			pallet_erc20_xcm_bridge::TeleportableErc20s::<Runtime>::insert(
				&contract,
				TeleportableErc20Status::Deregistered,
			);
			pallet_erc20_xcm_bridge::LockedSupply::<Runtime>::insert(
				&contract,
				sp_core::U256::from(1_000u128),
			);

			// Signed user can't force-remove.
			assert_noop!(
				Erc20XcmBridge::force_remove_teleportable_erc20(
					origin_of(AccountId::from(ALICE)),
					contract,
				),
				DispatchError::BadOrigin,
			);
			// Storage untouched.
			assert!(
				pallet_erc20_xcm_bridge::TeleportableErc20s::<Runtime>::contains_key(&contract)
			);

			// Admin can. Both maps are removed.
			assert_ok!(Erc20XcmBridge::force_remove_teleportable_erc20(
				root_origin(),
				contract
			));
			assert!(
				!pallet_erc20_xcm_bridge::TeleportableErc20s::<Runtime>::contains_key(&contract),
			);
			assert!(!pallet_erc20_xcm_bridge::LockedSupply::<Runtime>::contains_key(&contract));

			// Force-remove on an unknown contract errors.
			assert_noop!(
				Erc20XcmBridge::force_remove_teleportable_erc20(root_origin(), contract),
				pallet_erc20_xcm_bridge::Error::<Runtime>::Erc20NotTeleportable,
			);
		});
}

/// Pinning the core invariant of the lifecycle: after `remove_teleportable_erc20`
/// while `LockedSupply > 0`, the whitelist entry is **kept** in `Deregistered` state
/// so the inbound teleport-back path that returns supply from
/// `Erc20TeleportCheckingAccount` to users stays open indefinitely, while new outbound
/// teleports are refused immediately by both runtime filter forms.
#[test]
fn deregistered_contract_keeps_inbound_open_and_blocks_outbound_via_runtime_filters() {
	ExtBuilder::default()
		.with_balances(vec![(AccountId::from(ALICE), 1_000 * UNIT)])
		.build()
		.execute_with(|| {
			use pallet_erc20_xcm_bridge::TeleportableErc20Status;

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

			// Pre-seed `Deregistered + count > 0`. (The full lifecycle that gets us
			// here — add → flow → admin remove — is exercised in
			// `whitelist_admin_extrinsics_lifecycle` and the pallet unit tests.)
			pallet_erc20_xcm_bridge::TeleportableErc20s::<Runtime>::insert(
				&contract,
				TeleportableErc20Status::Deregistered,
			);
			pallet_erc20_xcm_bridge::LockedSupply::<Runtime>::insert(
				&contract,
				sp_core::U256::from(1_000u128),
			);

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
			// a `Deregistered` contract returns `Filtered` cleanly, before any EVM
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
