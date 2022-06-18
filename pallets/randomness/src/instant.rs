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
use crate::{Config, Error, GetBabeData, LocalVrfOutput, Pallet};
use sp_core::H256;
use sp_runtime::DispatchError;

/// Returns BABE one epoch ago randomness
pub fn instant_one_epoch_ago_randomness<T: Config>(salt: H256) -> Result<[u8; 32], DispatchError> {
	let randomness = T::BabeDataGetter::get_one_epoch_ago_randomness()
		.ok_or(Error::<T>::RandomnessNotAvailable)?;
	Ok(Pallet::<T>::concat_and_hash(randomness, salt))
}

/// Returns BABE two epochs ago randomness
pub fn instant_two_epochs_ago_randomness<T: Config>(salt: H256) -> Result<[u8; 32], DispatchError> {
	let randomness = T::BabeDataGetter::get_two_epochs_ago_randomness()
		.ok_or(Error::<T>::RandomnessNotAvailable)?;
	Ok(Pallet::<T>::concat_and_hash(randomness, salt))
}

/// Returns BABE current block randomness
pub fn instant_current_block_randomness<T: Config>(salt: H256) -> Result<[u8; 32], DispatchError> {
	let randomness = T::BabeDataGetter::get_current_block_randomness()
		.ok_or(Error::<T>::RandomnessNotAvailable)?;
	Ok(Pallet::<T>::concat_and_hash(randomness, salt))
}

/// Returns most recent value for the local randomness
pub fn instant_local_randomness<T: Config>(salt: H256) -> Result<[u8; 32], DispatchError> {
	let randomness = <LocalVrfOutput<T>>::get().ok_or(Error::<T>::RandomnessNotAvailable)?;
	Ok(Pallet::<T>::concat_and_hash(randomness, salt))
}
