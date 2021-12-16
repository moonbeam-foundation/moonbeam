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
//! This pallet maps NimbusId => AccountId which is most useful when using propositional style
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

pub mod migrations;

#[pallet]
pub mod pallet {
	use crate::WeightInfo;
	use frame_support::pallet_prelude::*;
	use frame_support::traits::{Currency, ReservableCurrency};
	use frame_system::pallet_prelude::*;
	use nimbus_primitives::{AccountLookup, NimbusId};

	pub type BalanceOf<T> = <<T as Config>::DepositCurrency as Currency<
		<T as frame_system::Config>::AccountId,
	>>::Balance;

	#[derive(Encode, Decode, PartialEq, Eq, Debug, scale_info::TypeInfo)]
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
		/// Currency in which the security deposit will be taken.
		type DepositCurrency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
		/// The amount that should be taken as a security deposit when registering a NimbusId.
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
		/// The NimbusId in question is already associated and cannot be overwritten
		AlreadyAssociated,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A NimbusId has been registered and mapped to an AccountId.
		AuthorRegistered(NimbusId, T::AccountId),
		/// An NimbusId has been de-registered, and its AccountId mapping removed.
		AuthorDeRegistered(NimbusId),
		/// An NimbusId has been registered, replacing a previous registration and its mapping.
		AuthorRotated(NimbusId, T::AccountId),
		/// An NimbusId has been forcibly deregistered after not being rotated or cleaned up.
		/// The reporteing account has been rewarded accordingly.
		DefunctAuthorBusted(NimbusId, T::AccountId),
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Register your NimbusId onchain so blocks you author are associated with your account.
		///
		/// Users who have been (or will soon be) elected active collators in staking,
		/// should submit this extrinsic to have their blocks accepted and earn rewards.
		#[pallet::weight(<T as Config>::WeightInfo::add_association())]
		pub fn add_association(origin: OriginFor<T>, author_id: NimbusId) -> DispatchResult {
			let account_id = ensure_signed(origin)?;

			ensure!(
				MappingWithDeposit::<T>::get(&author_id).is_none(),
				Error::<T>::AlreadyAssociated
			);

			Self::enact_registration(&author_id, &account_id)?;

			<Pallet<T>>::deposit_event(Event::AuthorRegistered(author_id, account_id));

			Ok(())
		}

		/// Change your Mapping.
		///
		/// This is useful for normal key rotation or for when switching from one physical collator
		/// machine to another. No new security deposit is required.
		#[pallet::weight(<T as Config>::WeightInfo::update_association())]
		pub fn update_association(
			origin: OriginFor<T>,
			old_author_id: NimbusId,
			new_author_id: NimbusId,
		) -> DispatchResult {
			let account_id = ensure_signed(origin)?;

			let stored_info = MappingWithDeposit::<T>::try_get(&old_author_id)
				.map_err(|_| Error::<T>::AssociationNotFound)?;

			ensure!(
				account_id == stored_info.account,
				Error::<T>::NotYourAssociation
			);

			MappingWithDeposit::<T>::remove(&old_author_id);
			MappingWithDeposit::<T>::insert(&new_author_id, &stored_info);

			<Pallet<T>>::deposit_event(Event::AuthorRotated(new_author_id, stored_info.account));

			Ok(())
		}

		/// Clear your Mapping.
		///
		/// This is useful when you are no longer an author and would like to re-claim your security
		/// deposit.
		#[pallet::weight(<T as Config>::WeightInfo::clear_association())]
		pub fn clear_association(
			origin: OriginFor<T>,
			author_id: NimbusId,
		) -> DispatchResultWithPostInfo {
			let account_id = ensure_signed(origin)?;

			let stored_info = MappingWithDeposit::<T>::try_get(&author_id)
				.map_err(|_| Error::<T>::AssociationNotFound)?;

			ensure!(
				account_id == stored_info.account,
				Error::<T>::NotYourAssociation
			);

			MappingWithDeposit::<T>::remove(&author_id);

			T::DepositCurrency::unreserve(&account_id, stored_info.deposit);

			<Pallet<T>>::deposit_event(Event::AuthorDeRegistered(author_id));

			Ok(().into())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn enact_registration(
			author_id: &NimbusId,
			account_id: &T::AccountId,
		) -> DispatchResult {
			let deposit = T::DepositAmount::get();

			T::DepositCurrency::reserve(&account_id, deposit)
				.map_err(|_| Error::<T>::CannotAffordSecurityDeposit)?;

			let info = RegistrationInfo {
				account: account_id.clone(),
				deposit,
			};

			MappingWithDeposit::<T>::insert(&author_id, &info);

			Ok(())
		}
	}

	#[pallet::storage]
	#[pallet::getter(fn account_and_deposit_of)]
	/// We maintain a mapping from the NimbusIds used in the consensus layer
	/// to the AccountIds runtime (including this staking pallet).
	pub type MappingWithDeposit<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		NimbusId,
		RegistrationInfo<T::AccountId, BalanceOf<T>>,
		OptionQuery,
	>;

	#[pallet::genesis_config]
	/// Genesis config for author mapping pallet
	pub struct GenesisConfig<T: Config> {
		/// The associations that should exist at chain genesis
		pub mappings: Vec<(NimbusId, T::AccountId)>,
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

	impl<T: Config> AccountLookup<T::AccountId> for Pallet<T> {
		fn lookup_account(author: &NimbusId) -> Option<T::AccountId> {
			Self::account_id_of(author)
		}
	}

	impl<T: Config> Pallet<T> {
		/// A helper function to lookup the account id associated with the given author id. This is
		/// the primary lookup that this pallet is responsible for.
		pub fn account_id_of(author_id: &NimbusId) -> Option<T::AccountId> {
			Self::account_and_deposit_of(author_id).map(|info| info.account)
		}
	}
}
