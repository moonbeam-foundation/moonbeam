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

///! Instant Randomness
///! exposes the most recent values for all the pallet storage values
use crate::{Config, Error, Pallet};
use frame_support::StorageValue;
use pallet_vrf::GetMaybeRandomness;
use sp_runtime::DispatchError;

/// Returns most recent value for the local randomness (per block VRF)
pub fn instant_local_randomness<T: Config>(salt: T::Hash) -> Result<[u8; 32], DispatchError> {
	let randomness =
		T::LocalRandomness::get_current_randomness().ok_or(Error::<T>::RandomnessNotAvailable)?;
	Ok(Pallet::<T>::concat_and_hash(randomness, salt))
}

/// Returns most recent value for the specified storage item `V`
pub fn instant_relay_randomness<T, V>(salt: T::Hash) -> Result<[u8; 32], DispatchError>
where
	T: Config,
	V: StorageValue<Option<T::Hash>, Query = Option<T::Hash>>,
{
	let randomness = V::get().ok_or(Error::<T>::RandomnessNotAvailable)?;
	Ok(Pallet::<T>::concat_and_hash(randomness, salt))
}
