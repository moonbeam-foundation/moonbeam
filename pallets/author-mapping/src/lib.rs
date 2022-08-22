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

//! Maps Author Ids as used in nimbus consensus layer to account ids as used in the runtime.
//! This should likely be moved to nimbus eventually.
//!
//! This pallet maps NimbusId => AccountId which is most useful when using propositional style
//! queries. This mapping will likely need to go the other way if using exhaustive authority sets.
//! That could either be a separate pallet, or this pallet could implement a two-way mapping. But
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
	use session_keys_primitives::KeysLookup;
	use sp_std::{mem::size_of, vec::Vec};

	pub type BalanceOf<T> = <<T as Config>::DepositCurrency as Currency<
		<T as frame_system::Config>::AccountId,
	>>::Balance;

	#[derive(Clone, Encode, Decode, PartialEq, Eq, Debug, scale_info::TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct RegistrationInfo<T: Config> {
		pub(crate) account: T::AccountId,
		pub(crate) deposit: BalanceOf<T>,
		pub(crate) keys: T::Keys,
	}

	/// Wrapper to form the input to `set_keys` from NimbusId + keys
	pub fn keys_wrapper<T: Config>(nimbus_id: NimbusId, keys: T::Keys) -> Vec<u8> {
		let mut r = nimbus_id.encode();
		r.extend(&keys.encode());
		r
	}

	/// Size of NimbusId + T::Keys to check size of `set_keys` input before trying to decode
	pub fn keys_size<T: Config>() -> usize {
		size_of::<NimbusId>() as usize + size_of::<T::Keys>() as usize
	}

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(PhantomData<T>);

	/// Configuration trait of this pallet
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Overarching event type
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// Currency in which the security deposit will be taken.
		type DepositCurrency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
		/// The amount that should be taken as a security deposit when registering a NimbusId.
		type DepositAmount: Get<<Self::DepositCurrency as Currency<Self::AccountId>>::Balance>;
		/// Additional keys
		/// Convertible From<NimbusId> to get default keys for each mapping (for the migration)
		type Keys: Parameter + Member + MaybeSerializeDeserialize + From<NimbusId>;
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
		/// No existing NimbusId can be found for the account
		OldAuthorIdNotFound,
		/// Keys have wrong size
		WrongKeySize,
		/// Failed to decode NimbusId for `set_keys`
		DecodeNimbusFailed,
		/// Failed to decode T::Keys for `set_keys`
		DecodeKeysFailed,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A NimbusId has been registered and mapped to an AccountId.
		KeysRegistered {
			nimbus_id: NimbusId,
			account_id: T::AccountId,
			keys: T::Keys,
		},
		/// An NimbusId has been de-registered, and its AccountId mapping removed.
		KeysRemoved {
			nimbus_id: NimbusId,
			account_id: T::AccountId,
			keys: T::Keys,
		},
		/// An NimbusId has been registered, replacing a previous registration and its mapping.
		KeysRotated {
			new_nimbus_id: NimbusId,
			account_id: T::AccountId,
			new_keys: T::Keys,
		},
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Register your NimbusId onchain so blocks you author are associated with your account.
		///
		/// Users who have been (or will soon be) elected active collators in staking,
		/// should submit this extrinsic to have their blocks accepted and earn rewards.
		#[pallet::weight(<T as Config>::WeightInfo::add_association())]
		pub fn add_association(origin: OriginFor<T>, nimbus_id: NimbusId) -> DispatchResult {
			let account_id = ensure_signed(origin)?;

			Self::register_keys(nimbus_id.clone(), account_id, nimbus_id.into())
		}

		/// Change your Mapping.
		///
		/// This is useful for normal key rotation or for when switching from one physical collator
		/// machine to another. No new security deposit is required.
		/// This sets keys to new_nimbus_id.into() by default.
		#[pallet::weight(<T as Config>::WeightInfo::update_association())]
		pub fn update_association(
			origin: OriginFor<T>,
			old_nimbus_id: NimbusId,
			new_nimbus_id: NimbusId,
		) -> DispatchResult {
			let account_id = ensure_signed(origin)?;

			Self::rotate_keys(
				old_nimbus_id,
				new_nimbus_id.clone(),
				account_id,
				new_nimbus_id.into(),
			)
		}

		/// Clear your Mapping.
		///
		/// This is useful when you are no longer an author and would like to re-claim your security
		/// deposit.
		#[pallet::weight(<T as Config>::WeightInfo::clear_association())]
		pub fn clear_association(origin: OriginFor<T>, nimbus_id: NimbusId) -> DispatchResult {
			let account_id = ensure_signed(origin)?;

			Self::rm_keys(nimbus_id, account_id)
		}

		/// Remove your Mapping.
		///
		/// This is useful when you are no longer an author and would like to re-claim your security
		/// deposit.
		#[pallet::weight(<T as Config>::WeightInfo::remove_keys())]
		pub fn remove_keys(origin: OriginFor<T>) -> DispatchResult {
			let account_id = ensure_signed(origin)?;
			let nimbus_id =
				Self::nimbus_id_of(&account_id).ok_or(Error::<T>::OldAuthorIdNotFound)?;

			Self::rm_keys(nimbus_id, account_id)
		}

		/// Set association and session keys at once.
		///
		/// This is useful for key rotation to update Nimbus and VRF keys in one call.
		/// No new security deposit is required. Will replace `update_association` which is kept
		/// now for backwards compatibility reasons.
		#[pallet::weight(<T as Config>::WeightInfo::set_keys())]
		pub fn set_keys(origin: OriginFor<T>, keys: Vec<u8>) -> DispatchResult {
			let account_id = ensure_signed(origin)?;
			ensure!(keys.len() == keys_size::<T>(), Error::<T>::WrongKeySize);
			let encoded = &mut keys.as_slice();
			let new_nimbus_id =
				NimbusId::decode(encoded).map_err(|_| Error::<T>::DecodeNimbusFailed)?;
			let keys = T::Keys::decode(encoded).map_err(|_| Error::<T>::DecodeKeysFailed)?;

			if let Some(old_nimbus_id) = Self::nimbus_id_of(&account_id) {
				Self::rotate_keys(old_nimbus_id, new_nimbus_id, account_id, keys)
			} else {
				Self::register_keys(new_nimbus_id, account_id, keys)
			}
		}
	}

	impl<T: Config> Pallet<T> {
		fn rm_keys(nimbus_id: NimbusId, account_id: T::AccountId) -> DispatchResult {
			let stored_info = MappingWithDeposit::<T>::try_get(&nimbus_id)
				.map_err(|_| Error::<T>::AssociationNotFound)?;

			ensure!(
				account_id == stored_info.account,
				Error::<T>::NotYourAssociation
			);

			MappingWithDeposit::<T>::remove(&nimbus_id);
			NimbusLookup::<T>::remove(&account_id);

			T::DepositCurrency::unreserve(&account_id, stored_info.deposit);

			<Pallet<T>>::deposit_event(Event::KeysRemoved {
				nimbus_id,
				account_id,
				keys: stored_info.keys,
			});
			Ok(())
		}
		fn register_keys(
			nimbus_id: NimbusId,
			account_id: T::AccountId,
			keys: T::Keys,
		) -> DispatchResult {
			ensure!(
				MappingWithDeposit::<T>::get(&nimbus_id).is_none(),
				Error::<T>::AlreadyAssociated
			);
			Self::enact_registration(&nimbus_id, &account_id, keys.clone())?;

			<Pallet<T>>::deposit_event(Event::KeysRegistered {
				nimbus_id,
				account_id,
				keys,
			});
			Ok(())
		}
		fn rotate_keys(
			old_nimbus_id: NimbusId,
			new_nimbus_id: NimbusId,
			account_id: T::AccountId,
			new_keys: T::Keys,
		) -> DispatchResult {
			let stored_info = MappingWithDeposit::<T>::try_get(&old_nimbus_id)
				.map_err(|_| Error::<T>::AssociationNotFound)?;

			ensure!(
				account_id == stored_info.account,
				Error::<T>::NotYourAssociation
			);
			if old_nimbus_id != new_nimbus_id {
				// cannot overwrite a NimbusId if it is not yours
				ensure!(
					MappingWithDeposit::<T>::get(&new_nimbus_id).is_none(),
					Error::<T>::AlreadyAssociated
				);
			}

			MappingWithDeposit::<T>::remove(&old_nimbus_id);
			let new_stored_info = RegistrationInfo {
				keys: new_keys.clone(),
				..stored_info
			};
			MappingWithDeposit::<T>::insert(&new_nimbus_id, &new_stored_info);
			NimbusLookup::<T>::insert(&account_id, &new_nimbus_id);

			<Pallet<T>>::deposit_event(Event::KeysRotated {
				new_nimbus_id,
				account_id,
				new_keys,
			});
			Ok(())
		}
		pub fn enact_registration(
			nimbus_id: &NimbusId,
			account_id: &T::AccountId,
			keys: T::Keys,
		) -> DispatchResult {
			let deposit = T::DepositAmount::get();

			T::DepositCurrency::reserve(account_id, deposit)
				.map_err(|_| Error::<T>::CannotAffordSecurityDeposit)?;

			let info = RegistrationInfo {
				account: account_id.clone(),
				deposit,
				keys,
			};

			MappingWithDeposit::<T>::insert(nimbus_id, info);
			NimbusLookup::<T>::insert(account_id, nimbus_id);

			Ok(())
		}
	}

	#[pallet::storage]
	#[pallet::getter(fn account_and_deposit_of)]
	/// We maintain a mapping from the NimbusIds used in the consensus layer
	/// to the AccountIds runtime.
	pub type MappingWithDeposit<T: Config> =
		StorageMap<_, Blake2_128Concat, NimbusId, RegistrationInfo<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn nimbus_lookup)]
	/// We maintain a reverse mapping from AccountIds to NimbusIDS
	pub type NimbusLookup<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, NimbusId, OptionQuery>;

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
			for (nimbus_id, account_id) in &self.mappings {
				if let Err(e) = Pallet::<T>::enact_registration(
					&nimbus_id,
					&account_id,
					nimbus_id.clone().into(),
				) {
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

	impl<T: Config> KeysLookup<NimbusId, T::Keys> for Pallet<T> {
		#[cfg(feature = "runtime-benchmarks")]
		type Account = T::AccountId;
		fn lookup_keys(author: &NimbusId) -> Option<T::Keys> {
			Self::keys_of(author)
		}
		/// Sets keys WITHOUT reserving deposit, for benchmarking purposes only
		#[cfg(feature = "runtime-benchmarks")]
		fn set_keys(id: NimbusId, account: T::AccountId, keys: T::Keys) {
			use sp_runtime::traits::Zero;
			MappingWithDeposit::<T>::insert(
				id,
				RegistrationInfo {
					account,
					deposit: Zero::zero(),
					keys,
				},
			);
		}
	}

	impl<T: Config> Pallet<T> {
		/// A helper function to lookup the account id associated with the given author id. This is
		/// the primary lookup that this pallet is responsible for.
		pub fn account_id_of(nimbus_id: &NimbusId) -> Option<T::AccountId> {
			Self::account_and_deposit_of(nimbus_id).map(|info| info.account)
		}
		/// A helper function to lookup the keys associated with the given author id.
		pub fn keys_of(nimbus_id: &NimbusId) -> Option<T::Keys> {
			Self::account_and_deposit_of(nimbus_id).map(|info| info.keys)
		}
		/// A helper function to lookup NimbusId associated with a given AccountId
		pub fn nimbus_id_of(account_id: &T::AccountId) -> Option<NimbusId> {
			NimbusLookup::<T>::get(account_id)
		}
	}
}
