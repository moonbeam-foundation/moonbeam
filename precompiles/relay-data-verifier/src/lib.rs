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

//! Precompile for verifying relay data against a relay block number.

#![cfg_attr(not(feature = "std"), no_std)]

use core::marker::PhantomData;
use cumulus_primitives_core::relay_chain::BlockNumber as RelayBlockNumber;
use fp_evm::PrecompileHandle;
use frame_support::traits::ConstU32;
use parity_scale_codec::Decode;
use precompile_utils::prelude::*;
use sp_core::{Get, H256};
use sp_std::vec::Vec;

mod proof;
use proof::*;

pub const CALL_DATA_LIMIT: u32 = 2u32.pow(16);
pub const ARRAY_LIMIT: u32 = 512;
pub const KEY_LENGTH_LIMIT: u32 = 256;

type GetCallDataLimit = ConstU32<CALL_DATA_LIMIT>;
type GetKeyLengthLimit = ConstU32<KEY_LENGTH_LIMIT>;
type GetArrayLimit = ConstU32<ARRAY_LIMIT>;

type RawStorageProof = BoundedBytes<GetCallDataLimit>;
type RawKey = BoundedBytes<GetKeyLengthLimit>;

/// Relay Data Verifier precompile.
pub struct RelayDataVerifierPrecompile<Runtime>(PhantomData<Runtime>);

#[precompile_utils::precompile]
impl<Runtime> RelayDataVerifierPrecompile<Runtime>
where
	Runtime: pallet_relay_storage_roots::Config + pallet_evm::Config,
{
	/// Verify the storage entry using the provided relay block number and proof. Return the value
	/// of the storage entry if the proof is valid and the entry exists.
	#[precompile::public("verifyEntry(uint32,bytes,bytes)")]
	#[precompile::public("verify_entry(uint32,bytes,bytes)")]
	fn verify_entry(
		handle: &mut impl PrecompileHandle,
		relay_block_number: RelayBlockNumber,
		proof: RawStorageProof,
		key: RawKey,
	) -> EvmResult<UnboundedBytes> {
		let storage_root = Self::get_storage_root(handle, relay_block_number)?;

		// Decode the proof of type `ReadProof` (The proof is expected to be
		// a SCALE encoded `ReadProof` that is returned by the `state_getProof` RPC call).
		let proof = ReadProof::decode(&mut proof.as_bytes()).map_err(|_| {
			revert("Failed to decode the proof. The proof is invalid or corrupted.")
		})?;

		let proof_checker = StorageProofChecker::new(storage_root, proof.proof)
			.map_err(|_| revert("Root Mismatch"))?;

		let value: Vec<u8> = proof_checker
			.read_entry(key.as_bytes(), None)
			.map_err(|_| revert("Invalid Proof"))?;

		Ok(value.into())
	}

	/// Verify the storage entries using the provided relay block number and proof. Return the
	/// values of the storage entries in the same order of keys, if the proof is valid and the
	/// entries exist.
	#[precompile::public("verifyEntries(uint32,bytes[],bytes[])")]
	#[precompile::public("verify_entries(uint32,bytes[],bytes[])")]
	fn verify_entries(
		handle: &mut impl PrecompileHandle,
		relay_block_number: RelayBlockNumber,
		proof: RawStorageProof,
		keys: BoundedVec<RawKey, GetArrayLimit>,
	) -> EvmResult<BoundedVec<UnboundedBytes, GetArrayLimit>> {
		let storage_root = Self::get_storage_root(handle, relay_block_number)?;

		let proof = ReadProof::decode(&mut proof.as_bytes()).map_err(|_| {
			revert("Failed to decode the proof. The proof is invalid or corrupted.")
		})?;

		let proof_checker = StorageProofChecker::new(storage_root, proof.proof)
			.map_err(|_| revert("Root Mismatch"))?;

		let mut values = Vec::new();
		for key in Vec::from(keys) {
			let value: Vec<u8> = proof_checker
				.read_entry(key.as_bytes(), None)
				.map_err(|_| revert("Invalid Proof"))?;
			values.push(value.into());
		}
		Ok(values.into())
	}

	#[precompile::public("latestRelayBlock()")]
	#[precompile::public("latest_relay_block()")]
	#[precompile::view]
	fn latest_relay_block(handle: &mut impl PrecompileHandle) -> EvmResult<RelayBlockNumber> {
		// RelayStorageRootKeys: BoundedVec<RelayBlockNumber>
		// 32 * MaxStorageRoots + 1 (for the length prefix)
		handle.record_db_read::<Runtime>(
			<Runtime as pallet_relay_storage_roots::Config>::MaxStorageRoots::get() as usize * 32
				+ 1,
		)?;

		pallet_relay_storage_roots::RelayStorageRootKeys::<Runtime>::get()
			.last()
			.cloned()
			.ok_or(revert("No relay block number found"))
	}

	/// Returns the storage root at the given relay block number stored on-chain. Use the pallet
	/// `pallet_relay_storage_roots` to store the storage roots on-chain.
	fn get_storage_root(
		handle: &mut impl PrecompileHandle,
		relay_block_number: RelayBlockNumber,
	) -> EvmResult<H256> {
		// RelayStorageRoot: StorageMap<RelayBlockNumber, H256>
		// twox_64(8) + key(4) + value(32)
		handle.record_db_read::<Runtime>(44)?;
		let storage_root =
			pallet_relay_storage_roots::RelayStorageRoot::<Runtime>::get(relay_block_number)
				.ok_or(revert(
					"Storage root is not stored on chain for the given relay block number",
				))?;
		Ok(storage_root)
	}
}
