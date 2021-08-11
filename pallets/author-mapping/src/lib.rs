// Copyright 2019-2021 PureStake Inc.
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

//! Maps Author Ids as used in nimbus consensus layer to account ids as used i nthe runtime.
//! This should likely be moved to nimbus eventually.
//!
//! This pallet maps AuthorId => AccountId which is most useful when using propositional style
//! queries. This mapping will likely need to go the other way if using exhaustive authority sets.
//! That could either be a seperate pallet, or this pallet could implement a two-way mapping. But
//! for now it it one-way

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet;

pub use pallet::*;

pub mod weights;
use weights::WeightInfo;
#[cfg(any(test, feature = "runtime-benchmarks"))]
mod benchmarks;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[pallet]
pub mod pallet {
	use crate::WeightInfo;
	use frame_support::pallet_prelude::*;
	use frame_support::traits::{Currency, ReservableCurrency};
	use frame_system::pallet_prelude::*;
	use nimbus_primitives::AccountLookup;

	pub type BalanceOf<T> = <<T as Config>::DepositCurrency as Currency<
		<T as frame_system::Config>::AccountId,
	>>::Balance;

	#[derive(Encode, Decode, PartialEq, Eq)]
	pub struct RegistrationInfo<AccountId, Balance> {
		account: AccountId,
		deposit: Balance,
	}

	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	/// Configuration trait of this pallet. We tightly couple to Parachain Staking in order to
	/// ensure that only staked accounts can create registrations in the first place. This could be
	/// generalized.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Overarching event type
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// The type of authority id that will be used at the consensus layer.
		type AuthorId: Member + Parameter + MaybeSerializeDeserialize + Default;
		/// Currency in which the security deposit will be taken.
		type DepositCurrency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
		/// The amount that should be taken as a security deposit when registering an AuthorId.
		type DepositAmount: Get<<Self::DepositCurrency as Currency<Self::AccountId>>::Balance>;
		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;
	}

	/// An error that can occur while executing the mapping pallet's logic.
	#[pallet::error]
	pub enum Error<T> {
		/// The association can't be cleared because it is not found.
		AssociationNotFound,
		/// The association can't be cleared because it belongs to another account.
		NotYourAssociation,
		/// This account cannot set an author because it cannon afford the security deposit
		CannotAffordSecurityDeposit,
		/// The AuthorId in question is already associated and cannot be overwritten
		AlreadyAssociated,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// An AuthorId has been registered and mapped to an AccountId.
		AuthorRegistered(T::AuthorId, T::AccountId),
		/// An AuthorId has been de-registered, and its AccountId mapping removed.
		AuthorDeRegistered(T::AuthorId),
		/// An AuthorId has been registered, replacing a previous registration and its mapping.
		AuthorRotated(T::AuthorId, T::AccountId),
		/// An AuthorId has been forcibly deregistered after not being rotated or cleaned up.
		/// The reporteing account has been rewarded accordingly.
		DefunctAuthorBusted(T::AuthorId, T::AccountId),
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Register your AuthorId onchain so blocks you author are associated with your account.
		///
		/// Users who have been (or will soon be) elected active collators in staking,
		/// should submit this extrinsic to have their blocks accepted and earn rewards.
		#[pallet::weight(<T as Config>::WeightInfo::add_association())]
		pub fn add_association(origin: OriginFor<T>, author_id: T::AuthorId) -> DispatchResult {
			let account_id = ensure_signed(origin)?;

			ensure!(
				Mapping::<T>::get(&author_id).is_none(),
				Error::<T>::AlreadyAssociated
			);

			Self::enact_registration(&author_id, &account_id)?;

			<Pallet<T>>::deposit_event(Event::AuthorRegistered(author_id, account_id));

			Ok(())
		}

		/// Change your AuthorId.
		///
		/// This is useful for normal key rotation or for when switching from one physical collator
		/// machine to another. No new security deposit is required.
		#[pallet::weight(<T as Config>::WeightInfo::update_association())]
		pub fn update_association(
			origin: OriginFor<T>,
			old_author_id: T::AuthorId,
			new_author_id: T::AuthorId,
		) -> DispatchResult {
			let account_id = ensure_signed(origin)?;

			let stored_info = Mapping::<T>::try_get(&old_author_id)
				.map_err(|_| Error::<T>::AssociationNotFound)?;

			ensure!(
				account_id == stored_info.account,
				Error::<T>::NotYourAssociation
			);

			Mapping::<T>::insert(&new_author_id, &stored_info);
			Mapping::<T>::remove(&old_author_id);

			<Pallet<T>>::deposit_event(Event::AuthorRotated(new_author_id, stored_info.account));

			Ok(())
		}

		/// Clear your AuthorId.
		///
		/// This is useful when you are no longer an author and would like to re-claim your security
		/// deposit.
		#[pallet::weight(<T as Config>::WeightInfo::clear_association())]
		pub fn clear_association(
			origin: OriginFor<T>,
			author_id: T::AuthorId,
		) -> DispatchResultWithPostInfo {
			let account_id = ensure_signed(origin)?;

			let stored_info =
				Mapping::<T>::try_get(&author_id).map_err(|_| Error::<T>::AssociationNotFound)?;

			ensure!(
				account_id == stored_info.account,
				Error::<T>::NotYourAssociation
			);

			Mapping::<T>::remove(&author_id);

			T::DepositCurrency::unreserve(&account_id, stored_info.deposit);

			<Pallet<T>>::deposit_event(Event::AuthorDeRegistered(author_id));

			Ok(().into())
		}

		//TODO maybe in the future we will add some more incentivization for key cleanup and also
		// proper key rotation
		// /// The portion of the security deposit that goes to the the account who reports it
		// /// occupying space after it should have been cleaned or rotated.
		// pub const NARC_REWARD: Percent = Percent::from_percent(5);

		// /// The period of time after which an AuthorId can be reported as defunct.
		// /// This value should be roughly the recommended key rotation period.
		// pub const NARC_GRACE_PERIOD: u32 = 2_000;
		//
		// /// Narc on another account for having a useless association and collect a bounty.
		// ///
		// /// This incentivizes good citizenship in the form of cleaning up others' defunct
		// /// associations. When you clean up another account's association, you will receive X
		// /// percent of their security deposit.
		// ///
		// /// No association can be cleaned up within the initial grace period which allows collators
		// /// some time to get their associations onchain before they become active, and to clean up
		// /// after they are no longer active.
		// ///
		// /// This also _forces_ collators to rotate their keys regularly because failing to will
		// /// make their mappings ripe for narcing. If an active collator gets its association reaped
		// /// they will lose out on their block rewards (and in the future potentially be slashed).
		// #[pallet::weight(0)]
		// pub fn narc_defunct_association(
		// 	origin: OriginFor<T>,
		// 	author_id: T::AuthorId,
		// ) -> DispatchResult {
		// 	todo!()
		// }
	}

	impl<T: Config> Pallet<T> {
		pub fn enact_registration(
			author_id: &T::AuthorId,
			account_id: &T::AccountId,
		) -> DispatchResult {
			let deposit = T::DepositAmount::get();

			T::DepositCurrency::reserve(&account_id, deposit)
				.map_err(|_| Error::<T>::CannotAffordSecurityDeposit)?;

			let info = RegistrationInfo {
				account: account_id.clone(),
				deposit,
			};

			Mapping::<T>::insert(&author_id, &info);

			Ok(())
		}
	}

	#[pallet::storage]
	#[pallet::getter(fn account_and_deposit_of)]
	/// We maintain a mapping from the AuthorIds used in the consensus layer
	/// to the AccountIds runtime (including the parachain staking pallet).
	type Mapping<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AuthorId,
		RegistrationInfo<T::AccountId, BalanceOf<T>>,
		OptionQuery,
	>;

	#[pallet::genesis_config]
	/// Genesis config for author mapping pallet
	pub struct GenesisConfig<T: Config> {
		/// The associations that should exist at chain genesis
		pub mappings: Vec<(T::AuthorId, T::AccountId)>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self { mappings: vec![] }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			for (author_id, account_id) in &self.mappings {
				if let Err(e) = Pallet::<T>::enact_registration(&author_id, &account_id) {
					log::warn!("Error with genesis author mapping registration: {:?}", e);
				}
			}
		}
	}

	impl<T: Config> AccountLookup<T::AuthorId, T::AccountId> for Pallet<T> {
		fn lookup_account(author: &T::AuthorId) -> Option<T::AccountId> {
			Self::account_id_of(author)
		}
	}

	impl<T: Config> Pallet<T> {
		/// A helper function to lookup the account id associated with the given author id. This is
		/// the primary lookup that this pallet is responsible for.
		pub fn account_id_of(author_id: &T::AuthorId) -> Option<T::AccountId> {
			Self::account_and_deposit_of(author_id).map(|info| info.account)
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_runtime_upgrade() -> Weight {
			let pallet_prefix: &[u8] = b"AuthorMapping";
			let old_storage_item_prefix: &[u8] = b"MappingWithDeposit";

			let mut migrated_entries: Weight = 0;

			// Move the mappings to the new storage location
			// https://crates.parity.io/frame_support/storage/migration/fn.storage_key_iter.html
			for (author_id, account_id) in frame_support::storage::migration::storage_key_iter::<
				T::AuthorId,
				RegistrationInfo<T::AccountId, BalanceOf<T>>,
				Twox64Concat,
			>(pallet_prefix, old_storage_item_prefix)
			{
				Mapping::<T>::insert(author_id, account_id);
				migrated_entries += 1;
			}

			// Now remove the old storage
			// https://crates.parity.io/frame_support/storage/migration/fn.remove_storage_prefix.html
			frame_support::storage::migration::remove_storage_prefix(
				pallet_prefix,
				old_storage_item_prefix,
				&[],
			);

			// Return the weight used.
			let db_weights = T::DbWeight::get();
			migrated_entries
				.saturating_add(2 * db_weights.write)
				.saturating_add(db_weights.read)
		}
	}
}
