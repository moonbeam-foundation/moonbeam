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

#![cfg(feature = "runtime-benchmarks")]
#![allow(clippy::no_effect)]

use crate::weights::WeightInfo;
use crate::{Config, Pallet};
use core::marker::PhantomData;
use cumulus_primitives_core::relay_chain::BlockNumber as RelayBlockNumber;
use fp_evm::{Context, Precompile, PrecompileResult};
use frame_benchmarking::benchmarks;
use frame_support::{traits::Get, weights::Weight, BoundedVec};
use parity_scale_codec::{Decode, Encode};
use sp_core::H256;
use sp_std::{collections::btree_map::BTreeMap, vec, vec::Vec};

use crate::mock;

fn fill_relay_storage_roots<T: Config>() -> Vec<u32> {
	// Initialize keys BoundedVec for RelayStorageRoots
	let mut keys = BoundedVec::<u32, _>::default();
	for i in 0..T::MaxStorageRoots::get() {
		let relay_root = H256::from_low_u64_be(i as u64);
		pallet_relay_storage_roots::RelayStorageRoot::<T>::insert(i, relay_root);
		keys.try_push(i)
			.expect("Keys should not exceed MaxStorageRoots");
	}
	pallet_relay_storage_roots::RelayStorageRootKeys::<T>::put(keys.clone());
	keys.to_vec()
}

fn get_latest_relay_block<T: Config>() -> RelayBlockNumber {
	pallet_relay_storage_roots::RelayStorageRootKeys::<T>::get()
		.last()
		.cloned()
		.expect("At least one relay block should be store")
}

fn p256verify<T: Config>(input: Vec<u8>) -> PrecompileResult {
	let context: Context = Context {
		address: Default::default(),
		caller: Default::default(),
		apparent_value: From::from(0),
	};

	let mut handle = mock::MockHandle::new(input, 4000, context);

	struct P256VerifyWeight<T>(PhantomData<T>);
	impl<T: Config> Get<Weight> for P256VerifyWeight<T> {
		fn get() -> Weight {
			<T as Config>::WeightInfo::p256_verify()
		}
	}

	pallet_evm_precompile_p256verify::P256Verify::<P256VerifyWeight<T>>::execute(&mut handle)
}

fn verify_risc0_receipt(receipt: Vec<u8>, image_id: [u32; 8]) {
	let receipt: risc0_zkvm::Receipt =
		postcard::from_bytes(&receipt).expect("Receipt decoding failed");

	receipt.verify(image_id).expect("Error verifying receipt");
}

benchmarks! {
	verify_entry {
		// x is the number of nodes in the proof
		let x in 100..2000 => 100;

		let mocked_proofs: BTreeMap<u32, (H256, Vec<Vec<u8>>)> =
			BTreeMap::decode(&mut &include_bytes!("../benchmark_proofs").to_vec()[..])
			.expect("Failed to decode mocked proofs");


		// if x is not multiple of 100, we will use the proof for the closest multiple of 100
		let x = (x / 100) * 100;
		let (state_root, mocked_proof) =
			mocked_proofs.get(&x).expect("Not Found").clone();

		// Set the state root for the relay block in the relay storage roots pallet
		let relay_block = 10;
		pallet_relay_storage_roots::RelayStorageRoot::<T>::insert(relay_block, state_root);

		let key = 2u128.encode();
	}:{

		Pallet::<T>::verify_entry(state_root, mocked_proof, &key)
			.expect("Should verify the entry without error.");
	}

	latest_relay_block {
		let keys = fill_relay_storage_roots::<T>();
	}:{
		get_latest_relay_block::<T>()
	}
	verify {
		assert_eq!(
			get_latest_relay_block::<T>(), keys.last().cloned().expect("There should be at least one key")
		);
	}

	p256_verify {
		let input = vec![
			181, 167, 126, 122, 144, 170, 20, 224, 191, 95, 51, 127, 6, 245, 151, 20, 134, 118, 66,
			79, 174, 38, 225, 117, 198, 229, 98, 28, 52, 53, 25, 85, 40, 159, 49, 151, 137, 218,
			66, 72, 69, 201, 234, 201, 53, 36, 95, 205, 221, 128, 89, 80, 226, 240, 37, 6, 208,
			155, 231, 228, 17, 25, 149, 86, 210, 98, 20, 68, 117, 177, 250, 70, 173, 133, 37, 7,
			40, 198, 0, 197, 61, 253, 16, 248, 179, 244, 173, 241, 64, 226, 114, 65, 174, 195, 194,
			218, 58, 129, 4, 103, 3, 252, 207, 70, 139, 72, 177, 69, 249, 57, 239, 219, 185, 108,
			55, 134, 219, 113, 43, 49, 19, 187, 36, 136, 239, 40, 108, 220, 239, 138, 254, 130,
			210, 0, 165, 187, 54, 181, 70, 33, 102, 232, 206, 119, 242, 216, 49, 165, 46, 242, 19,
			91, 42, 241, 136, 17, 11, 234, 239, 177
		];
	}:{
		let _ = p256verify::<T>(input).expect("Should verify the signature without any errors.");
	}

	zk_auth_verify {
		let receipt = vec![
			2, 128, 2, 43, 1, 50, 120, 33, 198, 254, 149, 68, 226, 225, 17, 140, 204, 69, 20, 109, 51,
			6, 168, 103, 117, 110, 250, 109, 184, 32, 196, 171, 167, 28, 85, 5, 244, 90, 120, 165, 223,
			25, 154, 108, 104, 122, 13, 232, 213, 14, 86, 19, 132, 209, 3, 193, 25, 213, 245, 201, 14,
			171, 188, 20, 61, 87, 143, 13, 130, 207, 30, 66, 49, 130, 239, 36, 170, 56, 67, 88, 46, 28,
			54, 66, 86, 165, 95, 68, 198, 216, 117, 26, 60, 223, 118, 135, 136, 12, 136, 32, 23, 65,
			122, 191, 152, 22, 242, 244, 200, 125, 118, 171, 39, 237, 221, 111, 54, 206, 237, 239, 43,
			53, 43, 18, 34, 183, 18, 246, 121, 66, 82, 32, 197, 167, 19, 129, 173, 59, 16, 34, 242, 30,
			235, 7, 146, 132, 118, 87, 148, 253, 81, 23, 208, 7, 168, 105, 210, 52, 109, 145, 151, 150,
			81, 6, 121, 61, 178, 93, 11, 164, 87, 126, 219, 168, 114, 138, 195, 52, 220, 254, 122, 146,
			136, 48, 3, 212, 72, 75, 128, 147, 141, 67, 218, 154, 60, 46, 220, 205, 203, 69, 96, 242,
			206, 27, 86, 2, 223, 100, 121, 144, 202, 185, 119, 43, 89, 171, 139, 197, 216, 117, 193,
			198, 116, 232, 116, 247, 177, 18, 174, 114, 105, 25, 177, 73, 197, 193, 153, 220, 185, 220,
			55, 126, 49, 54, 69, 48, 178, 207, 58, 130, 2, 134, 78, 60, 202, 24, 79, 241, 245, 0, 0,
			128, 236, 128, 1, 170, 250, 213, 155, 4, 174, 236, 161, 132, 15, 140, 173, 206, 233, 7,
			239, 144, 137, 146, 13, 236, 243, 157, 148, 11, 129, 196, 248, 250, 14, 176, 134, 243, 215,
			2, 238, 254, 237, 243, 9, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0,
			1, 0, 32, 0, 228, 11, 84, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 1, 0, 0, 0, 2, 0,
			0, 0, 3, 0, 0, 0, 0, 0, 177, 158, 148, 199, 9, 232, 199, 167, 255, 2, 168, 139, 240, 189,
			2, 173, 254, 225, 194, 7, 152, 247, 174, 196, 6, 196, 171, 223, 158, 9, 141, 241, 146, 220,
			10, 170, 237, 133, 231, 5, 32, 0, 228, 11, 84, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0,
			0, 1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 177, 158, 148, 199, 9, 232, 199, 167, 255, 2, 168,
			139, 240, 189, 2, 173, 254, 225, 194, 7, 152, 247, 174, 196, 6, 196, 171, 223, 158, 9, 141,
			241, 146, 220, 10, 170, 237, 133, 231, 5,
		];

		let image_id = [
			715585636, 3586935525, 3274293606, 2872050810, 564159597, 2621011314, 3667725176, 1510137221,
		];

	}:{
		let _ = verify_risc0_receipt(receipt, image_id);
	}
}
