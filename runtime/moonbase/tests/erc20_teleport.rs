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
//! - **Wrong-path safety net.** `pallet_xcm::limited_reserve_transfer_assets` of a
//!   whitelisted ERC-20 to AH is rejected with `Filtered` (pallet-xcm classifies it
//!   as teleportable and refuses reserve-transfer); the same call to a non-AH
//!   destination errors at execution time and the storage layer rolls back, so no
//!   supply leaks into the `TeleportCheckingAccount`.
//! - **Whitelist lifecycle.** End-to-end exercise of the three-state state
//!   machine (`Registered → Active → Deregistered`) and the dual-purpose
//!   `remove_teleportable_erc20` (admin-only purge from `Registered`,
//!   permissionless purge from `Deregistered` once `LockedSupply == 0`,
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
			// then prove the permission matrix on `count == 0`. We pre-set the status
			// directly because auto-promotion fires from the asset transactor; a
			// pure-Substrate `remove` test isolates the lifecycle logic.
			//
			// First, `Active + count == 0` is admin-only. A live operational entry
			// transits through `count == 0` between in/out flows, so a third party
			// must NOT be able to snipe it the moment the counter hits zero.
			pallet_erc20_xcm_bridge::TeleportableErc20s::<Runtime>::insert(
				&contract,
				TeleportableErc20Status::Active,
			);
			pallet_erc20_xcm_bridge::LockedSupply::<Runtime>::insert(
				&contract,
				sp_core::U256::zero(),
			);
			assert_noop!(
				Erc20XcmBridge::remove_teleportable_erc20(signed.clone(), contract),
				DispatchError::BadOrigin,
			);
			assert_eq!(
				pallet_erc20_xcm_bridge::TeleportableErc20s::<Runtime>::get(&contract),
				Some(TeleportableErc20Status::Active),
				"signed user must not be able to purge an Active entry",
			);
			// Admin can.
			assert_ok!(Erc20XcmBridge::remove_teleportable_erc20(
				root_origin(),
				contract,
			));
			assert!(
				!pallet_erc20_xcm_bridge::TeleportableErc20s::<Runtime>::contains_key(&contract)
			);

			// Now the only permissionless case: `Deregistered + count == 0` — admin
			// already opted into wind-down by flipping the entry, and the obligation
			// is fully discharged, so the public sweep is allowed.
			pallet_erc20_xcm_bridge::TeleportableErc20s::<Runtime>::insert(
				&contract,
				TeleportableErc20Status::Deregistered,
			);
			pallet_erc20_xcm_bridge::LockedSupply::<Runtime>::insert(
				&contract,
				sp_core::U256::zero(),
			);
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

/// Whitelisting an ERC-20 changes XCM `TransactAsset` handling for that contract on
/// every path (teleport, reserve transfer, raw `WithdrawAsset`+`DepositAsset` programs,
/// etc.) — `Erc20TeleportTransactor` is placed before the legacy `Erc20XcmBridge`
/// adapter in `AssetTransactors`, so for any whitelisted contract it intercepts
/// `withdraw_asset` / `deposit_asset` regardless of which user-facing extrinsic
/// triggered the program.
///
/// On the user-facing extrinsic surface, the safety net for "wrong path" callers is
/// `pallet_xcm`'s reserve-transfer entry point: `do_reserve_transfer_assets` calls
/// `XcmAssetTransfers::determine_for(asset, dest)`, which returns `TransferType::Teleport`
/// the moment `IsTeleporter::contains(asset, dest)` is true. `do_reserve_transfer_assets`
/// then explicitly refuses with `Filtered` (see
/// `pallet_xcm/src/lib.rs`, "Ensure assets are not teleportable to `dest`").
///
/// This test pins that contract for whitelisted ERC-20s with `dest = AssetHub` so that
/// any future change to `IsTeleporter` (or to `pallet_xcm`'s classification logic) that
/// would let a reserve-transfer call slip through and end up in the teleport transactor
/// will be caught here. We also verify no storage was mutated (no auto-promotion to
/// `Active`, no `LockedSupply` increment), pinning the "rejected pre-execution" property.
#[test]
fn limited_reserve_transfer_assets_for_whitelisted_erc20_to_ah_returns_filtered() {
	use pallet_erc20_xcm_bridge::TeleportableErc20Status;

	ExtBuilder::default()
		.with_balances(vec![(AccountId::from(ALICE), 1_000 * UNIT)])
		.build()
		.execute_with(|| {
			let contract = H160([0xc1; 20]);
			assert_ok!(Erc20XcmBridge::add_teleportable_erc20(
				root_origin(),
				contract
			));
			// Sanity-check the starting state so the post-call assertions are meaningful.
			assert_eq!(
				pallet_erc20_xcm_bridge::TeleportableErc20s::<Runtime>::get(&contract),
				Some(TeleportableErc20Status::Registered),
			);
			assert!(pallet_erc20_xcm_bridge::LockedSupply::<Runtime>::get(&contract).is_zero());

			let asset: Asset = (erc20_location(contract), 1_000u128).into();
			let beneficiary = VersionedLocation::from(Location::new(
				0,
				[AccountKey20 {
					network: None,
					key: ALICE,
				}],
			));

			// `assert_noop!` covers both "errored with `Filtered`" AND "no storage
			// mutation" in a single check. Filtered comes from line the
			// upstream `pallet_xcm::do_reserve_transfer_assets`:
			// `ensure!(assets_transfer_type != TransferType::Teleport, Error::Filtered)`.
			assert_noop!(
				PolkadotXcm::limited_reserve_transfer_assets(
					origin_of(AccountId::from(ALICE)),
					Box::new(VersionedLocation::from(dest_assethub())),
					Box::new(beneficiary),
					Box::new(VersionedAssets::from(vec![asset])),
					0,
					xcm::v5::WeightLimit::Unlimited,
				),
				pallet_xcm::Error::<Runtime>::Filtered,
			);

			// Belt-and-braces: the contract was not promoted (the executor never ran)
			// and no supply was parked in the checking account.
			assert_eq!(
				pallet_erc20_xcm_bridge::TeleportableErc20s::<Runtime>::get(&contract),
				Some(TeleportableErc20Status::Registered),
				"Filtered call must not auto-promote Registered → Active",
			);
			assert!(
				pallet_erc20_xcm_bridge::LockedSupply::<Runtime>::get(&contract).is_zero(),
				"Filtered call must not lock supply in TeleportCheckingAccount",
			);
		});
}

/// Companion to the AH test above. For a whitelisted ERC-20 with a *non-AH*
/// destination, `pallet_xcm::do_reserve_transfer_assets` does NOT classify the asset
/// as `Teleport` (because `IsTeleporter` is bound to `AssetHubLocation`), so the
/// reserve-transfer path is admitted by the call gate. The local XCM program then runs
/// and goes through `Erc20TeleportTransactor::withdraw_asset` (because the contract
/// is whitelisted and the teleport transactor sits before the legacy adapter in
/// `AssetTransactors`). That leg either:
///
/// 1. Fails on the EVM transfer (no contract code / no balance on the user) →
///    `XcmError::FailedToTransactAsset` from inside the storage layer, OR
/// 2. Reaches `TransferReserveAsset` and fails to convert `Here` to `H160` →
///    `XcmError::AccountIdConversionFailed`.
///
/// Either way, the property we MUST preserve is "the storage layer rolls back" — no
/// supply gets stranded in `TeleportCheckingAccount` and no `LockedSupply` accounting
/// drift survives. This test pins that property irrespective of which of the two
/// failure modes the executor surfaces (they can drift between polkadot-sdk versions
/// and depending on test EVM state). We pre-seed `Active + count == 0` to make the
/// "did the lock leak through?" assertion as sharp as possible.
#[test]
fn limited_reserve_transfer_assets_for_whitelisted_erc20_to_non_ah_does_not_strand_funds() {
	use pallet_erc20_xcm_bridge::TeleportableErc20Status;
	use xcm::latest::prelude::Parachain;

	ExtBuilder::default()
		.with_balances(vec![(AccountId::from(ALICE), 1_000 * UNIT)])
		.build()
		.execute_with(|| {
			let contract = H160([0xc2; 20]);
			pallet_erc20_xcm_bridge::TeleportableErc20s::<Runtime>::insert(
				&contract,
				TeleportableErc20Status::Active,
			);
			pallet_erc20_xcm_bridge::LockedSupply::<Runtime>::insert(
				&contract,
				sp_core::U256::zero(),
			);

			let asset: Asset = (erc20_location(contract), 1_000u128).into();
			let hostile_sibling = VersionedLocation::from(Location::new(1, [Parachain(2042)]));
			let beneficiary = VersionedLocation::from(Location::new(
				0,
				[AccountKey20 {
					network: None,
					key: ALICE,
				}],
			));

			let result = PolkadotXcm::limited_reserve_transfer_assets(
				origin_of(AccountId::from(ALICE)),
				Box::new(hostile_sibling),
				Box::new(beneficiary),
				Box::new(VersionedAssets::from(vec![asset])),
				0,
				xcm::v5::WeightLimit::Unlimited,
			);

			// We don't pin the exact error variant — the failure surface depends on
			// the polkadot-sdk version (it can be `FailedToTransactAsset`,
			// `AccountIdConversionFailed`, `Unroutable`, etc.). The contract under
			// test is "no fund leakage", not "this exact variant".
			assert!(
				result.is_err(),
				"limited_reserve_transfer_assets to a non-AH destination must \
				 not succeed for a whitelisted ERC-20, got {:?}",
				result,
			);

			// THE invariant: nothing landed in the checking account, status untouched.
			assert_eq!(
				pallet_erc20_xcm_bridge::TeleportableErc20s::<Runtime>::get(&contract),
				Some(TeleportableErc20Status::Active),
				"failed reserve-transfer must not flip the contract state",
			);
			assert!(
				pallet_erc20_xcm_bridge::LockedSupply::<Runtime>::get(&contract).is_zero(),
				"failed reserve-transfer must not leak teleport-locked supply \
				 into the checking account",
			);
		});
}

/// End-to-end regression at the **real `AssetTransactors` tuple + executor** level for
/// the deregistered split-transactor drain (the audited bug).
///
/// The attack is a local `WithdrawAsset(deregistered erc20) + DepositAsset` program.
/// Before the fix, the tuple split it across two transactors:
///   - `WithdrawAsset` → `Erc20TeleportTransactor` returned `AssetNotHandled` for the
///     `Deregistered` contract, so the executor fell through to the legacy
///     `Erc20XcmBridge` adapter, which only *records* the (faux) origin — no EVM debit.
///   - `DepositAsset` → `Erc20TeleportTransactor` admits `Deregistered` inbound and pays
///     out of `Erc20TeleportCheckingAccount`, draining it for free.
///
/// We deploy an always-succeed ERC-20 stub at the contract address so that, on the
/// *pre-fix* code, the deposit leg's `ERC20.transfer(checking → beneficiary)` would
/// succeed and decrement `LockedSupply` (i.e. the drain completes). After the fix the
/// program must be rejected at the `WithdrawAsset` leg (the teleport transactor returns
/// a hard `FailedToTransactAsset` for `Deregistered`, which does NOT fall through to the
/// legacy adapter, and the legacy adapter is fenced off for any whitelisted contract).
/// The whole message rolls back: outcome is not `Complete`, `LockedSupply` is untouched,
/// and the contract is not promoted out of `Deregistered`.
///
/// This pins the tuple *ordering + routing* contract, not just the per-transactor
/// behavior covered by the pallet unit tests.
#[test]
fn deregistered_withdraw_deposit_program_cannot_drain_checking_account() {
	use fp_evm::GenesisAccount;
	use moonbase_runtime::xcm_config::XcmExecutor;
	use pallet_erc20_xcm_bridge::{LockedSupply, TeleportableErc20Status, TeleportableErc20s};
	use sp_core::U256;
	use std::collections::BTreeMap;
	use xcm::latest::{
		prelude::{All, DepositAsset, Wild, WithdrawAsset},
		Assets as XcmAssets, ExecuteXcm, Outcome, Weight, Xcm, XcmHash,
	};

	let contract = H160([0xdd; 20]);
	let drain_amount = 400u128;
	let seeded_locked_supply = 1_000u128;

	// Always-succeed ERC-20 stub: returns the 32-byte word `0x..01` for any calldata,
	// which `Pallet::erc20_transfer` accepts as `transfer(...) == true`.
	// Bytecode: PUSH1 1; PUSH1 0; MSTORE; PUSH1 32; PUSH1 0; RETURN.
	let stub_code = vec![0x60, 0x01, 0x60, 0x00, 0x52, 0x60, 0x20, 0x60, 0x00, 0xF3];
	let mut evm_accounts = BTreeMap::new();
	evm_accounts.insert(
		contract,
		GenesisAccount {
			nonce: Default::default(),
			balance: Default::default(),
			storage: Default::default(),
			code: stub_code,
		},
	);

	ExtBuilder::default()
		.with_balances(vec![(AccountId::from(ALICE), 1_000 * UNIT)])
		.with_evm_accounts(evm_accounts)
		.build()
		.execute_with(|| {
			// Wind-down state: contract has been deregistered while supply is still
			// parked in `Erc20TeleportCheckingAccount` (so inbound unwind stays open).
			TeleportableErc20s::<Runtime>::insert(&contract, TeleportableErc20Status::Deregistered);
			LockedSupply::<Runtime>::insert(&contract, U256::from(seeded_locked_supply));

			let asset: Asset = (erc20_location(contract), drain_amount).into();
			// Local attacker origin (a plain signed account on this chain).
			let attacker = Location::new(
				0,
				[AccountKey20 {
					network: None,
					key: CHARLIE,
				}],
			);

			// The bug-report program (fees omitted; we grant weight credit directly so
			// `TakeWeightCredit` admits the message without a `BuyExecution` leg).
			let message: Xcm<RuntimeCall> = Xcm(vec![
				WithdrawAsset(XcmAssets::from(vec![asset.clone()])),
				DepositAsset {
					assets: Wild(All),
					beneficiary: attacker.clone(),
				},
			]);

			let mut hash: XcmHash = [0u8; 32];
			let max_weight = Weight::from_parts(10_000_000_000, 10_000_000);
			let outcome = XcmExecutor::prepare_and_execute(
				attacker, message, &mut hash, max_weight, max_weight,
			);

			// The split routing is gone: the program must NOT complete. We don't pin the
			// exact error/index (it can drift with polkadot-sdk), only "did not complete".
			assert!(
				!matches!(outcome, Outcome::Complete { .. }),
				"WithdrawAsset+DepositAsset for a Deregistered ERC-20 must not complete, \
				 got {:?}",
				outcome,
			);

			// THE invariant: no supply left the checking account and no accounting drift.
			assert_eq!(
				LockedSupply::<Runtime>::get(&contract),
				U256::from(seeded_locked_supply),
				"checking-account obligation (LockedSupply) must be untouched",
			);
			// And the contract was not promoted out of `Deregistered`.
			assert_eq!(
				TeleportableErc20s::<Runtime>::get(&contract),
				Some(TeleportableErc20Status::Deregistered),
				"rejected program must not flip the contract lifecycle",
			);
		});
}
