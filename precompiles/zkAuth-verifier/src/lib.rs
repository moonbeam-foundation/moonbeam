// Copyright 2024 Moonbeam Foundation.
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

#![cfg_attr(not(feature = "std"), no_std)]

use core::marker::PhantomData;
use fp_evm::{Context, ExitReason, PrecompileFailure, PrecompileHandle, Transfer};
use precompile_utils::{evm::costs::call_cost, prelude::*};
use sp_core::{ConstU32, U256};
use sp_std::vec::Vec;

pub mod encoded_receipt;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub const CALL_DATA_LIMIT: u32 = 2u32.pow(16);
pub const ARRAY_LIMIT: u32 = 1601;
pub type GetCallDataLimit = ConstU32<CALL_DATA_LIMIT>;
pub type GetArrayLimit = ConstU32<ARRAY_LIMIT>;

pub const JWT_VALIDATOR_ID: [u32; 8] = [
	1923256869, 654795233, 2887859926, 1709721587, 1196091263, 3916749566, 1248329059, 610202488,
];

pub struct ZkAuthVerifierPrecompile<Runtime>(PhantomData<Runtime>);

#[precompile_utils::precompile]
impl<Runtime> ZkAuthVerifierPrecompile<Runtime>
where
	Runtime: frame_system::Config + pallet_evm::Config,
{
	#[precompile::public("verifyProof(bytes)")]
	fn verify_proof(
		handle: &mut impl PrecompileHandle,
		receipt: BoundedBytes<GetArrayLimit>,
	) -> EvmResult {
		//TODO: record proper cost
		handle.record_cost(1000)?;
		let receipt: Vec<u8> = receipt.into();

		// Verify the risc0 zk-proof receipt
		let receipt: risc0_zkvm::Receipt = postcard::from_bytes(&receipt)
			.map_err(|_| RevertReason::Custom("Receipt decoding failed".into()))?;

		let image_id = storage::ImageId::get().ok_or(RevertReason::custom("no ImageId stored"))?;
		receipt
			.verify(image_id)
			.map_err(|_| RevertReason::Custom("Error verifying receipt".into()))?;

		// TODO: return journal fields
		Ok(())
	}
}

/// TODO: check if we really need this to be dynamic.
///
/// We need to store the ImageId so that we can dynamically change the guest program
/// to verify.
///
/// We implement a StorageInstance for it.
///
/// Calculated with -> sp_io::hashing::twox_128(b"zkAuth");
///
/// twox_128("zkAuth") => 0x74d1fb05c68193c306242692e7d1ac45
/// twox_128("ImageId") => 0x6312aac1f9ae01d96b2d8690d6a04689
mod storage {
	use frame_support::{
		storage::types::{OptionQuery, StorageValue},
		traits::StorageInstance,
	};

	pub struct ImageIdStorageInstance;
	impl StorageInstance for ImageIdStorageInstance {
		const STORAGE_PREFIX: &'static str = "ImageId";
		fn pallet_prefix() -> &'static str {
			"zkAuth"
		}
	}

	// TODO: is it better to store a BoundedVec<u32, ConstU32<8>>?
	pub type ImageId = StorageValue<ImageIdStorageInstance, [u32; 8], OptionQuery>;
}
