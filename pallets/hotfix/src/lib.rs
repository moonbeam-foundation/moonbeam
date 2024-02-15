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

pub mod weights;

#[cfg(any(test, feature = "runtime-benchmarks"))]
mod benchmarks;
#[cfg(all(feature = "std", test))]
mod mock;
#[cfg(all(feature = "std", test))]
mod tests;

use frame_support::traits::ConstU32;
pub use pallet::*;
use sp_core::H160;
pub use weights::WeightInfo;

pub const ARRAY_LIMIT: u32 = 1000;
pub type GetArrayLimit = ConstU32<ARRAY_LIMIT>;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_evm::Config {
		type WeightInfo: WeightInfo;
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
		#[pallet::call_index(0)]
		#[pallet::weight({
			let addresses_len = addresses.len() as u32;
			<T as crate::Config>::WeightInfo::clear_suicided_storage(addresses_len, *limit)
		})]
		pub fn clear_suicided_storage(
			origin: OriginFor<T>,
			addresses: BoundedVec<H160, GetArrayLimit>,
			limit: u32,
		) -> DispatchResultWithPostInfo {
			let _ = ensure_signed(origin)?;
			let mut limit = limit as usize;

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

				let deleted = pallet_evm::AccountStorages::<T>::drain_prefix(*address)
					.take(limit)
					.count();

				limit = limit.saturating_sub(deleted);
				if limit == 0 {
					return Ok(().into());
				}
			}
			Ok(().into())
		}
	}
}
