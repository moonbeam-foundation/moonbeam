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
use fp_evm::{PrecompileFailure, PrecompileHandle};
use precompile_utils::prelude::*;
use sp_std::vec::Vec;

pub mod encoded_receipt;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub const JWT_VALIDATOR_ID: [u32; 8] = [
	1923256869, 654795233, 2887859926, 1709721587, 1196091263, 3916749566, 1248329059, 610202488,
];

pub struct ZkAuthVerifierPrecompile<Runtime>(PhantomData<Runtime>);

#[precompile_utils::precompile]
impl<Runtime> ZkAuthVerifierPrecompile<Runtime>
where
	Runtime: frame_system::Config,
{
	#[precompile::public("verify(uint8[])")]
	fn verify_proof(handle: &mut impl PrecompileHandle, receipt: Vec<u8>) -> EvmResult {
		//TODO: record cost
		handle.record_cost(1000)?;

		let receipt: risc0_zkvm::Receipt = postcard::from_bytes(&receipt)
			.map_err(|_| RevertReason::Custom("Receipt decoding failed".into()))?;

		let image_id = storage::ImageId::get().ok_or(RevertReason::custom("no ImageId stored"))?;

		receipt
			.verify(image_id)
			.map_err(|_| RevertReason::Custom("Error verifying receipt".into()))?;
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
