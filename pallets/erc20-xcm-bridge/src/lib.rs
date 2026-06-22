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

//! Pallet that allow to transact erc20 tokens through xcm directly.
//!
//! It hosts two distinct asset transactors:
//!
//! - [`Pallet`] (the original "reserve mode" transactor): keeps a single EVM transfer per XCM
//!   message by deferring the actual ERC-20 movement until the in-message `DepositAsset`
//!   instruction. Suitable for reserve-backed transfers.
//! - [`Erc20TeleportTransactor`] (new): treats the ERC-20 supply on Moonbeam as locked in a
//!   runtime-controlled checking address whenever the contract is in the
//!   [`TeleportableErc20s`] whitelist. Performs an actual EVM transfer in `withdraw_asset` /
//!   `deposit_asset`. This is what backs `pallet_xcm::limited_teleport_assets` for the
//!   whitelisted ERC-20s, while the legacy reserve adapter continues to handle every other
//!   ERC-20.
//!
//! Native assets (DEV / GLMR / MOVR) are intentionally not handled here and remain
//! non-teleportable.
//!
//! ## Whitelist lifecycle
//!
//! Each whitelisted contract has a [`TeleportableErc20Status`] and a [`LockedSupply`]
//! counter. The counter is the canonical "outstanding obligation" the runtime owes
//! holders of the foreign-asset twin on the trusted counterparty
//! ([`Config::TeleportTrustedLocation`]); it is maintained in lockstep with every
//! teleport leg, so it never requires querying the contract.
//!
//! - **`Registered`**: admin just inserted the contract; no teleport leg has executed
//!   yet in the current lifecycle. Outbound is open so the first user teleport-out can
//!   actually run; the first successful leg auto-promotes to `Active`.
//! - **`Active`**: at least one teleport leg has executed. `LockedSupply` may be zero
//!   or positive — its counter routinely transits through zero between in/out flows
//!   without that meaning the contract is no longer in use.
//! - **`Deregistered`**: admin closed outbound while supply was still locked. New
//!   outbound teleports are refused; inbound teleports keep unwinding the counter so
//!   users can pull their tokens home.
//!
//! [`Pallet::remove_teleportable_erc20`] is dual-purpose:
//!
//! - When `LockedSupply == 0`, it deletes the entry outright. The call is
//!   **permissionless** ONLY for `Deregistered` (admin already opted into wind-down,
//!   the public sweep just finalizes that intent once the obligation is fully
//!   discharged). `Registered` and `Active` are **admin-only**: admin-only on
//!   `Registered` so a fresh add can't be undone before any flow happens, and
//!   admin-only on `Active` so a third party can't snipe a live operational entry
//!   the moment its counter momentarily hits zero between flows.
//! - When `LockedSupply > 0`, it requires admin and flips the entry to `Deregistered`.
//!
//! [`Pallet::force_remove_teleportable_erc20`] is the admin escape hatch that deletes
//! the entry regardless of state and counter, emitting an event that records both for
//! auditability.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

mod erc20_matcher;
mod erc20_trap;
mod errors;
mod xcm_holding_ext;

use frame_support::pallet;

pub use erc20_trap::AssetTrapWrapper;
pub use pallet::*;
pub use xcm_holding_ext::XcmExecutorWrapper;

#[pallet]
pub mod pallet {

	use crate::erc20_matcher::*;
	use crate::errors::*;
	use crate::xcm_holding_ext::*;
	use core::marker::PhantomData;
	use ethereum_types::BigEndianHash;
	use fp_evm::{ExitReason, ExitSucceed};
	use frame_support::pallet_prelude::*;
	use frame_support::traits::{Contains, ContainsPair, EnsureOrigin};
	use frame_system::pallet_prelude::*;
	use pallet_evm::{GasWeightMapping, Runner};
	use sp_core::{H160, H256, U256};
	use sp_std::vec::Vec;
	use xcm::latest::{
		Asset, AssetId, Error as XcmError, Junction, Location, Result as XcmResult, XcmContext,
	};
	use xcm_executor::traits::ConvertLocation;
	use xcm_executor::traits::{Error as MatchError, MatchesFungibles};
	use xcm_executor::AssetsInHolding;

	const ERC20_TRANSFER_CALL_DATA_SIZE: usize = 4 + 32 + 32; // selector + from + amount
	const ERC20_TRANSFER_SELECTOR: [u8; 4] = [0xa9, 0x05, 0x9c, 0xbb];

	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::config]
	pub trait Config:
		frame_system::Config<RuntimeEvent: From<Event<Self>>> + pallet_evm::Config
	{
		/// How a Location is converted into an EVM `H160` address (used to match accounts and
		/// the destination chain's sovereign account).
		type AccountIdConverter: ConvertLocation<H160>;
		/// XCM Location prefix used to identify ERC-20 multilocations on this chain (typically
		/// the pallet location, e.g. `(0, [PalletInstance(<this pallet index>)])`).
		type Erc20MultilocationPrefix: Get<Location>;
		/// Default gas limit used for ERC-20 transfers when the asset doesn't override it.
		type Erc20TransferGasLimit: Get<u64>;
		/// EVM runner used to execute ERC-20 calls (transfer / transferFrom).
		type EvmRunner: Runner<Self>;
		/// Origin that can edit the [`TeleportableErc20s`] whitelist.
		type TeleportAdminOrigin: EnsureOrigin<Self::RuntimeOrigin>;
		/// EVM `H160` address used to lock ERC-20 supply when teleporting whitelisted contracts.
		/// Must be controlled by this runtime; should not be a regular user account.
		type TeleportCheckingAccount: Get<H160>;
		/// The single counterparty location (typically Asset Hub) that this runtime trusts to
		/// teleport whitelisted ERC-20s in/out. Used by [`IsTeleportableErc20`] to bind the
		/// asset whitelist to a fixed peer for `xcm_executor::Config::IsTeleporter`, so an
		/// inbound `ReceiveTeleportedAsset` carrying a whitelisted contract is only accepted
		/// when the message origin equals this location. Outbound `pallet_xcm::limited_teleport_assets`
		/// is similarly restricted: `dest` must equal this location.
		type TeleportTrustedLocation: Get<Location>;
	}

	/// Lifecycle status of a contract in the teleport whitelist. See the module-level
	/// docs for the state diagram.
	///
	/// - `Registered`: admin just inserted the contract via
	///   [`Pallet::add_teleportable_erc20`]. No teleport leg has executed yet in this
	///   lifecycle; [`LockedSupply`] is `0`. Both directions are admitted by the gates
	///   so the first leg can actually run.
	/// - `Active`: at least one teleport leg (in or out) has executed. The transition
	///   from `Registered` is automatic and happens inside the asset transactor when
	///   the EVM transfer succeeds. `LockedSupply` may be `0` (e.g. inbound legs
	///   unwound everything) or positive — the counter routinely transits through
	///   zero between in/out flows. While in this state the entry only leaves the
	///   whitelist via an admin call; third parties cannot purge it.
	/// - `Deregistered`: admin called [`Pallet::remove_teleportable_erc20`] while
	///   `LockedSupply > 0`. Outbound is closed in every gate, but inbound stays open
	///   so users can teleport their twin back from
	///   [`Config::TeleportTrustedLocation`] and decrement the counter. Once the
	///   counter reaches `0`, anyone can finalize the wind-down by calling
	///   `remove_teleportable_erc20` again — this is the only state from which a
	///   permissionless purge is allowed.
	#[derive(
		Encode,
		Decode,
		DecodeWithMemTracking,
		MaxEncodedLen,
		TypeInfo,
		Clone,
		Copy,
		PartialEq,
		Eq,
		Debug,
	)]
	pub enum TeleportableErc20Status {
		Registered,
		Active,
		Deregistered,
	}

	/// Whitelist of ERC-20 contracts that are eligible for teleport semantics, keyed by
	/// EVM address. The stored variant decides what the per-message gates admit; see
	/// [`TeleportableErc20Status`] and the module-level state diagram.
	///
	/// Storage entries are removed under exactly two paths:
	/// - [`Pallet::remove_teleportable_erc20`] when [`LockedSupply`] is zero (admin
	///   for `Registered` and `Active`, permissionless only for `Deregistered`), and
	/// - [`Pallet::force_remove_teleportable_erc20`] (admin escape hatch).
	///
	/// While the entry is present, this runtime locks the contract's supply in
	/// `TeleportCheckingAccount` whenever it is sent cross-chain via XCM, and any
	/// counterparty that registered the asset's foreign-asset twin with
	/// `teleportable: true` and `reserve = (1, [Parachain(<this para>)])` will accept
	/// teleport semantics for it.
	#[pallet::storage]
	pub type TeleportableErc20s<T: Config> =
		StorageMap<_, Twox64Concat, H160, TeleportableErc20Status, OptionQuery>;

	/// Per-contract counter of ERC-20 supply currently locked in
	/// [`Config::TeleportCheckingAccount`] as a result of teleport-out legs handled by
	/// [`Erc20TeleportTransactor`]. Maintained in lockstep with every teleport leg:
	///
	/// - `withdraw_asset` (outbound lock leg): saturating-add `amount`.
	/// - `deposit_asset` (inbound unlock leg): saturating-sub `amount`.
	/// - `internal_transfer_asset`: untouched (same-chain hop, never moves the checking
	///   account).
	///
	/// The counter is **the** authoritative signal for "any outstanding obligation?".
	/// When it is zero, [`Pallet::remove_teleportable_erc20`] purges the storage entry
	/// outright (admin-only from `Registered`/`Active`, permissionless from
	/// `Deregistered`); when it is non-zero, removal flips the contract to
	/// `Deregistered` so users on the trusted counterparty can keep teleporting their
	/// twin back and decrementing the counter.
	///
	/// Drift modes:
	/// - **Donations.** A direct `ERC20.transfer(checking, amount)` outside XCM raises
	///   the on-chain balance without changing this counter. Such donated supply has
	///   no foreign-asset twin to match, so ignoring it is correct: the counter
	///   intentionally tracks the obligation, not the wallet balance.
	/// - **Off-pallet drains.** A contract that drains the checking account behind our
	///   back lowers the on-chain balance without changing this counter. The counter
	///   then over-reports the obligation, which conservatively keeps the entry
	///   non-purgeable until [`Pallet::force_remove_teleportable_erc20`].
	/// - **Inbound on a fresh `Registered` entry.** If the counterparty already had a
	///   twin balance before the first outbound leg (e.g. seeded externally), an
	///   inbound leg saturating-subs against `0`. The deposit still proceeds — the
	///   counter is a lower bound on the obligation, never a hard cap on inbound.
	#[pallet::storage]
	pub type LockedSupply<T: Config> = StorageMap<_, Twox64Concat, H160, U256, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// An ERC-20 contract was inserted into the teleport whitelist as `Registered`.
		/// Emitted both on a fresh add (`(none) → Registered`) and on revival from
		/// `Deregistered → Registered`; both go through
		/// [`Pallet::add_teleportable_erc20`].
		TeleportableErc20Added { contract: H160 },
		/// An ERC-20 contract was promoted `Registered → Active` by the asset
		/// transactor after its first successful teleport leg in this lifecycle. Not an
		/// admin event; it fires from inside `withdraw_asset`/`deposit_asset`/
		/// `internal_transfer_asset`.
		TeleportableErc20Activated { contract: H160 },
		/// An ERC-20 contract was moved from `Registered`/`Active` to `Deregistered`
		/// because [`LockedSupply`] was non-zero when
		/// [`Pallet::remove_teleportable_erc20`] was called. New outbound teleports for
		/// it are now refused, but inbound teleports from
		/// [`Config::TeleportTrustedLocation`] continue to unwind the counter back to
		/// zero.
		TeleportableErc20Removed { contract: H160 },
		/// The whitelist entry for the given contract was deleted from storage via
		/// [`Pallet::remove_teleportable_erc20`] because [`LockedSupply`] reached zero.
		/// The contract no longer participates in teleport semantics.
		TeleportableErc20Purged { contract: H160 },
		/// [`Pallet::force_remove_teleportable_erc20`] deleted the entry regardless of
		/// state. Emits the state and the `LockedSupply` counter at the time of the
		/// call so any orphaned obligation is auditable from chain events.
		TeleportableErc20ForceRemoved {
			contract: H160,
			status_before: TeleportableErc20Status,
			locked_supply: U256,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The contract is already in the teleport whitelist as `Registered` or
		/// `Active`. ([`Pallet::add_teleportable_erc20`] on a `Deregistered` contract
		/// is the legal revival path and is **not** considered duplicate.)
		Erc20AlreadyTeleportable,
		/// The contract is not in the teleport whitelist (either it was never added or
		/// it was already purged via [`Pallet::remove_teleportable_erc20`] /
		/// [`Pallet::force_remove_teleportable_erc20`]).
		Erc20NotTeleportable,
		/// The contract is already `Deregistered` and [`LockedSupply`] is still
		/// non-zero, so [`Pallet::remove_teleportable_erc20`] is a no-op. Wait for
		/// users to teleport supply back (which will eventually drive the counter to
		/// zero, allowing the entry to be swept), or use
		/// [`Pallet::force_remove_teleportable_erc20`] to forfeit the obligation.
		Erc20AlreadyRemoved,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Insert (or revive) an ERC-20 contract in the teleport whitelist as
		/// `Registered`. Callable only by [`Config::TeleportAdminOrigin`].
		///
		/// **Operator warning — this is NOT a narrow "teleport-on" flag.** Once a
		/// contract is in [`TeleportableErc20s`], the runtime's asset transactor
		/// tuple routes EVERY XCM `TransactAsset` operation for it (`withdraw_asset`,
		/// `deposit_asset`, `internal_transfer_asset`) through
		/// [`Erc20TeleportTransactor`] before falling back to the legacy reserve
		/// adapter. The transactor cannot tell teleport-driven flows apart from
		/// reserve-driven or `pallet_xcm::execute`-driven flows, so the
		/// lock/unlock-against-[`Config::TeleportCheckingAccount`] semantics apply
		/// uniformly. Whitelisted ERC-20s used in the wrong path
		/// (e.g. `pallet_xcm::limited_reserve_transfer_assets` to the trusted
		/// counterparty) are rejected with `Filtered` by `pallet_xcm` itself, but
		/// for non-counterparty destinations the failure surface lives inside
		/// `xcm-executor`. **Only whitelist contracts intended exclusively for the
		/// trusted teleport flow** (per [`Config::TeleportTrustedLocation`]).
		///
		/// Behaviour by current state:
		/// - **No entry** (`(none) → Registered`): fresh add. The first subsequent
		///   teleport leg auto-promotes to `Active`.
		/// - **`Deregistered` → `Registered`**: revival. Any [`LockedSupply`] from the
		///   pre-deregistration lifetime is preserved verbatim; the next teleport leg
		///   auto-promotes to `Active`. Use this to undo an in-error
		///   `remove_teleportable_erc20`, or to reopen outbound after a planned
		///   maintenance window.
		/// - **`Registered`**: rejected with [`Error::Erc20AlreadyTeleportable`] — the
		///   entry is already in the pre-flow state, no-op.
		/// - **`Active`**: rejected with [`Error::Erc20AlreadyTeleportable`] — the
		///   entry is already fully whitelisted, no-op.
		///
		/// Emits [`Event::TeleportableErc20Added`] on success.
		#[pallet::call_index(0)]
		#[pallet::weight(Weight::from_parts(15_000_000, 0).saturating_add(T::DbWeight::get().reads_writes(1, 1)))]
		pub fn add_teleportable_erc20(origin: OriginFor<T>, contract: H160) -> DispatchResult {
			T::TeleportAdminOrigin::ensure_origin(origin)?;
			match TeleportableErc20s::<T>::get(&contract) {
				Some(TeleportableErc20Status::Registered)
				| Some(TeleportableErc20Status::Active) => {
					return Err(Error::<T>::Erc20AlreadyTeleportable.into())
				}
				// `(none)` → fresh add, `Deregistered` → revival. Both land on
				// `Registered`; the next teleport leg auto-promotes to `Active`.
				None | Some(TeleportableErc20Status::Deregistered) => {
					TeleportableErc20s::<T>::insert(&contract, TeleportableErc20Status::Registered);
					Self::deposit_event(Event::TeleportableErc20Added { contract });
					Ok(())
				}
			}
		}

		/// Dual-purpose retirement extrinsic for a whitelisted ERC-20.
		///
		/// Behaviour depends on [`LockedSupply`] at call time:
		///
		/// - **`LockedSupply == 0`** (no outstanding obligation): the entry is purged
		///   from storage outright. `LockedSupply` is also removed.
		///   - Origin: [`Config::TeleportAdminOrigin`] is required when the current
		///     status is `Registered` or `Active`. Admin-only on `Registered` so a
		///     freshly added entry can't be erased by a third party racing the
		///     admin's intent. Admin-only on `Active` so a third party cannot snipe
		///     a live operational entry the moment its counter momentarily hits zero
		///     between in/out flows — the operator stays in control of when an
		///     active contract leaves the whitelist.
		///   - Origin: any **signed** origin (or root) is accepted when the current
		///     status is `Deregistered` — admin already opted into wind-down by
		///     flipping the entry, so the public sweep just finalizes that intent
		///     once the obligation is fully discharged. This is the "permissionless
		///     purge" path and is what gives a deregistered entry a natural
		///     terminus without further admin intervention.
		///   - Emits [`Event::TeleportableErc20Purged`].
		///
		/// - **`LockedSupply > 0`** (outstanding obligation): the entry is flipped to
		///   `Deregistered`. New outbound teleports are refused; inbound teleports keep
		///   unwinding the counter. Once the counter reaches zero, the entry can be
		///   swept permissionlessly via this same extrinsic.
		///   - Origin: [`Config::TeleportAdminOrigin`] is required.
		///   - Already-`Deregistered` entries return [`Error::Erc20AlreadyRemoved`].
		///   - Emits [`Event::TeleportableErc20Removed`].
		///
		/// Errors:
		/// - [`Error::Erc20NotTeleportable`] when the contract has no whitelist entry.
		/// - [`Error::Erc20AlreadyRemoved`] when the contract is already
		///   `Deregistered` and `LockedSupply > 0`.
		#[pallet::call_index(1)]
		#[pallet::weight(Weight::from_parts(15_000_000, 0).saturating_add(T::DbWeight::get().reads_writes(2, 2)))]
		pub fn remove_teleportable_erc20(origin: OriginFor<T>, contract: H160) -> DispatchResult {
			let status =
				TeleportableErc20s::<T>::get(&contract).ok_or(Error::<T>::Erc20NotTeleportable)?;
			let count = LockedSupply::<T>::get(&contract);

			if count.is_zero() {
				// Purge path. Permissionless ONLY for `Deregistered`: the admin has
				// already opted into wind-down by flipping the entry, so the public
				// sweep just finalizes that intent once the obligation is fully
				// discharged. `Registered` and `Active` are admin-only — a live
				// `Active` contract routinely transits through `count == 0` between
				// flows, and admitting third parties to that purge would let anyone
				// snipe operational entries the moment the counter hits zero,
				// forcing the operator to re-add. `Registered` is admin-only too so
				// a fresh add can't be undone by anyone but the admin.
				match status {
					TeleportableErc20Status::Registered | TeleportableErc20Status::Active => {
						T::TeleportAdminOrigin::ensure_origin(origin)?;
					}
					TeleportableErc20Status::Deregistered => {
						// Permissionless: any signed origin passes, and admin (root)
						// is the strict superset. Unsigned (`None`) is rejected with
						// `BadOrigin` to prevent free unsigned calls.
						let _ = ensure_signed_or_root(origin)?;
					}
				}
				TeleportableErc20s::<T>::remove(&contract);
				LockedSupply::<T>::remove(&contract);
				Self::deposit_event(Event::TeleportableErc20Purged { contract });
				return Ok(());
			}

			// `count > 0`: state-change path. Always admin.
			T::TeleportAdminOrigin::ensure_origin(origin)?;
			match status {
				TeleportableErc20Status::Deregistered => {
					return Err(Error::<T>::Erc20AlreadyRemoved.into())
				}
				// `Registered + count > 0` is unreachable in practice (the first leg
				// promotes to `Active`), but treat it like `Active` for defensive
				// correctness.
				TeleportableErc20Status::Registered | TeleportableErc20Status::Active => {}
			}
			TeleportableErc20s::<T>::insert(&contract, TeleportableErc20Status::Deregistered);
			Self::deposit_event(Event::TeleportableErc20Removed { contract });
			Ok(())
		}

		/// Admin escape hatch: delete the whitelist entry and its [`LockedSupply`]
		/// counter regardless of state and counter value. Callable only by
		/// [`Config::TeleportAdminOrigin`].
		///
		/// This is a destructive operation. If `LockedSupply > 0` at call time, any
		/// supply still parked in [`Config::TeleportCheckingAccount`] is effectively
		/// stranded from this pallet's bookkeeping perspective — inbound teleport-back
		/// messages will be rejected by the gates as the entry is gone, and users that
		/// still hold the foreign-asset twin on the trusted counterparty cannot redeem
		/// it through this pallet without a subsequent
		/// [`Pallet::add_teleportable_erc20`] revival.
		///
		/// Emits [`Event::TeleportableErc20ForceRemoved`] with `status_before` and
		/// `locked_supply` so the act is auditable on-chain.
		///
		/// Errors:
		/// - [`Error::Erc20NotTeleportable`] when the contract has no whitelist entry.
		#[pallet::call_index(2)]
		#[pallet::weight(Weight::from_parts(15_000_000, 0).saturating_add(T::DbWeight::get().reads_writes(2, 2)))]
		pub fn force_remove_teleportable_erc20(
			origin: OriginFor<T>,
			contract: H160,
		) -> DispatchResult {
			T::TeleportAdminOrigin::ensure_origin(origin)?;
			let status =
				TeleportableErc20s::<T>::get(&contract).ok_or(Error::<T>::Erc20NotTeleportable)?;
			let locked_supply = LockedSupply::<T>::get(&contract);
			TeleportableErc20s::<T>::remove(&contract);
			LockedSupply::<T>::remove(&contract);
			Self::deposit_event(Event::TeleportableErc20ForceRemoved {
				contract,
				status_before: status,
				locked_supply,
			});
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn is_erc20_asset(asset: &Asset) -> bool {
			Erc20Matcher::<T::Erc20MultilocationPrefix>::is_erc20_asset(asset)
		}
		/// Whether the given ERC-20 contract has any whitelist entry — `Registered`,
		/// `Active`, or `Deregistered`. Used by inbound gates (`IsTeleporter`,
		/// `can_check_in`, `deposit_asset`) so that supply locked in
		/// `TeleportCheckingAccount` can always come home, even after the contract has
		/// been moved to `Deregistered`.
		pub fn is_teleportable_erc20(contract: &H160) -> bool {
			TeleportableErc20s::<T>::contains_key(contract)
		}
		/// Whether the given ERC-20 contract is currently outbound-eligible
		/// (`Registered` or `Active`). Used by outbound gates (`withdraw_asset`,
		/// `can_check_out`, `internal_transfer_asset`, `XcmTeleportFilter`).
		/// `Deregistered` returns false — that's what disables new outbound locks while
		/// keeping the return path open.
		pub fn is_outbound_eligible_erc20(contract: &H160) -> bool {
			matches!(
				TeleportableErc20s::<T>::get(contract),
				Some(TeleportableErc20Status::Registered) | Some(TeleportableErc20Status::Active),
			)
		}
		/// Legacy reserve-mode transactor must not handle contracts delegated to
		/// [`Erc20TeleportTransactor`]. Prevents split-brain withdraw/deposit routing
		/// across the `AssetTransactors` tuple.
		fn legacy_transactor_rejects_teleportable(contract: &H160) -> Result<(), XcmError> {
			if TeleportableErc20s::<T>::contains_key(contract) {
				return Err(XcmError::AssetNotFound);
			}
			Ok(())
		}
		/// Promote a `Registered` entry to `Active`. Called from inside the asset
		/// transactor's storage layer immediately after a successful EVM transfer leg.
		/// Idempotent: a no-op if the contract is already `Active` or has been moved
		/// to a different state. Emits [`Event::TeleportableErc20Activated`] only on
		/// the actual `Registered → Active` transition.
		fn promote_registered_to_active(contract: &H160) {
			if matches!(
				TeleportableErc20s::<T>::get(contract),
				Some(TeleportableErc20Status::Registered),
			) {
				TeleportableErc20s::<T>::insert(contract, TeleportableErc20Status::Active);
				Self::deposit_event(Event::TeleportableErc20Activated {
					contract: *contract,
				});
			}
		}
		pub fn gas_limit_of_erc20_transfer(asset_id: &AssetId) -> u64 {
			let location = &asset_id.0;
			if let Some(Junction::GeneralKey {
				length: _,
				ref data,
			}) = location.interior().into_iter().next_back()
			{
				// As GeneralKey definition might change in future versions of XCM, this is meant
				// to throw a compile error as a warning that data type has changed.
				// If that happens, a new check is needed to ensure that data has at least 18
				// bytes (size of b"gas_limit:" + u64)
				let data: &[u8; 32] = &data;
				if let Ok(content) = core::str::from_utf8(&data[0..10]) {
					if content == "gas_limit:" {
						let mut bytes: [u8; 8] = Default::default();
						bytes.copy_from_slice(&data[10..18]);
						return u64::from_le_bytes(bytes);
					}
				}
			}
			T::Erc20TransferGasLimit::get()
		}
		pub fn weight_of_erc20_transfer(asset_id: &AssetId) -> Weight {
			T::GasWeightMapping::gas_to_weight(Self::gas_limit_of_erc20_transfer(asset_id), true)
		}
		/// Worst-case weight of an ERC-20 transfer when the concrete asset (and thus any
		/// per-asset `gas_limit:` override) is not known at weighing time — e.g. the flat
		/// `pallet_xcm::teleport_assets` dispatch weight, which charges a fixed amount but can
		/// drive a real EVM transfer during local XCM execution. Derived from the default
		/// `Erc20TransferGasLimit`; per-asset overrides above that bound are a governance
		/// concern (contracts are gated by `TeleportableErc20s` registration).
		pub fn worst_case_erc20_transfer_weight() -> Weight {
			T::GasWeightMapping::gas_to_weight(T::Erc20TransferGasLimit::get(), true)
		}
		pub(crate) fn erc20_transfer(
			erc20_contract_address: H160,
			from: H160,
			to: H160,
			amount: U256,
			gas_limit: u64,
		) -> Result<(), Erc20TransferError> {
			let mut input = Vec::with_capacity(ERC20_TRANSFER_CALL_DATA_SIZE);
			// ERC20.transfer method hash
			input.extend_from_slice(&ERC20_TRANSFER_SELECTOR);
			// append receiver address
			input.extend_from_slice(H256::from(to).as_bytes());
			// append amount to be transferred
			input.extend_from_slice(H256::from_uint(&amount).as_bytes());

			let weight_limit: Weight = T::GasWeightMapping::gas_to_weight(gas_limit, true);

			let exec_info = T::EvmRunner::call(
				from,
				erc20_contract_address,
				input,
				U256::default(),
				gas_limit,
				None,
				None,
				None,
				Default::default(),
				Default::default(),
				false,
				false,
				Some(weight_limit),
				Some(0),
				&<T as pallet_evm::Config>::config(),
			)
			.map_err(|_| Erc20TransferError::EvmCallFail)?;

			ensure!(
				matches!(
					exec_info.exit_reason,
					ExitReason::Succeed(ExitSucceed::Returned | ExitSucceed::Stopped)
				),
				Erc20TransferError::ContractTransferFail
			);

			// return value is true.
			let bytes: [u8; 32] = U256::from(1).to_big_endian();

			// Check return value to make sure not calling on empty contracts.
			ensure!(
				!exec_info.value.is_empty() && exec_info.value == bytes,
				Erc20TransferError::ContractReturnInvalidValue
			);

			Ok(())
		}
	}

	impl<T: Config> xcm_executor::traits::TransactAsset for Pallet<T> {
		// For optimization reasons, the asset we want to deposit has not really been withdrawn,
		// we have just traced from which account it should have been withdrawn.
		// So we will retrieve these information and make the transfer from the origin account.
		fn deposit_asset(what: &Asset, who: &Location, _context: Option<&XcmContext>) -> XcmResult {
			let (contract_address, amount) =
				Erc20Matcher::<T::Erc20MultilocationPrefix>::matches_fungibles(what)?;
			Self::legacy_transactor_rejects_teleportable(&contract_address)?;

			let beneficiary = T::AccountIdConverter::convert_location(who)
				.ok_or(MatchError::AccountIdConversionFailed)?;

			let gas_limit = Self::gas_limit_of_erc20_transfer(&what.id);

			// Get the global context to recover accounts origins.
			XcmHoldingErc20sOrigins::with(|erc20s_origins| {
				match erc20s_origins.drain(contract_address, amount) {
					// We perform the evm transfers in a storage transaction to ensure that if one
					// of them fails all the changes of the previous evm calls are rolled back.
					Ok(tokens_to_transfer) => frame_support::storage::with_storage_layer(|| {
						tokens_to_transfer
							.into_iter()
							.try_for_each(|(from, subamount)| {
								Self::erc20_transfer(
									contract_address,
									from,
									beneficiary,
									subamount,
									gas_limit,
								)
							})
					})
					.map_err(Into::into),
					Err(DrainError::AssetNotFound) => Err(XcmError::AssetNotFound),
					Err(DrainError::NotEnoughFounds) => Err(XcmError::FailedToTransactAsset(
						"not enough founds in xcm holding",
					)),
					Err(DrainError::SplitError) => Err(XcmError::FailedToTransactAsset(
						"SplitError: each withdrawal of erc20 tokens must be deposited at once",
					)),
				}
			})
			.ok_or(XcmError::FailedToTransactAsset(
				"missing erc20 executor context",
			))?
		}

		fn internal_transfer_asset(
			asset: &Asset,
			from: &Location,
			to: &Location,
			_context: &XcmContext,
		) -> Result<AssetsInHolding, XcmError> {
			let (contract_address, amount) =
				Erc20Matcher::<T::Erc20MultilocationPrefix>::matches_fungibles(asset)?;
			Self::legacy_transactor_rejects_teleportable(&contract_address)?;

			let from = T::AccountIdConverter::convert_location(from)
				.ok_or(MatchError::AccountIdConversionFailed)?;

			let to = T::AccountIdConverter::convert_location(to)
				.ok_or(MatchError::AccountIdConversionFailed)?;

			let gas_limit = Self::gas_limit_of_erc20_transfer(&asset.id);

			// We perform the evm transfers in a storage transaction to ensure that if it fail
			// any contract storage changes are rolled back.
			frame_support::storage::with_storage_layer(|| {
				Self::erc20_transfer(contract_address, from, to, amount, gas_limit)
			})?;

			Ok(asset.clone().into())
		}

		// Since we don't control the erc20 contract that manages the asset we want to withdraw,
		// we can't really withdraw this asset, we can only transfer it to another account.
		// It would be possible to transfer the asset to a dedicated account that would reflect
		// the content of the xcm holding, but this would imply to perform two evm calls instead of
		// one (1 to withdraw the asset and a second one to deposit it).
		// In order to perform only one evm call, we just trace the origin of the asset,
		// and then the transfer will only really be performed in the deposit instruction.
		fn withdraw_asset(
			what: &Asset,
			who: &Location,
			_context: Option<&XcmContext>,
		) -> Result<AssetsInHolding, XcmError> {
			let (contract_address, amount) =
				Erc20Matcher::<T::Erc20MultilocationPrefix>::matches_fungibles(what)?;
			Self::legacy_transactor_rejects_teleportable(&contract_address)?;
			let who = T::AccountIdConverter::convert_location(who)
				.ok_or(MatchError::AccountIdConversionFailed)?;

			XcmHoldingErc20sOrigins::with(|erc20s_origins| {
				erc20s_origins.insert(contract_address, who, amount)
			})
			.ok_or(XcmError::FailedToTransactAsset(
				"missing erc20 executor context",
			))?;

			Ok(what.clone().into())
		}
	}

	/// `TransactAsset` implementation for whitelisted ERC-20s. Performs real EVM
	/// transfers against `T::TeleportCheckingAccount` and maintains the [`LockedSupply`]
	/// counter and the [`TeleportableErc20Status`] lifecycle:
	///
	/// - `withdraw_asset(asset, who)` → `ERC20.transfer(who → checking)`, then
	///   `LockedSupply += amount`, then `Registered → Active` if applicable.
	/// - `deposit_asset(holding, beneficiary)` → `ERC20.transfer(checking → beneficiary)`,
	///   then `LockedSupply -=(saturating) amount`, then `Registered → Active` if
	///   applicable.
	/// - `internal_transfer_asset(asset, from, to)` → `ERC20.transfer(from → to)`
	///   (same-chain hop, never touches the checking account; only the status
	///   promotion runs).
	///
	/// All three legs are wrapped in `frame_support::storage::with_storage_layer` so
	/// the EVM transfer, counter mutation, and status promotion are atomic w.r.t. each
	/// other.
	///
	/// **Zero-amount fast path.** When the matcher resolves a whitelisted contract
	/// with `Fungible(0)`, all three legs short-circuit to `Ok(())` (or `Ok(holding)`
	/// for the outbound legs) **without** touching the EVM, the counter, or the
	/// lifecycle. This prevents spam-able zero-amount teleports from burning gas,
	/// writing the counter to its current value, and prematurely promoting
	/// `Registered → Active` (which would otherwise emit a spurious
	/// [`Event::TeleportableErc20Activated`] for a flow that moved nothing).
	///
	/// Non-whitelisted assets return `Err(AssetNotFound)` so the legacy `Pallet<T>`
	/// adapter (placed after this one in `AssetTransactors`) can handle them.
	/// `Deregistered` outbound legs return `FailedToTransactAsset` so the tuple does
	/// not fall through to legacy (see `legacy_transactor_rejects_teleportable`).
	pub struct Erc20TeleportTransactor<T>(PhantomData<T>);

	impl<T: Config> Erc20TeleportTransactor<T> {
		const DEREGISTERED_OUTBOUND_MSG: &'static str =
			"teleport outbound blocked for deregistered ERC-20";

		/// Decode an `Asset` to `(contract, amount)` only when the contract is
		/// outbound-eligible, i.e. its whitelist entry is `Registered` or `Active`.
		/// `Deregistered` contracts return a definitive error (no legacy fallthrough).
		fn match_outbound(asset: &Asset) -> Result<(H160, U256), XcmError> {
			let (contract, amount) =
				Erc20Matcher::<T::Erc20MultilocationPrefix>::matches_fungibles(asset)
					.map_err(XcmError::from)?;
			match TeleportableErc20s::<T>::get(&contract) {
				Some(TeleportableErc20Status::Registered)
				| Some(TeleportableErc20Status::Active) => Ok((contract, amount)),
				Some(TeleportableErc20Status::Deregistered) => Err(
					XcmError::FailedToTransactAsset(Self::DEREGISTERED_OUTBOUND_MSG),
				),
				None => Err(XcmError::AssetNotFound),
			}
		}

		/// Decode an `Asset` to `(contract, amount)` whenever the contract has any
		/// whitelist entry — `Registered`, `Active`, or `Deregistered`. Used by
		/// inbound gates so that supply already locked in `TeleportCheckingAccount` can
		/// always come home from `TeleportTrustedLocation`, even after the contract
		/// has been deregistered.
		fn match_admitted(asset: &Asset) -> Result<(H160, U256), MatchError> {
			let (contract, amount) =
				Erc20Matcher::<T::Erc20MultilocationPrefix>::matches_fungibles(asset)?;
			match TeleportableErc20s::<T>::get(&contract) {
				Some(_) => Ok((contract, amount)),
				None => Err(MatchError::AssetNotHandled),
			}
		}

		/// Returns `Ok` only when `peer` is the runtime's trusted teleport counterparty
		/// (typically Asset Hub). Used by `can_check_in`/`can_check_out` to refuse the
		/// transactor's own bookkeeping for any non-AH origin/destination, even if a
		/// (mis)configured `IsTeleporter` upstream tried to admit it. This is defense in
		/// depth: it keeps a single source of truth — `T::TeleportTrustedLocation` — for
		/// where teleports are allowed in/out of this runtime.
		fn ensure_trusted_peer(peer: &Location) -> XcmResult {
			if peer != &T::TeleportTrustedLocation::get() {
				return Err(XcmError::UntrustedTeleportLocation);
			}
			Ok(())
		}
	}

	impl<T: Config> xcm_executor::traits::TransactAsset for Erc20TeleportTransactor<T> {
		fn can_check_in(origin: &Location, what: &Asset, _context: &XcmContext) -> XcmResult {
			Self::ensure_trusted_peer(origin)?;
			// Inbound: admit any whitelisted entry so a `Deregistered` contract can
			// still unwind supply still locked in `TeleportCheckingAccount`.
			let _ = Self::match_admitted(what)?;
			Ok(())
		}

		fn check_in(_origin: &Location, _what: &Asset, _context: &XcmContext) {}

		fn can_check_out(dest: &Location, what: &Asset, _context: &XcmContext) -> XcmResult {
			Self::ensure_trusted_peer(dest)?;
			// Outbound: admit `Registered` (so the first leg can run) or `Active`.
			// `Deregistered` contracts must NOT lock new supply.
			let _ = Self::match_outbound(what)?;
			Ok(())
		}

		fn check_out(_dest: &Location, _what: &Asset, _context: &XcmContext) {}

		fn deposit_asset(what: &Asset, who: &Location, _context: Option<&XcmContext>) -> XcmResult {
			// Inbound side of the EVM transfer. Admit any whitelisted entry so the
			// teleport-back path stays open indefinitely after
			// `remove_teleportable_erc20` deregisters a contract.
			//
			// SECURITY INVARIANT: this pays out of `T::TeleportCheckingAccount` for any
			// admitted state (including `Deregistered`) without re-checking the origin —
			// `_context` is unreliable here (the executor may have cleared the origin
			// before `DepositAsset`). That is sound ONLY because an asset cannot enter the
			// XCM holding register unbacked: the executor gates every holding-fill
			// instruction upstream — `WithdrawAsset` performs a real EVM debit (and this
			// transactor refuses `Deregistered` outbound; the legacy adapter refuses every
			// whitelisted contract, see `legacy_transactor_rejects_teleportable`),
			// `ReserveAssetDeposited` is gated by `IsReserve`, and `ReceiveTeleportedAsset`
			// is gated by `IsTeleporter` + `can_check_in` (both bound to
			// `TeleportTrustedLocation`). If any of those upstream gates is ever loosened
			// (e.g. a new `Barrier`/`IsReserve` config), this unconditional payout would
			// become a checking-account drain — keep that coupling in mind.
			let (contract, amount) = Self::match_admitted(what)?;

			// Zero-amount fast path: trivially succeed without touching the EVM,
			// the counter, or the lifecycle. See the module-level docs for why we
			// short-circuit here. The contract was admitted by the matcher, so this
			// is only a no-op for whitelisted entries; non-whitelisted contracts
			// still fall through to the legacy adapter via `AssetNotFound`.
			if amount.is_zero() {
				return Ok(());
			}

			let beneficiary = T::AccountIdConverter::convert_location(who)
				.ok_or(MatchError::AccountIdConversionFailed)?;

			let gas_limit = Pallet::<T>::gas_limit_of_erc20_transfer(&what.id);
			let checking = T::TeleportCheckingAccount::get();

			frame_support::storage::with_storage_layer(|| -> Result<(), Erc20TransferError> {
				Pallet::<T>::erc20_transfer(contract, checking, beneficiary, amount, gas_limit)?;
				// Decrement the obligation counter. Saturating so that pre-seeded
				// supply on the trusted counterparty (which has no matching prior
				// outbound leg here) doesn't underflow; the deposit still proceeds.
				LockedSupply::<T>::mutate(&contract, |b| *b = b.saturating_sub(amount));
				// First leg in this lifecycle promotes `Registered → Active`. Done
				// inside the storage layer so a failed EVM transfer rolls back the
				// state transition along with the counter mutation.
				Pallet::<T>::promote_registered_to_active(&contract);
				Ok(())
			})
			.map_err(Into::into)
		}

		fn withdraw_asset(
			what: &Asset,
			who: &Location,
			_context: Option<&XcmContext>,
		) -> Result<AssetsInHolding, XcmError> {
			// Outbound side of the EVM transfer. Outbound-eligible only.
			// `Deregistered` contracts must NOT lock new supply here.
			let (contract, amount) = Self::match_outbound(what)?;

			// Zero-amount fast path. Skips the EVM call, the `LockedSupply` write,
			// and the `Registered → Active` promotion so spam-able 0-amount
			// teleports cannot eat through the lifecycle for free. Holding stays
			// consistent because we still report the asset as withdrawn (with
			// fungible amount 0).
			if amount.is_zero() {
				return Ok(what.clone().into());
			}

			let from = T::AccountIdConverter::convert_location(who)
				.ok_or(MatchError::AccountIdConversionFailed)?;

			let gas_limit = Pallet::<T>::gas_limit_of_erc20_transfer(&what.id);
			let checking = T::TeleportCheckingAccount::get();

			frame_support::storage::with_storage_layer(|| -> Result<(), Erc20TransferError> {
				Pallet::<T>::erc20_transfer(contract, from, checking, amount, gas_limit)?;
				// Increment the obligation counter to reflect the new locked supply.
				LockedSupply::<T>::mutate(&contract, |b| *b = b.saturating_add(amount));
				Pallet::<T>::promote_registered_to_active(&contract);
				Ok(())
			})?;

			Ok(what.clone().into())
		}

		fn internal_transfer_asset(
			asset: &Asset,
			from: &Location,
			to: &Location,
			_context: &XcmContext,
		) -> Result<AssetsInHolding, XcmError> {
			// Same-chain XCM hop. Treat like a fresh outbound for matching purposes
			// (outbound-eligible only); otherwise fall through to the legacy reserve
			// adapter via `AssetNotHandled`.
			//
			// The checking account is **not** involved, so `LockedSupply` is not
			// touched: a same-chain transfer changes neither the locked-supply
			// obligation nor the cross-chain accounting. The status promotion still
			// fires because this is a successful flow handled by this transactor.
			let (contract, amount) = Self::match_outbound(asset)?;

			// Zero-amount fast path: see `withdraw_asset` rationale.
			if amount.is_zero() {
				return Ok(asset.clone().into());
			}

			let from = T::AccountIdConverter::convert_location(from)
				.ok_or(MatchError::AccountIdConversionFailed)?;
			let to = T::AccountIdConverter::convert_location(to)
				.ok_or(MatchError::AccountIdConversionFailed)?;

			let gas_limit = Pallet::<T>::gas_limit_of_erc20_transfer(&asset.id);

			frame_support::storage::with_storage_layer(|| -> Result<(), Erc20TransferError> {
				Pallet::<T>::erc20_transfer(contract, from, to, amount, gas_limit)?;
				Pallet::<T>::promote_registered_to_active(&contract);
				Ok(())
			})?;

			Ok(asset.clone().into())
		}
	}

	/// Filter that admits teleports of whitelisted ERC-20s.
	///
	/// Implements both:
	/// - `ContainsPair<Asset, Location>` for use as `xcm_executor::Config::IsTeleporter`.
	///   This is the inbound + outbound counterparty trust gate. Returns `true` ONLY when
	///   the asset is a whitelisted ERC-20 in any state (`Registered`, `Active`, or
	///   `Deregistered`) AND the location equals `T::TeleportTrustedLocation::get()`.
	///   Without that location bind, any sibling chain able to deliver XCM to this
	///   runtime could present a whitelisted ERC-20 in `ReceiveTeleportedAsset`, pass
	///   `IsTeleporter`, and drain the EVM checking account via the subsequent
	///   `DepositAsset`. `Deregistered` contracts are still admitted here so the
	///   teleport-back path stays open; outbound teleports of `Deregistered` contracts
	///   are blocked instead by `Erc20TeleportTransactor::{can_check_out, withdraw_asset}`
	///   and the `XcmTeleportFilter` impl below — all of which require outbound
	///   eligibility (`Registered` or `Active`).
	/// - `Contains<(Location, Vec<Asset>)>` for use as
	///   `pallet_xcm::Config::XcmTeleportFilter`. The user-facing outbound gate. It
	///   admits ONLY outbound-eligible contracts (`Registered` or `Active`) so that
	///   `pallet_xcm::limited_teleport_assets` cannot start a fresh outbound teleport
	///   once the operator has called `remove_teleportable_erc20` (and the entry is
	///   either `Deregistered` or fully purged). The location argument here is the
	///   local caller's origin (set inside `pallet_xcm::limited_teleport_assets`
	///   before the executor runs), NOT the destination, so it is intentionally not
	///   bound. The destination is enforced separately by `IsTeleporter` when the
	///   executor builds the outbound message.
	pub struct IsTeleportableErc20<T>(PhantomData<T>);

	impl<T: Config> IsTeleportableErc20<T> {
		/// Whether the asset's ERC-20 contract has any whitelist entry. Used by
		/// `IsTeleporter` to keep the inbound unwind path open across all states.
		fn asset_is_admitted(asset: &Asset) -> bool {
			match Erc20Matcher::<T::Erc20MultilocationPrefix>::matches_fungibles(asset) {
				Ok((contract, _amount)) => TeleportableErc20s::<T>::contains_key(&contract),
				Err(_) => false,
			}
		}

		/// Whether the asset's ERC-20 contract is currently outbound-eligible
		/// (`Registered` or `Active`). Used by the user-facing outbound
		/// `XcmTeleportFilter` so `remove_teleportable_erc20` immediately closes the
		/// door to fresh outbound teleports.
		fn asset_is_outbound_eligible(asset: &Asset) -> bool {
			match Erc20Matcher::<T::Erc20MultilocationPrefix>::matches_fungibles(asset) {
				Ok((contract, _amount)) => matches!(
					TeleportableErc20s::<T>::get(&contract),
					Some(TeleportableErc20Status::Registered)
						| Some(TeleportableErc20Status::Active),
				),
				Err(_) => false,
			}
		}
	}

	impl<T: Config> ContainsPair<Asset, Location> for IsTeleportableErc20<T> {
		fn contains(asset: &Asset, location: &Location) -> bool {
			location == &T::TeleportTrustedLocation::get() && Self::asset_is_admitted(asset)
		}
	}

	impl<T: Config> Contains<(Location, Vec<Asset>)> for IsTeleportableErc20<T> {
		fn contains(value: &(Location, Vec<Asset>)) -> bool {
			let (_origin, assets) = value;
			!assets.is_empty() && assets.iter().all(Self::asset_is_outbound_eligible)
		}
	}
}
