// Copyright 2025 Moonbeam foundation
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

//! # Moonbeam bridge common primitives

#![cfg_attr(not(feature = "std"), no_std)]

use bp_runtime::{EncodedOrDecodedCall, StorageMapKeyProvider};
use frame_support::Blake2_128Concat;
use sp_core::storage::StorageKey;
use sp_runtime::generic;
use sp_std::vec::Vec;

pub use moonbeam_core_primitives::{AccountId, Balance, BlockNumber, Hash, Header, Signature};

/// Unchecked Extrinsic type.
pub type UncheckedExtrinsic<Call, SignedExt> =
	generic::UncheckedExtrinsic<AccountId, EncodedOrDecodedCall<Call>, Signature, SignedExt>;

/// Provides a storage key for account data.
///
/// We need to use this approach when we don't have access to the runtime.
/// The equivalent command to invoke in case full `Runtime` is known is this:
/// `let key = frame_system::Account::<Runtime>::storage_map_final_key(&account_id);`
pub struct AccountInfoStorageMapKeyProvider;

impl StorageMapKeyProvider for AccountInfoStorageMapKeyProvider {
	const MAP_NAME: &'static str = "Account";
	type Hasher = Blake2_128Concat;
	type Key = AccountId;
	// This should actually be `AccountInfo`, but we don't use this property in order to decode the
	// data. So we use `Vec<u8>` as if we would work with encoded data.
	type Value = Vec<u8>;
}

impl AccountInfoStorageMapKeyProvider {
	/// Name of the system pallet.
	const PALLET_NAME: &'static str = "System";

	/// Return storage key for given account data.
	pub fn final_key(id: &AccountId) -> StorageKey {
		<Self as StorageMapKeyProvider>::final_key(Self::PALLET_NAME, id)
	}
}
