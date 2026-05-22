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

//! Unit testing
use frame_support::traits::{Contains, ContainsPair};
use frame_support::{assert_noop, assert_ok};
use sp_core::{H160, U256};
use sp_io::TestExternalities;
use sp_runtime::{AccountId32, BoundedVec, BuildStorage};
use xcm::latest::prelude::XcmError;
use xcm::latest::{Asset, AssetId, Fungibility, Junction, Junctions, Location, XcmContext};
use xcm_executor::traits::TransactAsset;

use crate::mock::{
	Erc20XcmBridge, Erc20XcmBridgeTransferGasLimit, RuntimeEvent, RuntimeOrigin, System, Test,
};
use crate::{
	Erc20TeleportTransactor, Event, IsTeleportableErc20, LockedSupply, TeleportableErc20Status,
	TeleportableErc20s,
};

fn new_test_ext() -> TestExternalities {
	let storage = frame_system::GenesisConfig::<Test>::default()
		.build_storage()
		.expect("genesis storage builds");
	let mut ext = TestExternalities::new(storage);
	ext.execute_with(|| System::set_block_number(1));
	ext
}

fn erc20_asset(contract: [u8; 20], amount: u128) -> Asset {
	let location = Location {
		parents: 0,
		interior: Junctions::from([
			Junction::PalletInstance(42u8),
			Junction::AccountKey20 {
				key: contract,
				network: None,
			},
		]),
	};
	Asset::from((location, amount))
}

/// Constructs a zero-amount ERC-20 `Asset` directly, bypassing the typed
/// `Fungibility::from(u128)` constructor whose `debug_assert_ne!(amount, 0)`
/// would panic in debug builds. Adversaries can still deliver such assets to
/// the transactor via SCALE-decoded XCM messages (the assertion doesn't run on
/// the deserialization path), which is the precise threat the fast path
/// defends against, so we need to test that case directly.
fn erc20_asset_zero(contract: [u8; 20]) -> Asset {
	let location = Location {
		parents: 0,
		interior: Junctions::from([
			Junction::PalletInstance(42u8),
			Junction::AccountKey20 {
				key: contract,
				network: None,
			},
		]),
	};
	Asset {
		id: AssetId(location),
		fun: Fungibility::Fungible(0),
	}
}

/// Trusted counterparty location used by the mock (matches `mock::TeleportTrustedLocation`).
fn trusted_loc() -> Location {
	Location::new(1, [Junction::Parachain(1001)])
}

fn user_loc(addr: [u8; 20]) -> Location {
	Location {
		parents: 0,
		interior: Junctions::from([Junction::AccountKey20 {
			network: None,
			key: addr,
		}]),
	}
}

fn xcm_ctx() -> XcmContext {
	XcmContext {
		origin: None,
		message_id: [0u8; 32],
		topic: None,
	}
}

/// Some signed AccountId for the permissionless-purge tests.
fn signed_origin() -> RuntimeOrigin {
	RuntimeOrigin::from(frame_system::RawOrigin::Signed(AccountId32::from(
		[7u8; 32],
	)))
}

/// Deploy a minimal "always-succeed" ERC-20 stub at `contract`. Whatever calldata is
/// thrown at it, the runtime returns `0x000…001` (32 bytes), which `Pallet::erc20_transfer`
/// accepts as `transfer(...) == true`. This lets unit tests exercise
/// `Erc20TeleportTransactor::{withdraw_asset, deposit_asset, internal_transfer_asset}`
/// without deploying real OZ-style ERC-20 bytecode.
///
/// Bytecode (10 bytes): PUSH1 1 ; PUSH1 0 ; MSTORE ; PUSH1 32 ; PUSH1 0 ; RETURN
/// hex: `60 01 60 00 52 60 20 60 00 F3`
fn deploy_transfer_stub(contract: H160) {
	let runtime_bytecode = vec![0x60, 0x01, 0x60, 0x00, 0x52, 0x60, 0x20, 0x60, 0x00, 0xF3];
	pallet_evm::Pallet::<Test>::create_account(contract, runtime_bytecode, None)
		.expect("evm create_account succeeds in test ext");
}

// ---------------------------------------------------------------------------
// `add_teleportable_erc20`
// ---------------------------------------------------------------------------

/// Fresh add: `(none) → Registered`. The new entry is admitted by all gates so the
/// first leg can run, but `LockedSupply` is left untouched at zero.
#[test]
fn add_teleportable_erc20_fresh_inserts_as_registered() {
	new_test_ext().execute_with(|| {
		let contract = H160([1; 20]);
		assert!(!TeleportableErc20s::<Test>::contains_key(&contract));
		assert_ok!(Erc20XcmBridge::add_teleportable_erc20(
			RuntimeOrigin::root(),
			contract,
		));
		assert_eq!(
			TeleportableErc20s::<Test>::get(&contract),
			Some(TeleportableErc20Status::Registered),
		);
		assert!(LockedSupply::<Test>::get(&contract).is_zero());
		System::assert_has_event(RuntimeEvent::Erc20XcmBridge(
			Event::TeleportableErc20Added { contract },
		));
	});
}

#[test]
fn add_teleportable_erc20_requires_admin_origin() {
	new_test_ext().execute_with(|| {
		let contract = H160([2; 20]);
		assert_noop!(
			Erc20XcmBridge::add_teleportable_erc20(RuntimeOrigin::none(), contract),
			sp_runtime::DispatchError::BadOrigin,
		);
		assert_noop!(
			Erc20XcmBridge::add_teleportable_erc20(signed_origin(), contract),
			sp_runtime::DispatchError::BadOrigin,
		);
		assert!(!TeleportableErc20s::<Test>::contains_key(&contract));
	});
}

/// `add` on an already-`Registered` or already-`Active` contract is the duplicate
/// no-op case and rejects with `Erc20AlreadyTeleportable`.
#[test]
fn add_teleportable_erc20_rejects_registered_or_active() {
	new_test_ext().execute_with(|| {
		let contract = H160([3; 20]);
		assert_ok!(Erc20XcmBridge::add_teleportable_erc20(
			RuntimeOrigin::root(),
			contract,
		));
		// Status is `Registered`: duplicate add rejected.
		assert_noop!(
			Erc20XcmBridge::add_teleportable_erc20(RuntimeOrigin::root(), contract),
			crate::Error::<Test>::Erc20AlreadyTeleportable,
		);

		// Force the entry to `Active` (simulating a successful flow) and assert the
		// duplicate rule still fires there.
		TeleportableErc20s::<Test>::insert(&contract, TeleportableErc20Status::Active);
		assert_noop!(
			Erc20XcmBridge::add_teleportable_erc20(RuntimeOrigin::root(), contract),
			crate::Error::<Test>::Erc20AlreadyTeleportable,
		);
	});
}

/// Revival: `Deregistered → Registered` keeps any pre-existing `LockedSupply` so the
/// outstanding obligation continues to be tracked across the maintenance window.
#[test]
fn add_teleportable_erc20_revives_deregistered_to_registered_preserving_locked_supply() {
	new_test_ext().execute_with(|| {
		let contract = H160([0xab; 20]);
		// Simulate the pre-revival state: `Deregistered` + non-zero counter.
		TeleportableErc20s::<Test>::insert(&contract, TeleportableErc20Status::Deregistered);
		LockedSupply::<Test>::insert(&contract, U256::from(500u128));

		assert_ok!(Erc20XcmBridge::add_teleportable_erc20(
			RuntimeOrigin::root(),
			contract,
		));

		// Status revived; counter preserved.
		assert_eq!(
			TeleportableErc20s::<Test>::get(&contract),
			Some(TeleportableErc20Status::Registered),
		);
		assert_eq!(LockedSupply::<Test>::get(&contract), U256::from(500u128));

		// Same `Added` event fires.
		System::assert_has_event(RuntimeEvent::Erc20XcmBridge(
			Event::TeleportableErc20Added { contract },
		));
	});
}

// ---------------------------------------------------------------------------
// First-leg auto-promotion (`Registered → Active`) and counter maintenance
// ---------------------------------------------------------------------------

/// `withdraw_asset` (outbound lock leg) on a `Registered` entry promotes to `Active`,
/// increments `LockedSupply`, and emits `TeleportableErc20Activated` exactly once.
/// Subsequent legs on the same entry do not re-emit `Activated` and keep accumulating
/// the counter.
#[test]
fn withdraw_asset_promotes_registered_to_active_and_tracks_locked_supply() {
	new_test_ext().execute_with(|| {
		let contract = H160([0x55; 20]);
		deploy_transfer_stub(contract);

		assert_ok!(Erc20XcmBridge::add_teleportable_erc20(
			RuntimeOrigin::root(),
			contract,
		));
		assert_eq!(
			TeleportableErc20s::<Test>::get(&contract),
			Some(TeleportableErc20Status::Registered),
		);

		let user = user_loc([0x42; 20]);

		// First leg: Registered → Active, counter += 100.
		assert_ok!(Erc20TeleportTransactor::<Test>::withdraw_asset(
			&erc20_asset(contract.0, 100),
			&user,
			None,
		));
		assert_eq!(
			TeleportableErc20s::<Test>::get(&contract),
			Some(TeleportableErc20Status::Active),
		);
		assert_eq!(LockedSupply::<Test>::get(&contract), U256::from(100u128));
		System::assert_has_event(RuntimeEvent::Erc20XcmBridge(
			Event::TeleportableErc20Activated { contract },
		));

		// Second leg: still Active, counter += 250 → 350. Activated must NOT fire again.
		let activated_count_before = System::events()
			.iter()
			.filter(|r| {
				matches!(
					r.event,
					RuntimeEvent::Erc20XcmBridge(Event::TeleportableErc20Activated { .. })
				)
			})
			.count();
		assert_ok!(Erc20TeleportTransactor::<Test>::withdraw_asset(
			&erc20_asset(contract.0, 250),
			&user,
			None,
		));
		assert_eq!(LockedSupply::<Test>::get(&contract), U256::from(350u128));
		let activated_count_after = System::events()
			.iter()
			.filter(|r| {
				matches!(
					r.event,
					RuntimeEvent::Erc20XcmBridge(Event::TeleportableErc20Activated { .. })
				)
			})
			.count();
		assert_eq!(
			activated_count_before, activated_count_after,
			"Activated should fire exactly once per lifecycle, on the first leg only",
		);
	});
}

/// `deposit_asset` (inbound unlock leg) saturates the counter, promotes
/// `Registered → Active` on first leg, and unwinds outstanding supply on later legs.
#[test]
fn deposit_asset_promotes_and_saturating_subs_locked_supply() {
	new_test_ext().execute_with(|| {
		let contract = H160([0x66; 20]);
		deploy_transfer_stub(contract);

		// Pre-existing counter from a prior outbound (simulated directly to keep this
		// test focused on the inbound side).
		assert_ok!(Erc20XcmBridge::add_teleportable_erc20(
			RuntimeOrigin::root(),
			contract,
		));
		LockedSupply::<Test>::insert(&contract, U256::from(300u128));

		let beneficiary = user_loc([0x88; 20]);

		// First inbound leg from `Registered`: promotes to Active, counter -= 100.
		assert_ok!(Erc20TeleportTransactor::<Test>::deposit_asset(
			&erc20_asset(contract.0, 100),
			&beneficiary,
			None,
		));
		assert_eq!(
			TeleportableErc20s::<Test>::get(&contract),
			Some(TeleportableErc20Status::Active),
		);
		assert_eq!(LockedSupply::<Test>::get(&contract), U256::from(200u128));
		System::assert_has_event(RuntimeEvent::Erc20XcmBridge(
			Event::TeleportableErc20Activated { contract },
		));

		// Inbound for *more* than the counter holds: saturating-sub clamps to zero
		// (no abort, the user still gets their tokens). This handles pre-seeded twin
		// supply on the trusted counterparty.
		assert_ok!(Erc20TeleportTransactor::<Test>::deposit_asset(
			&erc20_asset(contract.0, 1_000_000),
			&beneficiary,
			None,
		));
		assert_eq!(LockedSupply::<Test>::get(&contract), U256::from(0u128));
	});
}

/// `internal_transfer_asset` (same-chain XCM hop) promotes `Registered → Active`
/// because it's a successful flow handled by this transactor, but it does NOT touch
/// `LockedSupply` — same-chain transfers never move the checking account.
#[test]
fn internal_transfer_promotes_but_does_not_touch_locked_supply() {
	new_test_ext().execute_with(|| {
		let contract = H160([0x77; 20]);
		deploy_transfer_stub(contract);

		assert_ok!(Erc20XcmBridge::add_teleportable_erc20(
			RuntimeOrigin::root(),
			contract,
		));

		let from = user_loc([0x42; 20]);
		let to = user_loc([0x43; 20]);

		assert_ok!(Erc20TeleportTransactor::<Test>::internal_transfer_asset(
			&erc20_asset(contract.0, 100),
			&from,
			&to,
			&xcm_ctx(),
		));
		assert_eq!(
			TeleportableErc20s::<Test>::get(&contract),
			Some(TeleportableErc20Status::Active),
		);
		assert!(LockedSupply::<Test>::get(&contract).is_zero());
	});
}

// ---------------------------------------------------------------------------
// Zero-amount fast path
// ---------------------------------------------------------------------------

/// All three transactor legs short-circuit on `Fungible(0)` for a whitelisted
/// contract: the EVM is never touched, `LockedSupply` is left at its prior value,
/// the lifecycle stays `Registered`, and no `Activated` event is emitted. This
/// pins the defense against spam-able zero-amount XCM teleports that would
/// otherwise burn gas, write the counter to its current value, and prematurely
/// flip `Registered → Active`.
///
/// We deliberately do NOT deploy the transfer stub here: if the fast path
/// fails to short-circuit, the call would surface the missing-bytecode EVM
/// error, making the regression unmistakable.
#[test]
fn zero_amount_legs_no_op_for_whitelisted_contract() {
	new_test_ext().execute_with(|| {
		let contract = H160([0x0a; 20]);
		assert_ok!(Erc20XcmBridge::add_teleportable_erc20(
			RuntimeOrigin::root(),
			contract,
		));

		// Pre-seed a non-zero `LockedSupply` so any spurious mutation would be
		// detectable (and not coincidentally land back at the same value).
		LockedSupply::<Test>::insert(&contract, U256::from(123u128));

		let user = user_loc([0x42; 20]);
		let other = user_loc([0x43; 20]);
		let asset_zero = erc20_asset_zero(contract.0);

		// Outbound (lock leg).
		assert_ok!(Erc20TeleportTransactor::<Test>::withdraw_asset(
			&asset_zero,
			&user,
			None,
		));

		// Inbound (unlock leg).
		assert_ok!(Erc20TeleportTransactor::<Test>::deposit_asset(
			&asset_zero,
			&user,
			None,
		));

		// Same-chain hop.
		assert_ok!(Erc20TeleportTransactor::<Test>::internal_transfer_asset(
			&asset_zero,
			&user,
			&other,
			&xcm_ctx(),
		));

		// Counter unchanged across all three legs.
		assert_eq!(LockedSupply::<Test>::get(&contract), U256::from(123u128));

		// Lifecycle still `Registered` — no auto-promotion fired.
		assert_eq!(
			TeleportableErc20s::<Test>::get(&contract),
			Some(TeleportableErc20Status::Registered),
		);

		// And no `TeleportableErc20Activated` event was emitted.
		let activated_count = System::events()
			.iter()
			.filter(|r| {
				matches!(
					r.event,
					RuntimeEvent::Erc20XcmBridge(Event::TeleportableErc20Activated { .. })
				)
			})
			.count();
		assert_eq!(
			activated_count, 0,
			"zero-amount legs must not emit `TeleportableErc20Activated`",
		);
	});
}

/// Pins that the zero-amount fast path is gated by the whitelist: a non-whitelisted
/// contract with `Fungible(0)` still surfaces `AssetNotFound` from the matcher, so
/// the legacy reserve adapter sitting after this transactor in `AssetTransactors`
/// can take over (or, if it also rejects, the runtime returns a clean error rather
/// than silently succeeding on an unknown asset).
#[test]
fn zero_amount_legs_still_reject_non_whitelisted() {
	new_test_ext().execute_with(|| {
		let unknown = H160([0x0b; 20]);
		let user = user_loc([0x42; 20]);
		let other = user_loc([0x43; 20]);
		let asset_zero = erc20_asset_zero(unknown.0);

		assert_eq!(
			Erc20TeleportTransactor::<Test>::withdraw_asset(&asset_zero, &user, None,).err(),
			Some(XcmError::AssetNotFound),
		);
		assert_eq!(
			Erc20TeleportTransactor::<Test>::deposit_asset(&asset_zero, &user, None,).err(),
			Some(XcmError::AssetNotFound),
		);
		assert_eq!(
			Erc20TeleportTransactor::<Test>::internal_transfer_asset(
				&asset_zero,
				&user,
				&other,
				&xcm_ctx(),
			)
			.err(),
			Some(XcmError::AssetNotFound),
		);
	});
}

// ---------------------------------------------------------------------------
// `remove_teleportable_erc20`
// ---------------------------------------------------------------------------

#[test]
fn remove_teleportable_erc20_rejects_unknown_contract() {
	new_test_ext().execute_with(|| {
		let contract = H160([5; 20]);
		assert_noop!(
			Erc20XcmBridge::remove_teleportable_erc20(RuntimeOrigin::root(), contract),
			crate::Error::<Test>::Erc20NotTeleportable,
		);
		assert_noop!(
			Erc20XcmBridge::remove_teleportable_erc20(signed_origin(), contract),
			crate::Error::<Test>::Erc20NotTeleportable,
		);
	});
}

/// Purge path on `Registered + count == 0` requires admin origin. A signed user's
/// attempt is rejected with `BadOrigin` (so a third party can't snipe a fresh add
/// before the operator is done configuring).
#[test]
fn remove_on_registered_with_zero_count_is_admin_only_purge() {
	new_test_ext().execute_with(|| {
		let contract = H160([0xa1; 20]);
		assert_ok!(Erc20XcmBridge::add_teleportable_erc20(
			RuntimeOrigin::root(),
			contract,
		));
		assert!(LockedSupply::<Test>::get(&contract).is_zero());

		// Signed (non-admin) is rejected: Registered's purge path is admin-only.
		assert_noop!(
			Erc20XcmBridge::remove_teleportable_erc20(signed_origin(), contract),
			sp_runtime::DispatchError::BadOrigin,
		);
		assert_eq!(
			TeleportableErc20s::<Test>::get(&contract),
			Some(TeleportableErc20Status::Registered),
		);

		// Admin: purges. Both maps cleared. `Purged` event emitted (NOT `Removed`).
		assert_ok!(Erc20XcmBridge::remove_teleportable_erc20(
			RuntimeOrigin::root(),
			contract,
		));
		assert!(!TeleportableErc20s::<Test>::contains_key(&contract));
		assert!(!LockedSupply::<Test>::contains_key(&contract));
		System::assert_has_event(RuntimeEvent::Erc20XcmBridge(
			Event::TeleportableErc20Purged { contract },
		));
	});
}

/// Permissionless purge on `Active + count == 0`: any signed user can sweep an entry
/// whose obligation has been fully discharged. The same call from admin also works.
#[test]
fn remove_on_active_with_zero_count_is_permissionless_purge() {
	new_test_ext().execute_with(|| {
		let contract = H160([0xa2; 20]);
		// Pre-seed `Active + count == 0` (e.g. all supply was teleported back already).
		TeleportableErc20s::<Test>::insert(&contract, TeleportableErc20Status::Active);

		assert_ok!(Erc20XcmBridge::remove_teleportable_erc20(
			signed_origin(),
			contract,
		));
		assert!(!TeleportableErc20s::<Test>::contains_key(&contract));
		assert!(!LockedSupply::<Test>::contains_key(&contract));
		System::assert_has_event(RuntimeEvent::Erc20XcmBridge(
			Event::TeleportableErc20Purged { contract },
		));

		// Same call from admin also works (admin is the strict superset).
		TeleportableErc20s::<Test>::insert(&contract, TeleportableErc20Status::Active);
		assert_ok!(Erc20XcmBridge::remove_teleportable_erc20(
			RuntimeOrigin::root(),
			contract,
		));
		assert!(!TeleportableErc20s::<Test>::contains_key(&contract));
	});
}

/// Permissionless purge on `Deregistered + count == 0`: same as `Active + count == 0`
/// — anyone can sweep an entry whose users have all teleported their twin home.
#[test]
fn remove_on_deregistered_with_zero_count_is_permissionless_purge() {
	new_test_ext().execute_with(|| {
		let contract = H160([0xa3; 20]);
		TeleportableErc20s::<Test>::insert(&contract, TeleportableErc20Status::Deregistered);

		assert_ok!(Erc20XcmBridge::remove_teleportable_erc20(
			signed_origin(),
			contract,
		));
		assert!(!TeleportableErc20s::<Test>::contains_key(&contract));
		System::assert_has_event(RuntimeEvent::Erc20XcmBridge(
			Event::TeleportableErc20Purged { contract },
		));
	});
}

/// State-change path on `Active + count > 0` requires admin and flips to
/// `Deregistered`. The counter is preserved verbatim so users can keep teleporting
/// their twin home and unwinding it.
#[test]
fn remove_on_active_with_positive_count_admin_only_flips_to_deregistered() {
	new_test_ext().execute_with(|| {
		let contract = H160([0xa4; 20]);
		TeleportableErc20s::<Test>::insert(&contract, TeleportableErc20Status::Active);
		LockedSupply::<Test>::insert(&contract, U256::from(500u128));

		// Signed user can't flip state.
		assert_noop!(
			Erc20XcmBridge::remove_teleportable_erc20(signed_origin(), contract),
			sp_runtime::DispatchError::BadOrigin,
		);

		// Admin: flips to Deregistered. Counter preserved. `Removed` event (NOT `Purged`).
		assert_ok!(Erc20XcmBridge::remove_teleportable_erc20(
			RuntimeOrigin::root(),
			contract,
		));
		assert_eq!(
			TeleportableErc20s::<Test>::get(&contract),
			Some(TeleportableErc20Status::Deregistered),
		);
		assert_eq!(LockedSupply::<Test>::get(&contract), U256::from(500u128));
		System::assert_has_event(RuntimeEvent::Erc20XcmBridge(
			Event::TeleportableErc20Removed { contract },
		));
	});
}

/// State-change path on `Deregistered + count > 0` is the no-op error. The contract
/// is already retired and its outstanding obligation has not yet been discharged;
/// the operator must wait or use `force_remove_teleportable_erc20`.
#[test]
fn remove_on_deregistered_with_positive_count_returns_already_removed() {
	new_test_ext().execute_with(|| {
		let contract = H160([0xa5; 20]);
		TeleportableErc20s::<Test>::insert(&contract, TeleportableErc20Status::Deregistered);
		LockedSupply::<Test>::insert(&contract, U256::from(10u128));

		// Signed user: rejected at origin layer (we ensure_origin first when count > 0).
		assert_noop!(
			Erc20XcmBridge::remove_teleportable_erc20(signed_origin(), contract),
			sp_runtime::DispatchError::BadOrigin,
		);
		// Admin: rejected with `Erc20AlreadyRemoved`.
		assert_noop!(
			Erc20XcmBridge::remove_teleportable_erc20(RuntimeOrigin::root(), contract),
			crate::Error::<Test>::Erc20AlreadyRemoved,
		);
	});
}

/// Full happy-path lifecycle, including the permissionless terminus. End-to-end:
///   add → first outbound (auto-Active) → second outbound (count grows) →
///   admin remove (count > 0 → Deregistered) → user inbound (count → 0) →
///   permissionless remove (count == 0 → purge) → fresh add succeeds again.
#[test]
fn lifecycle_end_to_end_purge_after_drain() {
	new_test_ext().execute_with(|| {
		let contract = H160([0xc7; 20]);
		deploy_transfer_stub(contract);

		assert_ok!(Erc20XcmBridge::add_teleportable_erc20(
			RuntimeOrigin::root(),
			contract,
		));

		let user = user_loc([0x42; 20]);
		// 2 outbound legs: counter = 100 + 50 = 150, Active.
		assert_ok!(Erc20TeleportTransactor::<Test>::withdraw_asset(
			&erc20_asset(contract.0, 100),
			&user,
			None,
		));
		assert_ok!(Erc20TeleportTransactor::<Test>::withdraw_asset(
			&erc20_asset(contract.0, 50),
			&user,
			None,
		));
		assert_eq!(LockedSupply::<Test>::get(&contract), U256::from(150u128));

		// Admin retires: count > 0 → Deregistered.
		assert_ok!(Erc20XcmBridge::remove_teleportable_erc20(
			RuntimeOrigin::root(),
			contract,
		));
		assert_eq!(
			TeleportableErc20s::<Test>::get(&contract),
			Some(TeleportableErc20Status::Deregistered),
		);

		// Users teleport everything home. Counter unwinds to zero.
		assert_ok!(Erc20TeleportTransactor::<Test>::deposit_asset(
			&erc20_asset(contract.0, 150),
			&user,
			None,
		));
		assert_eq!(LockedSupply::<Test>::get(&contract), U256::from(0u128));

		// Anyone can now sweep the entry: permissionless purge.
		assert_ok!(Erc20XcmBridge::remove_teleportable_erc20(
			signed_origin(),
			contract,
		));
		assert!(!TeleportableErc20s::<Test>::contains_key(&contract));
		assert!(!LockedSupply::<Test>::contains_key(&contract));

		// Subsequent `remove` returns `NotTeleportable` — the entry is gone.
		assert_noop!(
			Erc20XcmBridge::remove_teleportable_erc20(RuntimeOrigin::root(), contract),
			crate::Error::<Test>::Erc20NotTeleportable,
		);

		// Re-adding works: fresh add lands at `Registered` (no leftover state).
		assert_ok!(Erc20XcmBridge::add_teleportable_erc20(
			RuntimeOrigin::root(),
			contract,
		));
		assert_eq!(
			TeleportableErc20s::<Test>::get(&contract),
			Some(TeleportableErc20Status::Registered),
		);
	});
}

// ---------------------------------------------------------------------------
// `force_remove_teleportable_erc20`
// ---------------------------------------------------------------------------

#[test]
fn force_remove_teleportable_erc20_requires_admin_origin() {
	new_test_ext().execute_with(|| {
		let contract = H160([0xf1; 20]);
		TeleportableErc20s::<Test>::insert(&contract, TeleportableErc20Status::Active);
		assert_noop!(
			Erc20XcmBridge::force_remove_teleportable_erc20(signed_origin(), contract),
			sp_runtime::DispatchError::BadOrigin,
		);
		assert_noop!(
			Erc20XcmBridge::force_remove_teleportable_erc20(RuntimeOrigin::none(), contract),
			sp_runtime::DispatchError::BadOrigin,
		);
		assert!(TeleportableErc20s::<Test>::contains_key(&contract));
	});
}

#[test]
fn force_remove_teleportable_erc20_rejects_unknown_contract() {
	new_test_ext().execute_with(|| {
		let contract = H160([0xf2; 20]);
		assert_noop!(
			Erc20XcmBridge::force_remove_teleportable_erc20(RuntimeOrigin::root(), contract),
			crate::Error::<Test>::Erc20NotTeleportable,
		);
	});
}

/// `force_remove` purges regardless of state and counter, and emits an event with
/// `status_before` and `locked_supply` so the act is auditable from chain events.
/// Cover the three states with non-trivial counter values.
#[test]
fn force_remove_purges_any_state_and_emits_status_and_locked_supply() {
	new_test_ext().execute_with(|| {
		// (status, counter)
		let cases = [
			(TeleportableErc20Status::Registered, 0u128),
			(TeleportableErc20Status::Active, 1_234u128),
			(TeleportableErc20Status::Deregistered, 9_999u128),
		];
		for (i, (status, counter)) in cases.iter().enumerate() {
			let contract = H160([0xf0 + i as u8; 20]);
			TeleportableErc20s::<Test>::insert(&contract, *status);
			LockedSupply::<Test>::insert(&contract, U256::from(*counter));

			assert_ok!(Erc20XcmBridge::force_remove_teleportable_erc20(
				RuntimeOrigin::root(),
				contract,
			));
			assert!(!TeleportableErc20s::<Test>::contains_key(&contract));
			assert!(!LockedSupply::<Test>::contains_key(&contract));

			System::assert_has_event(RuntimeEvent::Erc20XcmBridge(
				Event::TeleportableErc20ForceRemoved {
					contract,
					status_before: *status,
					locked_supply: U256::from(*counter),
				},
			));
		}
	});
}

/// After a `force_remove` with locked supply still > 0, inbound teleports for that
/// contract must be refused — the entry is gone, so the gates fall through to the
/// legacy reserve adapter via `AssetNotFound`. Pins the audit trail consequence.
#[test]
fn force_remove_with_outstanding_supply_blocks_inbound_after() {
	new_test_ext().execute_with(|| {
		let contract = H160([0xf3; 20]);
		deploy_transfer_stub(contract);
		TeleportableErc20s::<Test>::insert(&contract, TeleportableErc20Status::Active);
		LockedSupply::<Test>::insert(&contract, U256::from(500u128));

		assert_ok!(Erc20XcmBridge::force_remove_teleportable_erc20(
			RuntimeOrigin::root(),
			contract,
		));

		// Inbound from the trusted location now fails because the asset is no longer
		// admitted. Counter is gone too, so there's no obligation to unwind.
		let asset = erc20_asset(contract.0, 100);
		assert_eq!(
			Erc20TeleportTransactor::<Test>::can_check_in(&trusted_loc(), &asset, &xcm_ctx()),
			Err(XcmError::AssetNotFound),
		);
		assert_eq!(
			Erc20TeleportTransactor::<Test>::deposit_asset(&asset, &user_loc([0x88; 20]), None,)
				.err(),
			Some(XcmError::AssetNotFound),
		);
	});
}

// ---------------------------------------------------------------------------
// Public read accessors
// ---------------------------------------------------------------------------

/// Cover `is_teleportable_erc20` (admits any entry) vs. `is_outbound_eligible_erc20`
/// (admits `Registered` or `Active`) across all four storage states. They are the
/// read-side API surfaced to runtime code; pinning them here protects against a
/// future refactor flipping the semantics.
#[test]
fn pallet_helpers_track_storage_state_across_lifecycle() {
	new_test_ext().execute_with(|| {
		let contract = H160([0xb7; 20]);

		// Unknown.
		assert!(!Erc20XcmBridge::is_teleportable_erc20(&contract));
		assert!(!Erc20XcmBridge::is_outbound_eligible_erc20(&contract));

		// Registered: admitted everywhere, including outbound (so the first leg can run).
		TeleportableErc20s::<Test>::insert(&contract, TeleportableErc20Status::Registered);
		assert!(Erc20XcmBridge::is_teleportable_erc20(&contract));
		assert!(Erc20XcmBridge::is_outbound_eligible_erc20(&contract));

		// Active: same.
		TeleportableErc20s::<Test>::insert(&contract, TeleportableErc20Status::Active);
		assert!(Erc20XcmBridge::is_teleportable_erc20(&contract));
		assert!(Erc20XcmBridge::is_outbound_eligible_erc20(&contract));

		// Deregistered: still admitted (so inbound stays open) but outbound-blocked.
		TeleportableErc20s::<Test>::insert(&contract, TeleportableErc20Status::Deregistered);
		assert!(Erc20XcmBridge::is_teleportable_erc20(&contract));
		assert!(!Erc20XcmBridge::is_outbound_eligible_erc20(&contract));
	});
}

// ---------------------------------------------------------------------------
// Filter behaviour (`IsTeleportableErc20`)
// ---------------------------------------------------------------------------

/// `IsTeleporter` (the `ContainsPair<Asset, Location>` impl) keeps admitting all
/// three states from the trusted peer. The location bind rejects every other origin.
#[test]
fn is_teleporter_pair_admits_all_states_from_trusted_location() {
	new_test_ext().execute_with(|| {
		let contract = H160([0xc1; 20]);
		let asset = erc20_asset(contract.0, 1_000);
		let trusted = trusted_loc();
		let untrusted_sibling = Location::new(1, [Junction::Parachain(2042)]);
		let relay = Location::parent();

		for status in [
			TeleportableErc20Status::Registered,
			TeleportableErc20Status::Active,
			TeleportableErc20Status::Deregistered,
		] {
			TeleportableErc20s::<Test>::insert(&contract, status);

			// Trusted peer + admitted asset: always pass.
			assert!(<IsTeleportableErc20<Test> as ContainsPair<
				Asset,
				Location,
			>>::contains(&asset, &trusted));

			// Untrusted peers: always rejected, regardless of state.
			assert!(!<IsTeleportableErc20<Test> as ContainsPair<
				Asset,
				Location,
			>>::contains(&asset, &untrusted_sibling));
			assert!(!<IsTeleportableErc20<Test> as ContainsPair<
				Asset,
				Location,
			>>::contains(&asset, &relay));
		}

		// Unknown contract: rejected even from the trusted peer.
		TeleportableErc20s::<Test>::remove(&contract);
		assert!(!<IsTeleportableErc20<Test> as ContainsPair<
			Asset,
			Location,
		>>::contains(&asset, &trusted));
	});
}

/// User-facing outbound gate (`Contains<(Location, Vec<Asset>)>`): admits `Registered`
/// and `Active`, rejects `Deregistered`. Empty asset lists are rejected (defense
/// against trivial bypass). Mixed lists are rejected (outbound-eligible required for
/// every asset).
#[test]
fn xcm_teleport_filter_admits_outbound_eligible_only() {
	new_test_ext().execute_with(|| {
		let contract = H160([0xc2; 20]);
		let asset = erc20_asset(contract.0, 1_000);
		let dest = trusted_loc();

		// Registered: admitted.
		TeleportableErc20s::<Test>::insert(&contract, TeleportableErc20Status::Registered);
		assert!(<IsTeleportableErc20<Test> as Contains<(
			Location,
			Vec<Asset>,
		)>>::contains(&(dest.clone(), vec![asset.clone()])));

		// Active: admitted.
		TeleportableErc20s::<Test>::insert(&contract, TeleportableErc20Status::Active);
		assert!(<IsTeleportableErc20<Test> as Contains<(
			Location,
			Vec<Asset>,
		)>>::contains(&(dest.clone(), vec![asset.clone()])));

		// Deregistered: rejected.
		TeleportableErc20s::<Test>::insert(&contract, TeleportableErc20Status::Deregistered);
		assert!(!<IsTeleportableErc20<Test> as Contains<(
			Location,
			Vec<Asset>,
		)>>::contains(&(dest.clone(), vec![asset.clone()])));

		// Unknown: rejected.
		TeleportableErc20s::<Test>::remove(&contract);
		assert!(!<IsTeleportableErc20<Test> as Contains<(
			Location,
			Vec<Asset>,
		)>>::contains(&(dest.clone(), vec![asset.clone()])));

		// Empty asset list: rejected.
		assert!(!<IsTeleportableErc20<Test> as Contains<(
			Location,
			Vec<Asset>,
		)>>::contains(&(dest.clone(), Vec::<Asset>::new())));

		// Mixed list (one outbound-eligible + one unknown): rejected.
		let other = H160([0xc3; 20]);
		TeleportableErc20s::<Test>::insert(&contract, TeleportableErc20Status::Active);
		assert!(!<IsTeleportableErc20<Test> as Contains<(
			Location,
			Vec<Asset>,
		)>>::contains(&(
			dest,
			vec![erc20_asset(contract.0, 1), erc20_asset(other.0, 1)],
		)));

		// Native asset: never admitted by either filter form.
		let native = Asset::from((Location::parent(), 1u128));
		assert!(!<IsTeleportableErc20<Test> as ContainsPair<
			Asset,
			Location,
		>>::contains(&native, &trusted_loc()));
	});
}

/// Asset-transactor location-bind: trusted peer is the only origin/destination
/// admitted by `can_check_in`/`can_check_out`. The `ensure_trusted_peer` gate runs
/// before the asset-status check, so untrusted peers get `UntrustedTeleportLocation`
/// even for an `Active` asset (and `AssetNotFound` only when status fails for the
/// trusted peer).
#[test]
fn transactor_check_hooks_bind_to_trusted_location() {
	new_test_ext().execute_with(|| {
		let contract = H160([0x9a; 20]);
		let asset = erc20_asset(contract.0, 1_000);
		let trusted = trusted_loc();
		let untrusted = Location::new(1, [Junction::Parachain(2042)]);
		let ctx = xcm_ctx();

		assert_ok!(Erc20XcmBridge::add_teleportable_erc20(
			RuntimeOrigin::root(),
			contract,
		));

		assert_eq!(
			Erc20TeleportTransactor::<Test>::can_check_in(&untrusted, &asset, &ctx),
			Err(XcmError::UntrustedTeleportLocation),
		);
		assert_eq!(
			Erc20TeleportTransactor::<Test>::can_check_out(&untrusted, &asset, &ctx),
			Err(XcmError::UntrustedTeleportLocation),
		);

		assert_ok!(Erc20TeleportTransactor::<Test>::can_check_in(
			&trusted, &asset, &ctx,
		));
		assert_ok!(Erc20TeleportTransactor::<Test>::can_check_out(
			&trusted, &asset, &ctx,
		));
	});
}

/// Outbound transactor methods (`withdraw_asset`, `internal_transfer_asset`,
/// `can_check_out`) reject `Deregistered` and unknown contracts at the match layer
/// BEFORE any EVM call, surfacing as `XcmError::AssetNotFound`. This is what lets the
/// legacy reserve adapter, placed after this transactor in `AssetTransactors`, take
/// over for non-eligible cases. Crucially this proves that retiring a contract via
/// `remove_teleportable_erc20` halts new outbound at the asset-transactor layer too,
/// not just at the upstream `XcmTeleportFilter`.
#[test]
fn outbound_transactor_rejects_deregistered_and_unknown() {
	new_test_ext().execute_with(|| {
		let deregistered = H160([0xd0; 20]);
		let unknown = H160([0xd1; 20]);
		TeleportableErc20s::<Test>::insert(&deregistered, TeleportableErc20Status::Deregistered);

		let user = user_loc([0x42; 20]);
		let other = user_loc([0x43; 20]);
		let ctx = xcm_ctx();

		for &contract in &[deregistered, unknown] {
			let asset = erc20_asset(contract.0, 100);
			assert_eq!(
				Erc20TeleportTransactor::<Test>::withdraw_asset(&asset, &user, None).err(),
				Some(XcmError::AssetNotFound),
			);
			assert_eq!(
				Erc20TeleportTransactor::<Test>::internal_transfer_asset(
					&asset, &user, &other, &ctx,
				)
				.err(),
				Some(XcmError::AssetNotFound),
			);
			assert_eq!(
				Erc20TeleportTransactor::<Test>::can_check_out(&trusted_loc(), &asset, &ctx),
				Err(XcmError::AssetNotFound),
			);
		}
	});
}

/// Inbound transactor entry (`deposit_asset`) admits all three whitelist states; only
/// unknown contracts short-circuit to `AssetNotFound`. The `Deregistered` admission
/// is the whole point of keeping the entry in storage after `remove_teleportable_erc20`.
#[test]
fn deposit_asset_admits_all_states_rejects_only_unknown() {
	new_test_ext().execute_with(|| {
		let unknown = H160([0xd2; 20]);
		assert_eq!(
			Erc20TeleportTransactor::<Test>::deposit_asset(
				&erc20_asset(unknown.0, 100),
				&user_loc([0x88; 20]),
				None,
			)
			.err(),
			Some(XcmError::AssetNotFound),
		);

		// Deregistered: admitted (otherwise users can't pull supply home after `remove`).
		// We deploy the transfer stub and exercise the actual path so the unwind
		// counter mutation is observed too.
		let dereg = H160([0xd3; 20]);
		deploy_transfer_stub(dereg);
		TeleportableErc20s::<Test>::insert(&dereg, TeleportableErc20Status::Deregistered);
		LockedSupply::<Test>::insert(&dereg, U256::from(500u128));
		assert_ok!(Erc20TeleportTransactor::<Test>::deposit_asset(
			&erc20_asset(dereg.0, 200),
			&user_loc([0x88; 20]),
			None,
		));
		assert_eq!(LockedSupply::<Test>::get(&dereg), U256::from(300u128));
		// Status stays Deregistered — promotion only fires from Registered.
		assert_eq!(
			TeleportableErc20s::<Test>::get(&dereg),
			Some(TeleportableErc20Status::Deregistered),
		);
	});
}

// ---------------------------------------------------------------------------
// Pre-existing gas-limit / matcher-shape regressions (carried over)
// ---------------------------------------------------------------------------

#[test]
fn general_key_data_size_32() {
	let junction: Junction = (BoundedVec::new()).into();

	// Assert that GeneralKey data length is 32 bytes
	match junction {
		Junction::GeneralKey { length: _, data } => {
			let _: [u8; 32] = data;
		}
		_ => assert!(false),
	}

	assert_eq!(
		Erc20XcmBridge::gas_limit_of_erc20_transfer(&junction.into()),
		Erc20XcmBridgeTransferGasLimit::get()
	)
}

#[test]
fn gas_limit_override() {
	let text = "gas_limit:".as_bytes();
	let limit = 300_000u64;
	let data = [text, &limit.to_le_bytes()].concat();
	let vec = BoundedVec::try_from(data).expect("vec should convert");
	let junction: Junction = (vec).into();
	assert_eq!(
		Erc20XcmBridge::gas_limit_of_erc20_transfer(&junction.into()),
		limit
	)
}

#[test]
fn gas_limit_override_typo() {
	let text = "gaslimit:".as_bytes();
	let limit = 300_000u64;
	let data = [text, &limit.to_le_bytes()].concat();
	let vec = BoundedVec::try_from(data).expect("vec should convert");
	let junction: Junction = (vec).into();
	assert_eq!(
		Erc20XcmBridge::gas_limit_of_erc20_transfer(&junction.into()),
		Erc20XcmBridgeTransferGasLimit::get()
	)
}
