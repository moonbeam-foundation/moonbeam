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
use sp_core::H160;
use sp_io::TestExternalities;
use sp_runtime::{BoundedVec, BuildStorage};
use xcm::latest::prelude::XcmError;
use xcm::latest::{Asset, Junction, Junctions, Location, XcmContext};
use xcm_executor::traits::TransactAsset;

use crate::mock::{
	Erc20XcmBridge, Erc20XcmBridgeTransferGasLimit, RuntimeEvent, RuntimeOrigin, System, Test,
};
use crate::{
	Erc20TeleportTransactor, Event, IsTeleportableErc20, TeleportableErc20Status,
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

#[test]
fn add_teleportable_erc20_root_works_and_emits_event() {
	new_test_ext().execute_with(|| {
		let contract = H160([1; 20]);
		assert!(!TeleportableErc20s::<Test>::contains_key(&contract));
		assert_ok!(Erc20XcmBridge::add_teleportable_erc20(
			RuntimeOrigin::root(),
			contract,
		));
		assert_eq!(
			TeleportableErc20s::<Test>::get(&contract),
			Some(TeleportableErc20Status::Active),
		);
		System::assert_has_event(RuntimeEvent::Erc20XcmBridge(
			Event::TeleportableErc20Added { contract },
		));
	});
}

#[test]
fn add_teleportable_erc20_requires_root() {
	new_test_ext().execute_with(|| {
		let contract = H160([2; 20]);
		assert_noop!(
			Erc20XcmBridge::add_teleportable_erc20(RuntimeOrigin::none(), contract),
			sp_runtime::DispatchError::BadOrigin
		);
		assert!(!TeleportableErc20s::<Test>::contains_key(&contract));
	});
}

#[test]
fn add_teleportable_erc20_rejects_duplicate_active() {
	new_test_ext().execute_with(|| {
		let contract = H160([3; 20]);
		assert_ok!(Erc20XcmBridge::add_teleportable_erc20(
			RuntimeOrigin::root(),
			contract,
		));
		// Calling `add` on an already-`Active` contract is the no-op case and returns
		// `Erc20AlreadyTeleportable`. The "re-activate from `InboundOnly`" case is
		// covered by `lifecycle_inbound_only_can_be_reactivated_by_root`.
		assert_noop!(
			Erc20XcmBridge::add_teleportable_erc20(RuntimeOrigin::root(), contract),
			crate::Error::<Test>::Erc20AlreadyTeleportable
		);
	});
}

/// `remove_teleportable_erc20` flips `Active → InboundOnly` in a single step. The
/// whitelist entry is intentionally kept in storage so the inbound (return) path stays
/// open. `add_teleportable_erc20` is the legal `InboundOnly → Active` revival path; the
/// reactivation regression is covered separately below.
#[test]
fn lifecycle_active_to_inbound_only() {
	new_test_ext().execute_with(|| {
		let contract = H160([4; 20]);

		// Add → Active.
		assert_ok!(Erc20XcmBridge::add_teleportable_erc20(
			RuntimeOrigin::root(),
			contract,
		));
		assert_eq!(
			TeleportableErc20s::<Test>::get(&contract),
			Some(TeleportableErc20Status::Active),
		);
		System::assert_has_event(RuntimeEvent::Erc20XcmBridge(
			Event::TeleportableErc20Added { contract },
		));

		// Active → InboundOnly. Outbound is now closed; inbound remains open so users
		// can teleport back any supply still locked in the EVM checking account.
		assert_ok!(Erc20XcmBridge::remove_teleportable_erc20(
			RuntimeOrigin::root(),
			contract,
		));
		assert_eq!(
			TeleportableErc20s::<Test>::get(&contract),
			Some(TeleportableErc20Status::InboundOnly),
		);
		System::assert_has_event(RuntimeEvent::Erc20XcmBridge(
			Event::TeleportableErc20Removed { contract },
		));

		// Removing again is rejected: `InboundOnly` is the terminal `remove` state.
		assert_noop!(
			Erc20XcmBridge::remove_teleportable_erc20(RuntimeOrigin::root(), contract),
			crate::Error::<Test>::Erc20AlreadyRemoved,
		);
	});
}

/// Root can reverse `remove_teleportable_erc20` by calling `add_teleportable_erc20`
/// again, flipping the entry back from `InboundOnly` to `Active`. This pins both:
/// - the storage transition (`InboundOnly → Active` via `add`), and
/// - the runtime-visible effect (the outbound `XcmTeleportFilter` admits the contract
///   again, while it had been rejecting the very same asset while `InboundOnly`).
#[test]
fn lifecycle_inbound_only_can_be_reactivated_by_root() {
	new_test_ext().execute_with(|| {
		let contract = H160([0xab; 20]);
		let asset = erc20_asset(contract.0, 1_000);
		let trusted = Location::new(1, [Junction::Parachain(1001)]);

		// Active → InboundOnly.
		assert_ok!(Erc20XcmBridge::add_teleportable_erc20(
			RuntimeOrigin::root(),
			contract,
		));
		assert_ok!(Erc20XcmBridge::remove_teleportable_erc20(
			RuntimeOrigin::root(),
			contract,
		));
		assert_eq!(
			TeleportableErc20s::<Test>::get(&contract),
			Some(TeleportableErc20Status::InboundOnly),
		);
		// Outbound user gate (`XcmTeleportFilter`) rejects the contract while
		// `InboundOnly` — sanity baseline before the re-activation.
		assert!(!<IsTeleportableErc20<Test> as Contains<(
			Location,
			Vec<Asset>,
		)>>::contains(&(trusted.clone(), vec![asset.clone()])));

		// InboundOnly → Active via `add_teleportable_erc20`. Same extrinsic that
		// inserts fresh entries; here it revives the existing one.
		assert_ok!(Erc20XcmBridge::add_teleportable_erc20(
			RuntimeOrigin::root(),
			contract,
		));
		assert_eq!(
			TeleportableErc20s::<Test>::get(&contract),
			Some(TeleportableErc20Status::Active),
		);
		System::assert_has_event(RuntimeEvent::Erc20XcmBridge(
			Event::TeleportableErc20Added { contract },
		));

		// Outbound is open again — the runtime-visible payoff of re-activation.
		assert!(<IsTeleportableErc20<Test> as Contains<(
			Location,
			Vec<Asset>,
		)>>::contains(&(trusted, vec![asset])));

		// Now in `Active`, calling `add` again is the duplicate-active no-op.
		assert_noop!(
			Erc20XcmBridge::add_teleportable_erc20(RuntimeOrigin::root(), contract),
			crate::Error::<Test>::Erc20AlreadyTeleportable,
		);
	});
}

/// Pin that the `Active ↔ InboundOnly` cycle is fully repeatable: nothing latches.
/// Every state transition emits the right event, and the `add` no-op vs. re-activation
/// branches stay distinct across multiple cycles.
#[test]
fn lifecycle_active_inbound_active_cycle_repeats() {
	new_test_ext().execute_with(|| {
		let contract = H160([0xc7; 20]);

		// Cycle 1: fresh add → remove.
		assert_ok!(Erc20XcmBridge::add_teleportable_erc20(
			RuntimeOrigin::root(),
			contract,
		));
		assert_ok!(Erc20XcmBridge::remove_teleportable_erc20(
			RuntimeOrigin::root(),
			contract,
		));
		assert_eq!(
			TeleportableErc20s::<Test>::get(&contract),
			Some(TeleportableErc20Status::InboundOnly),
		);
		// Cycle 1 is terminal w.r.t. `remove`: a second `remove` is rejected.
		assert_noop!(
			Erc20XcmBridge::remove_teleportable_erc20(RuntimeOrigin::root(), contract),
			crate::Error::<Test>::Erc20AlreadyRemoved,
		);

		// Cycle 2: re-activate → remove → reject duplicate `remove`.
		assert_ok!(Erc20XcmBridge::add_teleportable_erc20(
			RuntimeOrigin::root(),
			contract,
		));
		assert_eq!(
			TeleportableErc20s::<Test>::get(&contract),
			Some(TeleportableErc20Status::Active),
		);
		// `add` on Active is the duplicate no-op, even after several cycles.
		assert_noop!(
			Erc20XcmBridge::add_teleportable_erc20(RuntimeOrigin::root(), contract),
			crate::Error::<Test>::Erc20AlreadyTeleportable,
		);
		assert_ok!(Erc20XcmBridge::remove_teleportable_erc20(
			RuntimeOrigin::root(),
			contract,
		));

		// Cycle 3: revive once more, prove the entry never decays into "unknown".
		assert_ok!(Erc20XcmBridge::add_teleportable_erc20(
			RuntimeOrigin::root(),
			contract,
		));
		assert_eq!(
			TeleportableErc20s::<Test>::get(&contract),
			Some(TeleportableErc20Status::Active),
		);

		// Each successful add/remove emitted exactly one event of the matching kind: 3
		// `Added` (one fresh + two reactivations) and 2 `Removed`.
		let added_count = System::events()
			.iter()
			.filter(|r| {
				matches!(
					r.event,
					RuntimeEvent::Erc20XcmBridge(Event::TeleportableErc20Added { contract: c })
						if c == contract
				)
			})
			.count();
		let removed_count = System::events()
			.iter()
			.filter(|r| {
				matches!(
					r.event,
					RuntimeEvent::Erc20XcmBridge(Event::TeleportableErc20Removed { contract: c })
						if c == contract
				)
			})
			.count();
		assert_eq!(added_count, 3);
		assert_eq!(removed_count, 2);
	});
}

#[test]
fn lifecycle_extrinsics_reject_unknown_contracts() {
	new_test_ext().execute_with(|| {
		let contract = H160([5; 20]);
		assert_noop!(
			Erc20XcmBridge::remove_teleportable_erc20(RuntimeOrigin::root(), contract),
			crate::Error::<Test>::Erc20NotTeleportable,
		);
	});
}

#[test]
fn lifecycle_extrinsics_require_admin_origin() {
	new_test_ext().execute_with(|| {
		let contract = H160([6; 20]);
		assert_ok!(Erc20XcmBridge::add_teleportable_erc20(
			RuntimeOrigin::root(),
			contract,
		));
		assert_noop!(
			Erc20XcmBridge::remove_teleportable_erc20(RuntimeOrigin::none(), contract),
			sp_runtime::DispatchError::BadOrigin,
		);
	});
}

/// Pinning the core invariant of the simplified design: a "removed" (i.e.
/// `InboundOnly`) contract MUST be rejected by every outbound gate but MUST keep
/// passing every inbound gate, so users can always pull supply back from
/// `TeleportCheckingAccount` to AssetHub and from there get their tokens unlocked
/// home.
#[test]
fn inbound_only_contract_blocks_outbound_but_keeps_inbound_open() {
	new_test_ext().execute_with(|| {
		let contract = H160([0xd1; 20]);
		let asset = erc20_asset(contract.0, 1_000);
		let trusted = Location::new(1, [Junction::Parachain(1001)]);
		let ctx = XcmContext {
			origin: None,
			message_id: [0u8; 32],
			topic: None,
		};

		assert_ok!(Erc20XcmBridge::add_teleportable_erc20(
			RuntimeOrigin::root(),
			contract,
		));
		assert_ok!(Erc20XcmBridge::remove_teleportable_erc20(
			RuntimeOrigin::root(),
			contract,
		));
		assert_eq!(
			TeleportableErc20s::<Test>::get(&contract),
			Some(TeleportableErc20Status::InboundOnly),
		);

		// Outbound user gate (`XcmTeleportFilter`): rejects InboundOnly.
		assert!(!<IsTeleportableErc20<Test> as Contains<(
			Location,
			Vec<Asset>,
		)>>::contains(&(trusted.clone(), vec![asset.clone()])));

		// Outbound asset-transactor gate (`can_check_out`): rejects InboundOnly via
		// `MatchError::AssetNotHandled`, which the trait converts to
		// `XcmError::AssetNotFound`. That's the conventional "I don't handle this asset"
		// signal so the executor can still try the legacy reserve adapter behind us —
		// but never as a teleport.
		assert_eq!(
			Erc20TeleportTransactor::<Test>::can_check_out(&trusted, &asset, &ctx),
			Err(XcmError::AssetNotFound),
		);

		// Inbound trust gate (`IsTeleporter`): KEEPS admitting InboundOnly so locked
		// supply can come home indefinitely. This is the whole point of keeping the
		// whitelist entry around after `remove`.
		assert!(<IsTeleportableErc20<Test> as ContainsPair<
			Asset,
			Location,
		>>::contains(&asset, &trusted));

		// Inbound asset-transactor gate (`can_check_in`): KEEPS admitting InboundOnly.
		assert_ok!(Erc20TeleportTransactor::<Test>::can_check_in(
			&trusted, &asset, &ctx,
		));
	});
}

#[test]
fn is_teleportable_erc20_filter_pair_admits_only_trusted_location_with_whitelisted_asset() {
	new_test_ext().execute_with(|| {
		let contract = H160([6; 20]);
		let asset = erc20_asset(contract.0, 100);
		// `TeleportTrustedLocation` in `mock.rs` is `(1, [Parachain(1001)])`.
		let trusted = Location::new(1, [Junction::Parachain(1001)]);
		// Hostile siblings / arbitrary parachains and the relay must NEVER pass the gate
		// regardless of whitelist state — that's the security fix we're enforcing.
		let untrusted_sibling = Location::new(1, [Junction::Parachain(2042)]);
		let relay = Location::parent();

		// Not yet whitelisted: rejected for every location.
		assert!(!<IsTeleportableErc20<Test> as ContainsPair<
			Asset,
			Location,
		>>::contains(&asset, &trusted));
		assert!(!<IsTeleportableErc20<Test> as ContainsPair<
			Asset,
			Location,
		>>::contains(&asset, &untrusted_sibling));

		assert_ok!(Erc20XcmBridge::add_teleportable_erc20(
			RuntimeOrigin::root(),
			contract,
		));

		// Whitelisted + trusted location: admitted.
		assert!(<IsTeleportableErc20<Test> as ContainsPair<
			Asset,
			Location,
		>>::contains(&asset, &trusted));

		// Whitelisted but untrusted origin: rejected. This is the bug the location bind
		// closes — without it, any sibling para or the relay could present a whitelisted
		// ERC-20 in `ReceiveTeleportedAsset` and drain `TeleportCheckingAccount` via the
		// follow-up `DepositAsset`.
		assert!(!<IsTeleportableErc20<Test> as ContainsPair<
			Asset,
			Location,
		>>::contains(&asset, &untrusted_sibling));
		assert!(!<IsTeleportableErc20<Test> as ContainsPair<
			Asset,
			Location,
		>>::contains(&asset, &relay));

		// Non-ERC-20 asset (e.g. native) is never admitted, even from the trusted peer.
		let native = Asset::from((Location::parent(), 1u128));
		assert!(!<IsTeleportableErc20<Test> as ContainsPair<
			Asset,
			Location,
		>>::contains(&native, &trusted));
	});
}

#[test]
fn transactor_check_hooks_bind_to_trusted_location() {
	new_test_ext().execute_with(|| {
		let contract = H160([0x9a; 20]);
		let asset = erc20_asset(contract.0, 1_000);
		let trusted = Location::new(1, [Junction::Parachain(1001)]);
		let untrusted = Location::new(1, [Junction::Parachain(2042)]);
		let ctx = XcmContext {
			origin: None,
			message_id: [0u8; 32],
			topic: None,
		};

		assert_ok!(Erc20XcmBridge::add_teleportable_erc20(
			RuntimeOrigin::root(),
			contract,
		));

		// Asset is whitelisted but origin is untrusted: rejected with UntrustedTeleportLocation,
		// not AssetNotFound, so the executor cannot try the next transactor and accidentally
		// succeed. This is the second authoritative gate after `IsTeleporter`.
		assert_eq!(
			Erc20TeleportTransactor::<Test>::can_check_in(&untrusted, &asset, &ctx),
			Err(XcmError::UntrustedTeleportLocation),
		);
		assert_eq!(
			Erc20TeleportTransactor::<Test>::can_check_out(&untrusted, &asset, &ctx),
			Err(XcmError::UntrustedTeleportLocation),
		);

		// Same asset, trusted peer: passes both hooks.
		assert_ok!(Erc20TeleportTransactor::<Test>::can_check_in(
			&trusted, &asset, &ctx,
		));
		assert_ok!(Erc20TeleportTransactor::<Test>::can_check_out(
			&trusted, &asset, &ctx,
		));
	});
}

/// Cover the public read accessors `Pallet::is_teleportable_erc20` (admits both
/// states — used by inbound-side runtime callers) and `Pallet::is_active_teleportable_erc20`
/// (admits Active only — used by outbound-side runtime callers). They are the read-side
/// API surfaced to runtime code; pinning them here protects against a future refactor
/// flipping the semantics.
#[test]
fn pallet_helpers_track_storage_state() {
	new_test_ext().execute_with(|| {
		let contract = H160([0xb7; 20]);

		// Unknown: both helpers return false.
		assert!(!Erc20XcmBridge::is_teleportable_erc20(&contract));
		assert!(!Erc20XcmBridge::is_active_teleportable_erc20(&contract));

		// Active: both true.
		assert_ok!(Erc20XcmBridge::add_teleportable_erc20(
			RuntimeOrigin::root(),
			contract,
		));
		assert!(Erc20XcmBridge::is_teleportable_erc20(&contract));
		assert!(Erc20XcmBridge::is_active_teleportable_erc20(&contract));

		// InboundOnly: admitted (`is_teleportable_erc20` stays true so the inbound path
		// stays open) but no longer Active (`is_active_teleportable_erc20` flips false
		// so outbound is closed). This split is the whole reason the helpers exist.
		assert_ok!(Erc20XcmBridge::remove_teleportable_erc20(
			RuntimeOrigin::root(),
			contract,
		));
		assert!(Erc20XcmBridge::is_teleportable_erc20(&contract));
		assert!(!Erc20XcmBridge::is_active_teleportable_erc20(&contract));

		// Re-activation flips Active back on without losing admission.
		assert_ok!(Erc20XcmBridge::add_teleportable_erc20(
			RuntimeOrigin::root(),
			contract,
		));
		assert!(Erc20XcmBridge::is_teleportable_erc20(&contract));
		assert!(Erc20XcmBridge::is_active_teleportable_erc20(&contract));
	});
}

/// `IsTeleporter` (the `ContainsPair<Asset, Location>` impl) must keep admitting
/// `InboundOnly` contracts so the teleport-back path stays open after `remove`, but
/// only when the message actually came from `TeleportTrustedLocation`. The trusted-
/// location bind is the security fix; pinning it for `InboundOnly` (not just `Active`)
/// closes the matrix.
#[test]
fn is_teleporter_pair_admits_inbound_only_only_from_trusted_location() {
	new_test_ext().execute_with(|| {
		let contract = H160([0xab; 20]);
		let asset = erc20_asset(contract.0, 1_000);
		let trusted = Location::new(1, [Junction::Parachain(1001)]);
		let untrusted_sibling = Location::new(1, [Junction::Parachain(2042)]);
		let relay = Location::parent();

		// Move the contract into InboundOnly.
		assert_ok!(Erc20XcmBridge::add_teleportable_erc20(
			RuntimeOrigin::root(),
			contract,
		));
		assert_ok!(Erc20XcmBridge::remove_teleportable_erc20(
			RuntimeOrigin::root(),
			contract,
		));
		assert_eq!(
			TeleportableErc20s::<Test>::get(&contract),
			Some(TeleportableErc20Status::InboundOnly),
		);

		// Trusted peer: STILL admitted in `InboundOnly` so users can pull supply home.
		assert!(<IsTeleportableErc20<Test> as ContainsPair<
			Asset,
			Location,
		>>::contains(&asset, &trusted));

		// Untrusted peers: rejected. `InboundOnly` does NOT widen the trust surface;
		// the location bind is enforced regardless of state.
		assert!(!<IsTeleportableErc20<Test> as ContainsPair<
			Asset,
			Location,
		>>::contains(&asset, &untrusted_sibling));
		assert!(!<IsTeleportableErc20<Test> as ContainsPair<
			Asset,
			Location,
		>>::contains(&asset, &relay));
	});
}

/// Outbound transactor methods (`withdraw_asset`, `internal_transfer_asset`) gate on
/// `Active` via `match_active`. For `InboundOnly` and unknown contracts, the gate must
/// trip BEFORE any EVM call and surface as `XcmError::AssetNotFound` (the conventional
/// "I don't handle this asset" signal that lets the legacy reserve adapter, placed
/// after this transactor in `AssetTransactors`, take over). This is what makes the
/// "remove just blocks outbound" claim true at the asset-transactor layer (not just at
/// the upstream `XcmTeleportFilter`).
#[test]
fn transactor_outbound_methods_reject_unhandled_assets() {
	new_test_ext().execute_with(|| {
		let inbound_only = H160([0xc0; 20]);
		let unknown = H160([0xc1; 20]);
		let any_loc = Location::here();
		let other_loc = Location::parent();
		let ctx = XcmContext {
			origin: None,
			message_id: [0u8; 32],
			topic: None,
		};

		// One contract in `InboundOnly`, one not in the whitelist at all.
		assert_ok!(Erc20XcmBridge::add_teleportable_erc20(
			RuntimeOrigin::root(),
			inbound_only,
		));
		assert_ok!(Erc20XcmBridge::remove_teleportable_erc20(
			RuntimeOrigin::root(),
			inbound_only,
		));

		let inbound_asset = erc20_asset(inbound_only.0, 1_000);
		let unknown_asset = erc20_asset(unknown.0, 1_000);

		// `withdraw_asset` must short-circuit on both, BEFORE touching the EVM.
		assert_eq!(
			Erc20TeleportTransactor::<Test>::withdraw_asset(&inbound_asset, &any_loc, None,).err(),
			Some(XcmError::AssetNotFound),
		);
		assert_eq!(
			Erc20TeleportTransactor::<Test>::withdraw_asset(&unknown_asset, &any_loc, None,).err(),
			Some(XcmError::AssetNotFound),
		);

		// Same for `internal_transfer_asset`.
		assert_eq!(
			Erc20TeleportTransactor::<Test>::internal_transfer_asset(
				&inbound_asset,
				&any_loc,
				&other_loc,
				&ctx,
			)
			.err(),
			Some(XcmError::AssetNotFound),
		);
		assert_eq!(
			Erc20TeleportTransactor::<Test>::internal_transfer_asset(
				&unknown_asset,
				&any_loc,
				&other_loc,
				&ctx,
			)
			.err(),
			Some(XcmError::AssetNotFound),
		);
	});
}

/// Inbound transactor entry (`deposit_asset`) gates on "any whitelist entry" via
/// `match_admitted`. Non-whitelisted contracts must short-circuit to `AssetNotFound`
/// pre-EVM so the legacy reserve adapter can still claim them. Active and InboundOnly
/// contracts proceed past this gate into the EVM transfer (covered end-to-end in the
/// moonbase integration / zombienet flows; not exercised here because the unit-test
/// EVM has no contract bytecode deployed at the contract address).
#[test]
fn transactor_deposit_asset_rejects_non_whitelisted() {
	new_test_ext().execute_with(|| {
		let unknown = H160([0xd2; 20]);
		let beneficiary = Location::here();
		let unknown_asset = erc20_asset(unknown.0, 1_000);

		assert_eq!(
			Erc20TeleportTransactor::<Test>::deposit_asset(&unknown_asset, &beneficiary, None,)
				.err(),
			Some(XcmError::AssetNotFound),
		);
	});
}

#[test]
fn is_teleportable_erc20_filter_extrinsic_requires_all_whitelisted() {
	new_test_ext().execute_with(|| {
		let whitelisted = H160([7; 20]);
		let non_whitelisted = H160([8; 20]);
		let dest = Location::new(1, [Junction::Parachain(1001)]);

		assert_ok!(Erc20XcmBridge::add_teleportable_erc20(
			RuntimeOrigin::root(),
			whitelisted,
		));

		// Empty asset list: rejected.
		let empty = (dest.clone(), Vec::<Asset>::new());
		assert!(!<IsTeleportableErc20<Test> as Contains<(
			Location,
			Vec<Asset>,
		)>>::contains(&empty));

		// All whitelisted: admitted.
		let all_ok = (
			dest.clone(),
			vec![erc20_asset(whitelisted.0, 1), erc20_asset(whitelisted.0, 2)],
		);
		assert!(<IsTeleportableErc20<Test> as Contains<(
			Location,
			Vec<Asset>,
		)>>::contains(&all_ok));

		// Mixed: rejected.
		let mixed = (
			dest,
			vec![
				erc20_asset(whitelisted.0, 1),
				erc20_asset(non_whitelisted.0, 1),
			],
		);
		assert!(!<IsTeleportableErc20<Test> as Contains<(
			Location,
			Vec<Asset>,
		)>>::contains(&mixed));
	});
}

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
