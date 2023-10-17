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

//! # Hotfix Pallet
//! A pallet that allows for the execution of hotfixes.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::traits::ConstU32;
use pallet_evm::AddressMapping;
use sp_core::H160;

pub const ARRAY_LIMIT: u32 = 1000;
type GetArrayLimit = ConstU32<ARRAY_LIMIT>;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_evm::Config {
		#[pallet::constant]
		type EntryClearLimit: Get<u32>;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::error]
	pub enum Error<T> {
		/// The contract is not suicided
		ContractNotSuicided,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		#[pallet::call_index(0)]
		pub fn clear_suicided_storage(
			origin: OriginFor<T>,
			addresses: BoundedVec<H160, GetArrayLimit>,
		) -> DispatchResultWithPostInfo {
			let _ = ensure_signed(origin)?;
			let limit = T::EntryClearLimit::get();
			let mut deleted = 0;

			for address in &addresses {
				// Ensure that the contract is suicided by checking that it has no code and at least
				// one storage entry.
				ensure!(
					!pallet_evm::AccountCodes::<T>::contains_key(&address)
						&& pallet_evm::AccountStorages::<T>::iter_key_prefix(&address)
							.next()
							.is_some(),
					Error::<T>::ContractNotSuicided
				);

				let mut iter = pallet_evm::AccountStorages::<T>::iter_key_prefix(address).drain();
				while let Some(key) = iter.next() {
					pallet_evm::AccountStorages::<T>::remove(address, key);
					deleted += 1;
					if deleted >= limit {
						if iter.next().is_none() {
							Self::clear_suicided_contract(&address);
						}
						break;
					}
				}
				Self::clear_suicided_contract(address);
			}

			Ok(().into())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn clear_suicided_contract(address: &H160) {
			// Decrement the sufficients of the account
			let account_id = T::AddressMapping::into_account_id(*address);
			let _ = frame_system::Pallet::<T>::dec_sufficients(&account_id);
		}
	}
}
