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
use sp_runtime::DispatchError;

/// Returns most recent value for the specified storage item `V`
/// generic over storage map to be generic over type of randomness
pub fn instant_randomness<T, V>(salt: T::Hash) -> Result<[u8; 32], DispatchError>
where
	T: Config,
	V: StorageValue<Option<T::Hash>, Query = Option<T::Hash>>,
{
	let randomness = V::get().ok_or(Error::<T>::RequestedRandomnessNotCorrectlyUpdated)?;
	Ok(Pallet::<T>::concat_and_hash(randomness, salt))
}
