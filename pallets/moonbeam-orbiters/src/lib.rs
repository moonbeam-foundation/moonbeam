// Copyright 2019-2022 PureStake Inc.
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

//! # Pallet moonbeam-orbiters
//!
//! This pallet allows authorized collators to share their block creation rights and rewards with
//! multiple entities named "orbiters".
//! Each authorized collator will define a group of orbiters, and each orbiter will replace the
//! collator in turn with the other orbiters (rotation every `RotatePeriod` rounds).
//!
//! This pallet is designed to work with the nimbus consensus.
//! In order not to impact the other pallets (notably nimbus and parachain-staking) this pallet
//! simply redefines the lookup NimbusId-> AccountId, in order to replace the collator by its
//! currently selected orbiter.

#![cfg_attr(not(feature = "std"), no_std)]

pub mod migrations;
pub mod types;

#[cfg(any(test, feature = "runtime-benchmarks"))]
mod benchmarks;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub use pallet::*;
pub use types::*;

use frame_support::pallet;
use nimbus_primitives::{AccountLookup, NimbusId};

#[pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_support::traits::{Currency, Imbalance, NamedReservableCurrency};
	use frame_system::pallet_prelude::*;
	use sp_runtime::traits::{One, StaticLookup, Zero};

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(PhantomData<T>);

	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	pub type ReserveIdentifierOf<T> = <<T as Config>::Currency as NamedReservableCurrency<
		<T as frame_system::Config>::AccountId,
	>>::ReserveIdentifier;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Overarching event type.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// A type to convert between AuthorId and AccountId. This pallet wrap the lookup to allow
		/// orbiters authoring.
		type AccountLookup: AccountLookup<Self::AccountId>;

		/// Origin that is allowed to add a collator in orbiters program.
		type AddCollatorOrigin: EnsureOrigin<Self::Origin>;

		/// The currency type.
		type Currency: NamedReservableCurrency<Self::AccountId>;

		/// Origin that is allowed to remove a collator from orbiters program.
		type DelCollatorOrigin: EnsureOrigin<Self::Origin>;

		/// Maximum number of orbiters per collator.
		type MaxPoolSize: Get<u32>;

		/// Maximum number of round to keep on storage.
		type MaxRoundArchive: Get<Self::RoundIndex>;

		/// Reserve identifier for this pallet instance.
		type OrbiterReserveIdentifier: Get<ReserveIdentifierOf<Self>>;

		/// Number of rounds before changing the selected orbiter.
		type RotatePeriod: Get<Self::RoundIndex>;

		/// Round index type.
		type RoundIndex: Parameter
			+ Member
			+ MaybeSerializeDeserialize
			+ sp_std::fmt::Debug
			+ Default
			+ sp_runtime::traits::MaybeDisplay
			+ sp_runtime::traits::AtLeast32Bit
			+ Copy;

		/// Origin that is allowed to update the minimal orbiter deposit amount.
		type UpdateMinOrbiterDepositOrigin: EnsureOrigin<Self::Origin>;
	}

	#[pallet::storage]
	#[pallet::getter(fn account_lookup_override)]
	/// Account lookup override
	pub type AccountLookupOverride<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, Option<T::AccountId>>;

	#[pallet::storage]
	#[pallet::getter(fn collators_pool)]
	/// Current orbiters, with their "parent" collator
	pub type CollatorsPool<T: Config> =
		CountedStorageMap<_, Blake2_128Concat, T::AccountId, CollatorPoolInfo<T::AccountId>>;

	#[pallet::storage]
	/// Current round index
	pub(crate) type CurrentRound<T: Config> = StorageValue<_, T::RoundIndex, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn min_orbiter_deposit)]
	/// Minimum deposit required to be registered as an orbiter
	pub type MinOrbiterDeposit<T: Config> = StorageValue<_, BalanceOf<T>, OptionQuery>;

	#[pallet::storage]
	/// Store active orbiter per round and per parent collator
	pub(crate) type OrbiterPerRound<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		T::RoundIndex,
		Blake2_128Concat,
		T::AccountId,
		T::AccountId,
		OptionQuery,
	>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub min_orbiter_deposit: BalanceOf<T>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self {
				min_orbiter_deposit: One::one(),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			assert!(
				self.min_orbiter_deposit > Zero::zero(),
				"Minumal orbiter deposit should be greater tham zero"
			);
			MinOrbiterDeposit::<T>::put(self.min_orbiter_deposit)
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(_: T::BlockNumber) -> Weight {
			// Prune old OrbiterPerRound entries
			let current_round = CurrentRound::<T>::get();
			let max_round_archive = T::MaxRoundArchive::get();
			if current_round > max_round_archive {
				let round_to_prune = current_round - max_round_archive;
				OrbiterPerRound::<T>::remove_prefix(round_to_prune, None);
				T::DbWeight::get().reads_writes(1, 1)
			} else {
				T::DbWeight::get().reads(1)
			}
		}
	}

	/// An error that can occur while executing this pallet's extrinsics.
	#[pallet::error]
	pub enum Error<T> {
		/// The collator is already added in orbiters program.
		CollatorAlreadyAdded,
		/// This collator is not in orbiters program.
		CollatorNotFound,
		/// There are already too many orbiters associated with this collator.
		CollatorPoolTooLarge,
		/// There are more collator pools than the number specified in the parameter.
		CollatorsPoolCountTooLow,
		/// The minimum deposit required to register as an orbiter has not yet been included in the
		/// onchain storage
		MinOrbiterDepositNotSet,
		/// This orbiter is already associated with this collator.
		OrbiterAlreadyInPool,
		/// Orbiter cant leave this round
		OrbiterCantLeaveThisRound,
		/// This orbiter has not made a deposit
		OrbiterDepositNotFound,
		/// This orbiter is not found
		OrbiterNotFound,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Paid the orbiter account the balance as liquid rewards.
		OrbiterRewarded {
			account: T::AccountId,
			rewards: BalanceOf<T>,
		},
		/*SelectedOrbiters {
			orbiters:
		}*/
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Add an orbiter in a collator pool
		#[pallet::weight(500_000_000)]
		pub fn collator_add_orbiter(
			origin: OriginFor<T>,
			orbiter: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResult {
			let collator = ensure_signed(origin)?;
			let orbiter = T::Lookup::lookup(orbiter)?;

			let mut collator_pool =
				CollatorsPool::<T>::get(&collator).ok_or(Error::<T>::CollatorNotFound)?;
			let orbiters = collator_pool.get_orbiters();
			ensure!(
				(orbiters.len() as u32) < T::MaxPoolSize::get(),
				Error::<T>::CollatorPoolTooLarge
			);
			if orbiters.iter().any(|orbiter_| orbiter_ == &orbiter) {
				return Err(Error::<T>::OrbiterAlreadyInPool.into());
			}

			// Make sure the orbiter has made a deposit. It can be an old orbiter whose deposit
			// is lower than the current minimum (if the minimum was lower in the past), so we just
			// have to check that a deposit exists (which means checking that the deposit amount
			// is not zero).
			let orbiter_deposit =
				T::Currency::reserved_balance_named(&T::OrbiterReserveIdentifier::get(), &orbiter);
			ensure!(
				orbiter_deposit > BalanceOf::<T>::zero(),
				Error::<T>::OrbiterDepositNotFound
			);

			collator_pool.add_orbiter(orbiter);
			CollatorsPool::<T>::insert(collator, collator_pool);

			Ok(())
		}

		/// Remove an orbiter from the caller collator pool
		#[pallet::weight(500_000_000)]
		pub fn collator_remove_orbiter(
			origin: OriginFor<T>,
			orbiter: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResult {
			let collator = ensure_signed(origin)?;
			let orbiter = T::Lookup::lookup(orbiter)?;

			let mut collator_pool =
				CollatorsPool::<T>::get(&collator).ok_or(Error::<T>::CollatorNotFound)?;

			ensure!(
				collator_pool.remove_orbiter(&orbiter),
				Error::<T>::OrbiterNotFound
			);

			CollatorsPool::<T>::insert(collator, collator_pool);
			Ok(())
		}

		/// Remove the caller from the specified collator pool
		#[pallet::weight(500_000_000)]
		pub fn orbiter_leave_collator_pool(
			origin: OriginFor<T>,
			collator: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResult {
			let orbiter = ensure_signed(origin)?;
			let collator = T::Lookup::lookup(collator)?;

			let mut collator_pool =
				CollatorsPool::<T>::get(&collator).ok_or(Error::<T>::CollatorNotFound)?;

			ensure!(
				collator_pool.remove_orbiter(&orbiter),
				Error::<T>::OrbiterNotFound
			);

			CollatorsPool::<T>::insert(collator, collator_pool);
			Ok(())
		}

		/// Registering as an orbiter
		#[pallet::weight(500_000_000)]
		pub fn orbiter_register(origin: OriginFor<T>) -> DispatchResult {
			let orbiter = ensure_signed(origin)?;

			if let Some(min_orbiter_deposit) = MinOrbiterDeposit::<T>::get() {
				// The use of `ensure_reserved_named` allows to update the deposit amount in case a
				// deposit has already been made.
				T::Currency::ensure_reserved_named(
					&T::OrbiterReserveIdentifier::get(),
					&orbiter,
					min_orbiter_deposit,
				)
			} else {
				Err(Error::<T>::MinOrbiterDepositNotSet.into())
			}
		}

		/// Deregistering from orbiters
		#[pallet::weight(500_000_000)]
		pub fn orbiter_unregister(
			origin: OriginFor<T>,
			collators_pool_count: u32,
		) -> DispatchResult {
			let orbiter = ensure_signed(origin)?;

			// If the orbiter is currently active in this round, it cannot unregister, this would
			// create annoying side effects. So we force the orbiter to redo is call later (or to
			// schedule his call).
			ensure!(
				AccountLookupOverride::<T>::get(&orbiter).is_none(),
				Error::<T>::OrbiterCantLeaveThisRound
			);

			// We have to make sure that the `collators_pool_count` parameter is large enough,
			// because its value is used to calculate the weight of this extrinsic
			ensure!(
				collators_pool_count >= CollatorsPool::<T>::count(),
				Error::<T>::CollatorsPoolCountTooLow
			);

			// We remove the orbiter from all the collator pools in which it is located.
			CollatorsPool::<T>::translate_values(
				|mut collator_pool: CollatorPoolInfo<T::AccountId>| {
					if collator_pool.remove_orbiter(&orbiter) {
						Some(collator_pool)
					} else {
						// Optimization: if the orbiter is not in this pool, we return None to
						// avoid adding an unnecessary write
						None
					}
				},
			);

			T::Currency::unreserve_all_named(&T::OrbiterReserveIdentifier::get(), &orbiter);

			Ok(())
		}

		/// Add a collator to orbiters program.
		///
		#[pallet::weight(500_000_000)]
		pub fn add_collator(
			origin: OriginFor<T>,
			collator: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResult {
			T::AddCollatorOrigin::ensure_origin(origin)?;
			let collator = T::Lookup::lookup(collator)?;

			ensure!(
				CollatorsPool::<T>::get(&collator).is_none(),
				Error::<T>::CollatorAlreadyAdded
			);

			CollatorsPool::<T>::insert(collator, CollatorPoolInfo::default());

			Ok(())
		}

		/// Remove a collator from orbiters program.
		#[pallet::weight(500_000_000)]
		pub fn remove_collator(
			origin: OriginFor<T>,
			collator: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResult {
			T::DelCollatorOrigin::ensure_origin(origin)?;
			let collator = T::Lookup::lookup(collator)?;

			// Remove the pool associated to this collator
			let collator_pool =
				CollatorsPool::<T>::take(&collator).ok_or(Error::<T>::CollatorNotFound)?;

			// Remove all AccountLookupOverride entries related to this collator
			for orbiter in collator_pool.get_orbiters() {
				AccountLookupOverride::<T>::remove(&orbiter);
			}
			AccountLookupOverride::<T>::remove(&collator);

			Ok(())
		}

		/// Update minimum orbiter deposit
		#[pallet::weight(500_000_000)]
		pub fn update_min_orbiter_deposit(
			origin: OriginFor<T>,
			new_min_orbiter_deposit: BalanceOf<T>,
		) -> DispatchResult {
			T::UpdateMinOrbiterDepositOrigin::ensure_origin(origin)?;

			MinOrbiterDeposit::<T>::put(new_min_orbiter_deposit);
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Notify this pallet that a new round begin
		pub fn on_new_round(round_index: T::RoundIndex) -> Weight {
			let mut writes = 1;
			CurrentRound::<T>::put(round_index);

			if round_index % T::RotatePeriod::get() == Zero::zero() {
				// Update current orbiter for each pool and edit AccountLookupOverride accordingly.
				CollatorsPool::<T>::translate::<CollatorPoolInfo<T::AccountId>, _>(
					|collator, mut pool| {
						// remove current orbiter, if any.
						if let Some(current_orbiter) = pool.get_current_orbiter() {
							AccountLookupOverride::<T>::remove(current_orbiter);
							writes += 1;
						}
						if let Some(next_orbiter) = pool.next_orbiter() {
							// Forbidding the collator to write blocks, it is now up to its orbiters to do it.
							AccountLookupOverride::<T>::insert(
								collator.clone(),
								Option::<T::AccountId>::None,
							);
							// Insert new current orbiter
							AccountLookupOverride::<T>::insert(
								next_orbiter.clone(),
								Some(collator.clone()),
							);
							OrbiterPerRound::<T>::insert(round_index, collator, next_orbiter);
							writes += 3;
						} else {
							// If there is no more active orbiter, you have to remove the collator override.
							AccountLookupOverride::<T>::remove(collator);
							writes += 1;
						}
						writes += 1;
						Some(pool)
					},
				);
			}
			T::DbWeight::get().reads_writes(1, writes)
		}
		/// Notify this pallet that a collator received rewards
		pub fn distribute_rewards(
			pay_for_round: T::RoundIndex,
			collator: T::AccountId,
			amount: BalanceOf<T>,
		) -> Weight {
			if let Some(orbiter) = OrbiterPerRound::<T>::take(pay_for_round, &collator) {
				if let Ok(amount_to_transfer) = T::Currency::withdraw(
					&collator,
					amount,
					frame_support::traits::WithdrawReasons::TRANSFER,
					frame_support::traits::ExistenceRequirement::KeepAlive,
				) {
					let real_reward = amount_to_transfer.peek();
					if T::Currency::resolve_into_existing(&orbiter, amount_to_transfer).is_ok() {
						Self::deposit_event(Event::OrbiterRewarded {
							account: orbiter,
							rewards: real_reward,
						});
						// reads: withdraw + resolve_into_existing
						// writes: take + withdraw + resolve_into_existing
						T::DbWeight::get().reads_writes(2, 3)
					} else {
						// reads: withdraw + resolve_into_existing
						// writes: take + withdraw
						T::DbWeight::get().reads_writes(2, 2)
					}
				} else {
					// reads: withdraw
					// writes: take
					T::DbWeight::get().reads_writes(1, 1)
				}
			} else {
				// writes: take
				T::DbWeight::get().writes(1)
			}
		}
	}
}

impl<T: Config> AccountLookup<T::AccountId> for Pallet<T> {
	fn lookup_account(nimbus_id: &NimbusId) -> Option<T::AccountId> {
		let account_id = T::AccountLookup::lookup_account(nimbus_id)?;
		match AccountLookupOverride::<T>::get(&account_id) {
			Some(override_) => override_,
			None => Some(account_id),
		}
	}
}
