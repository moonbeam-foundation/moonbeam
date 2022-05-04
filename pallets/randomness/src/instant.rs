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
use crate::{traits::SendRandomness, Config, Error, Pallet};
use frame_support::StorageValue;
use sp_runtime::DispatchResult;

// TODO: precompile methods to get this
pub fn instant_randomness<T, V>(contract_address: T::AccountId, salt: T::Hash) -> DispatchResult
where
	T: Config,
	V: StorageValue<Option<T::Hash>, Query = Option<T::Hash>>,
{
	let raw_randomness = V::get().ok_or(Error::<T>::RequestedRandomnessNotCorrectlyUpdated)?;
	let randomness = Pallet::<T>::concat_and_hash(raw_randomness, salt);
	T::RandomnessSender::send_randomness(contract_address, randomness);
	Ok(())
}
