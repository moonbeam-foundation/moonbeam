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

pub mod types;
pub mod weights;

#[cfg(any(test, feature = "runtime-benchmarks"))]
mod benchmarks;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub use pallet::*;
pub use types::*;
pub use weights::WeightInfo;

use frame_support::pallet;
use nimbus_primitives::{AccountLookup, NimbusId};

#[pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_support::traits::{Currency, NamedReservableCurrency};
	use frame_system::pallet_prelude::*;
	use sp_runtime::traits::{CheckedSub, One, Saturating, StaticLookup, Zero};

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
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// A type to convert between AuthorId and AccountId. This pallet wrap the lookup to allow
		/// orbiters authoring.
		type AccountLookup: AccountLookup<Self::AccountId>;

		/// Origin that is allowed to add a collator in orbiters program.
		type AddCollatorOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// The currency type.
		type Currency: NamedReservableCurrency<Self::AccountId>;

		/// Origin that is allowed to remove a collator from orbiters program.
		type DelCollatorOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		#[pallet::constant]
		/// Maximum number of orbiters per collator.
		type MaxPoolSize: Get<u32>;

		#[pallet::constant]
		/// Maximum number of round to keep on storage.
		type MaxRoundArchive: Get<Self::RoundIndex>;

		/// Reserve identifier for this pallet instance.
		type OrbiterReserveIdentifier: Get<ReserveIdentifierOf<Self>>;

		#[pallet::constant]
		/// Number of rounds before changing the selected orbiter.
		/// WARNING: when changing `RotatePeriod`, you need a migration code that sets
		/// `ForceRotation` to true to avoid holes in `OrbiterPerRound`.
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

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;
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
	/// If true, it forces the rotation at the next round.
	/// A use case: when changing RotatePeriod, you need a migration code that sets this value to
	/// true to avoid holes in OrbiterPerRound.
	pub(crate) type ForceRotation<T: Config> = StorageValue<_, bool, ValueQuery>;

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

	#[pallet::storage]
	#[pallet::getter(fn orbiter)]
	/// Check if account is an orbiter
	pub type RegisteredOrbiter<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, bool>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub min_orbiter_deposit: BalanceOf<T>,
	}

	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self {
				min_orbiter_deposit: One::one(),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			assert!(
				self.min_orbiter_deposit > Zero::zero(),
				"Minimal orbiter deposit should be greater than zero"
			);
			MinOrbiterDeposit::<T>::put(self.min_orbiter_deposit)
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(_: BlockNumberFor<T>) -> Weight {
			// Prune old OrbiterPerRound entries
			if let Some(round_to_prune) =
				CurrentRound::<T>::get().checked_sub(&T::MaxRoundArchive::get())
			{
				// TODO: Find better limit.
				// Is it sure to be cleared in a single block? In which case we can probably have
				// a lower limit.
				// Otherwise, we should still have a lower limit, and implement a multi-block clear
				// by using the return value of clear_prefix for subsequent blocks.
				let result = OrbiterPerRound::<T>::clear_prefix(round_to_prune, u32::MAX, None);
				T::WeightInfo::on_initialize(result.unique)
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
		/// This orbiter has not made a deposit
		OrbiterDepositNotFound,
		/// This orbiter is not found
		OrbiterNotFound,
		/// The orbiter is still at least in one pool
		OrbiterStillInAPool,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// An orbiter join a collator pool
		OrbiterJoinCollatorPool {
			collator: T::AccountId,
			orbiter: T::AccountId,
		},
		/// An orbiter leave a collator pool
		OrbiterLeaveCollatorPool {
			collator: T::AccountId,
			orbiter: T::AccountId,
		},
		/// Paid the orbiter account the balance as liquid rewards.
		OrbiterRewarded {
			account: T::AccountId,
			rewards: BalanceOf<T>,
		},
		OrbiterRotation {
			collator: T::AccountId,
			old_orbiter: Option<T::AccountId>,
			new_orbiter: Option<T::AccountId>,
		},
		/// An orbiter has registered
		OrbiterRegistered {
			account: T::AccountId,
			deposit: BalanceOf<T>,
		},
		/// An orbiter has unregistered
		OrbiterUnregistered { account: T::AccountId },
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Add an orbiter in a collator pool
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::collator_add_orbiter())]
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

			collator_pool.add_orbiter(orbiter.clone());
			CollatorsPool::<T>::insert(&collator, collator_pool);

			Self::deposit_event(Event::OrbiterJoinCollatorPool { collator, orbiter });

			Ok(())
		}

		/// Remove an orbiter from the caller collator pool
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::collator_remove_orbiter())]
		pub fn collator_remove_orbiter(
			origin: OriginFor<T>,
			orbiter: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResult {
			let collator = ensure_signed(origin)?;
			let orbiter = T::Lookup::lookup(orbiter)?;

			Self::do_remove_orbiter_from_pool(collator, orbiter)
		}

		/// Remove the caller from the specified collator pool
		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::orbiter_leave_collator_pool())]
		pub fn orbiter_leave_collator_pool(
			origin: OriginFor<T>,
			collator: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResult {
			let orbiter = ensure_signed(origin)?;
			let collator = T::Lookup::lookup(collator)?;

			Self::do_remove_orbiter_from_pool(collator, orbiter)
		}

		/// Registering as an orbiter
		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::orbiter_register())]
		pub fn orbiter_register(origin: OriginFor<T>) -> DispatchResult {
			let orbiter = ensure_signed(origin)?;

			if let Some(min_orbiter_deposit) = MinOrbiterDeposit::<T>::get() {
				// The use of `ensure_reserved_named` allows to update the deposit amount in case a
				// deposit has already been made.
				T::Currency::ensure_reserved_named(
					&T::OrbiterReserveIdentifier::get(),
					&orbiter,
					min_orbiter_deposit,
				)?;
				RegisteredOrbiter::<T>::insert(&orbiter, true);
				Self::deposit_event(Event::OrbiterRegistered {
					account: orbiter,
					deposit: min_orbiter_deposit,
				});
				Ok(())
			} else {
				Err(Error::<T>::MinOrbiterDepositNotSet.into())
			}
		}

		/// Deregistering from orbiters
		#[pallet::call_index(4)]
		#[pallet::weight(T::WeightInfo::orbiter_unregister(*collators_pool_count))]
		pub fn orbiter_unregister(
			origin: OriginFor<T>,
			collators_pool_count: u32,
		) -> DispatchResult {
			let orbiter = ensure_signed(origin)?;

			// We have to make sure that the `collators_pool_count` parameter is large enough,
			// because its value is used to calculate the weight of this extrinsic
			ensure!(
				collators_pool_count >= CollatorsPool::<T>::count(),
				Error::<T>::CollatorsPoolCountTooLow
			);

			// Ensure that the orbiter is not in any pool
			ensure!(
				!CollatorsPool::<T>::iter_values()
					.any(|collator_pool| collator_pool.contains_orbiter(&orbiter)),
				Error::<T>::OrbiterStillInAPool,
			);

			T::Currency::unreserve_all_named(&T::OrbiterReserveIdentifier::get(), &orbiter);
			RegisteredOrbiter::<T>::remove(&orbiter);
			Self::deposit_event(Event::OrbiterUnregistered { account: orbiter });

			Ok(())
		}

		/// Add a collator to orbiters program.
		#[pallet::call_index(5)]
		#[pallet::weight(T::WeightInfo::add_collator())]
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
		#[pallet::call_index(6)]
		#[pallet::weight(T::WeightInfo::remove_collator())]
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
	}

	impl<T: Config> Pallet<T> {
		fn do_remove_orbiter_from_pool(
			collator: T::AccountId,
			orbiter: T::AccountId,
		) -> DispatchResult {
			let mut collator_pool =
				CollatorsPool::<T>::get(&collator).ok_or(Error::<T>::CollatorNotFound)?;

			match collator_pool.remove_orbiter(&orbiter) {
				RemoveOrbiterResult::OrbiterNotFound => {
					return Err(Error::<T>::OrbiterNotFound.into())
				}
				RemoveOrbiterResult::OrbiterRemoved => {
					Self::deposit_event(Event::OrbiterLeaveCollatorPool {
						collator: collator.clone(),
						orbiter,
					});
				}
				RemoveOrbiterResult::OrbiterRemoveScheduled => (),
			}

			CollatorsPool::<T>::insert(collator, collator_pool);
			Ok(())
		}
		fn on_rotate(round_index: T::RoundIndex) -> Weight {
			let mut writes = 1;
			// Update current orbiter for each pool and edit AccountLookupOverride accordingly.
			CollatorsPool::<T>::translate::<CollatorPoolInfo<T::AccountId>, _>(
				|collator, mut pool| {
					let RotateOrbiterResult {
						maybe_old_orbiter,
						maybe_next_orbiter,
					} = pool.rotate_orbiter();

					// remove old orbiter, if any.
					if let Some(CurrentOrbiter {
						account_id: ref current_orbiter,
						removed,
					}) = maybe_old_orbiter
					{
						if removed {
							Self::deposit_event(Event::OrbiterLeaveCollatorPool {
								collator: collator.clone(),
								orbiter: current_orbiter.clone(),
							});
						}
						AccountLookupOverride::<T>::remove(current_orbiter.clone());
						writes += 1;
					}
					if let Some(next_orbiter) = maybe_next_orbiter {
						// Forbidding the collator to write blocks, it is now up to its orbiters to do it.
						AccountLookupOverride::<T>::insert(
							collator.clone(),
							Option::<T::AccountId>::None,
						);
						writes += 1;

						// Insert new current orbiter
						AccountLookupOverride::<T>::insert(
							next_orbiter.clone(),
							Some(collator.clone()),
						);
						writes += 1;

						let mut i = Zero::zero();
						while i < T::RotatePeriod::get() {
							OrbiterPerRound::<T>::insert(
								round_index.saturating_add(i),
								collator.clone(),
								next_orbiter.clone(),
							);
							i += One::one();
							writes += 1;
						}

						Self::deposit_event(Event::OrbiterRotation {
							collator,
							old_orbiter: maybe_old_orbiter.map(|orbiter| orbiter.account_id),
							new_orbiter: Some(next_orbiter),
						});
					} else {
						// If there is no more active orbiter, you have to remove the collator override.
						AccountLookupOverride::<T>::remove(collator.clone());
						writes += 1;
						Self::deposit_event(Event::OrbiterRotation {
							collator,
							old_orbiter: maybe_old_orbiter.map(|orbiter| orbiter.account_id),
							new_orbiter: None,
						});
					}
					writes += 1;
					Some(pool)
				},
			);
			T::DbWeight::get().reads_writes(1, writes)
		}
		/// Notify this pallet that a new round begin
		pub fn on_new_round(round_index: T::RoundIndex) -> Weight {
			CurrentRound::<T>::put(round_index);

			if ForceRotation::<T>::get() {
				ForceRotation::<T>::put(false);
				let _ = Self::on_rotate(round_index);
				T::WeightInfo::on_new_round()
			} else if round_index % T::RotatePeriod::get() == Zero::zero() {
				let _ = Self::on_rotate(round_index);
				T::WeightInfo::on_new_round()
			} else {
				T::DbWeight::get().writes(1)
			}
		}
		/// Notify this pallet that a collator received rewards
		pub fn distribute_rewards(
			pay_for_round: T::RoundIndex,
			collator: T::AccountId,
			amount: BalanceOf<T>,
		) -> Weight {
			if let Some(orbiter) = OrbiterPerRound::<T>::take(pay_for_round, &collator) {
				if T::Currency::deposit_into_existing(&orbiter, amount).is_ok() {
					Self::deposit_event(Event::OrbiterRewarded {
						account: orbiter,
						rewards: amount,
					});
				}
				T::WeightInfo::distribute_rewards()
			} else {
				// writes: take
				T::DbWeight::get().writes(1)
			}
		}

		/// Check if an account is a collator pool account with an
		/// orbiter assigned for a given round
		pub fn is_collator_pool_with_active_orbiter(
			for_round: T::RoundIndex,
			collator: T::AccountId,
		) -> bool {
			OrbiterPerRound::<T>::contains_key(for_round, &collator)
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
