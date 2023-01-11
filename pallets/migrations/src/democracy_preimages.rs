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

use {
	super::Error,
	frame_support::{
		pallet_prelude::*,
		traits::{Currency, PalletInfo, StorageInstance},
	},
	frame_system::{pallet_prelude::*, Config as SystemConfig},
	pallet_democracy::Config as DemocracyConfig,
	pallet_preimage::{Config as PreimageConfig, RequestStatus},
	parity_scale_codec::Input,
	sp_std::prelude::*,
};

/// Maximum size of preimage we can store is 4mb.
pub const MAX_SIZE: u32 = 4 * 1024 * 1024;

pub type BalanceOf<T> = <<T as pallet_democracy::Config>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::Balance;

#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum PreimageStatus<AccountId, Balance, BlockNumber> {
	/// The preimage is imminently needed at the argument.
	Missing(BlockNumber),
	/// The preimage is available.
	Available {
		data: Vec<u8>,
		provider: AccountId,
		deposit: Balance,
		since: BlockNumber,
		/// None if it's not imminent.
		expiry: Option<BlockNumber>,
	},
}

// ---- Old storage ----

pub struct DeprecatedDemocracyPreimagesPrefix<T>(PhantomData<T>);

impl<T: DemocracyConfig> StorageInstance for DeprecatedDemocracyPreimagesPrefix<T> {
	const STORAGE_PREFIX: &'static str = "Preimages";

	fn pallet_prefix() -> &'static str {
		T::PalletInfo::name::<pallet_democracy::Pallet<T>>()
			.expect("there is pallet democracy installed")
	}
}

/// Storage entry for preimages once stored in pallet_democracy.
pub(crate) type DeprecatedDemocracyPreimages<T> = StorageMap<
	DeprecatedDemocracyPreimagesPrefix<T>,
	Identity,
	<T as SystemConfig>::Hash,
	PreimageStatus<<T as SystemConfig>::AccountId, BalanceOf<T>, <T as SystemConfig>::BlockNumber>,
>;

// ---- New storage ----

pub struct StatusForPrefix<T>(PhantomData<T>);

impl<T: PreimageConfig> StorageInstance for StatusForPrefix<T> {
	const STORAGE_PREFIX: &'static str = "StatusFor";

	fn pallet_prefix() -> &'static str {
		T::PalletInfo::name::<pallet_preimage::Pallet<T>>()
			.expect("there is pallet preimage installed")
	}
}

pub(crate) type StatusFor<T> = StorageMap<
	StatusForPrefix<T>,
	Identity,
	<T as SystemConfig>::Hash,
	RequestStatus<<T as SystemConfig>::AccountId, BalanceOf<T>>,
>;

pub struct PreimageForPrefix<T>(PhantomData<T>);

impl<T: PreimageConfig> StorageInstance for PreimageForPrefix<T> {
	const STORAGE_PREFIX: &'static str = "PreimageFor";

	fn pallet_prefix() -> &'static str {
		T::PalletInfo::name::<pallet_preimage::Pallet<T>>()
			.expect("there is pallet preimage installed")
	}
}

pub(crate) type PreimageFor<T> = StorageMap<
	PreimageForPrefix<T>,
	Identity,
	(<T as SystemConfig>::Hash, u32),
	BoundedVec<u8, ConstU32<MAX_SIZE>>,
>;

// ---- Call ----

impl<T: super::Config> super::Pallet<T> {
	pub(crate) fn migrate_democracy_preimage_inner(
		origin: OriginFor<T>,
		proposal_hash: T::Hash,
		proposal_len_upper_bound: u32,
	) -> DispatchResultWithPostInfo {
		let _who = ensure_signed(origin);

		// Check that this hash doesn't already exist in the new storage.
		ensure!(
			!StatusFor::<T>::contains_key(proposal_hash),
			Error::<T>::PreimageAlreadyExists,
		);

		// Check bound is correct.
		ensure!(
			Self::pre_image_data_len(proposal_hash)? <= proposal_len_upper_bound,
			Error::<T>::WrongUpperBound,
		);

		// Get the old preimage data.
		let (data, provider, deposit) = <DeprecatedDemocracyPreimages<T>>::get(&proposal_hash)
			.and_then(|m| match m {
				PreimageStatus::Available {
					data,
					provider,
					deposit,
					..
				} => Some((data, provider, deposit)),
				_ => None,
			})
			.ok_or(Error::<T>::PreimageMissing)?;

		// Insert data in new storages.
		let data: BoundedVec<_, _> = data.try_into().map_err(|_| Error::<T>::PreimageIsTooBig)?;

		let request_status = RequestStatus::Unrequested {
			deposit: (provider, deposit),
			len: data.len() as u32,
		};

		StatusFor::<T>::insert(proposal_hash, request_status);
		PreimageFor::<T>::insert((proposal_hash, data.len() as u32), data);

		// Clean old storage.
		<DeprecatedDemocracyPreimages<T>>::remove(&proposal_hash);

		Ok(().into())
	}

	fn pre_image_data_len(proposal_hash: T::Hash) -> Result<u32, DispatchError> {
		// To decode the `data` field of Available variant we need:
		// * one byte for the variant
		// * at most 5 bytes to decode a `Compact<u32>`
		let mut buf = [0u8; 6];
		let key = <DeprecatedDemocracyPreimages<T>>::hashed_key_for(proposal_hash);
		let bytes = sp_io::storage::read(&key, &mut buf, 0).ok_or(Error::<T>::PreimageMissing)?;
		// The value may be smaller that 6 bytes.
		let mut input = &buf[0..buf.len().min(bytes as usize)];

		match input.read_byte() {
			Ok(1) => (), // Check that input exists and is second variant.
			Ok(0) => return Err(Error::<T>::PreimageMissing.into()),
			_ => {
				sp_runtime::print("Failed to decode `PreimageStatus` variant");
				return Err(Error::<T>::PreimageMissing.into());
			}
		}

		// Decode the length of the vector.
		let len = parity_scale_codec::Compact::<u32>::decode(&mut input)
			.map_err(|_| {
				sp_runtime::print("Failed to decode `PreimageStatus` variant");
				DispatchError::from(Error::<T>::PreimageMissing)
			})?
			.0;

		Ok(len)
	}
}
