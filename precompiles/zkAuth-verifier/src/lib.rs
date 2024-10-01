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
use fp_evm::PrecompileHandle;
use frame_support::{ensure, traits::Time};
use moonbeam_runtime_common::weights::pallet_precompile_benchmarks::WeightInfo;
use pallet_precompile_benchmarks::WeightInfo as TWeightInfo;
use precompile_utils::prelude::*;
use sp_core::ConstU32;
use sp_runtime::traits::UniqueSaturatedInto;
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
	715585636, 3586935525, 3274293606, 2872050810, 564159597, 2621011314, 3667725176, 1510137221,
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
	) -> EvmResult<UnboundedBytes> {
		// Charge weight for zkAuth receipt verification
		let weight = WeightInfo::<Runtime>::zk_auth_verify();
		handle.record_external_cost(Some(weight.ref_time()), Some(0), Some(0))?;

		let encoded_receipt: Vec<u8> = receipt.into();

		// Verify the risc0 zk-proof receipt
		let receipt: risc0_zkvm::Receipt = postcard::from_bytes(&encoded_receipt)
			.map_err(|_| RevertReason::Custom("Receipt decoding failed".into()))?;

		let image_id = storage::ImageId::get().ok_or(RevertReason::custom("no ImageId stored"))?;
		receipt
			.verify(image_id)
			.map_err(|_| RevertReason::Custom("Error verifying receipt".into()))?;

		let (jwt_exp, user_op_hash) = receipt
			.journal
			.decode::<(u128, Vec<u8>)>()
			.map_err(|_| RevertReason::Custom("Error decoding journal".into()))?;

		// Check JWT expiration time against relay timestamp.
		// Relay timestamp is in milliseconds and the JWT exp time is in seconds,
		// so we do de conversion.
		let timestamp: u128 =
			<Runtime as pallet_evm::Config>::Timestamp::now().unique_saturated_into();
		let timestamp: u128 = timestamp / 1000;

		ensure!(jwt_exp >= timestamp, revert("JWT expired"));

		Ok(user_op_hash.into())
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
