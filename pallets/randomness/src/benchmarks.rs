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

#![cfg(feature = "runtime-benchmarks")]

//! Benchmarking
use crate::vrf::*;
use crate::{
	BalanceOf, Call, Config, LocalVrfOutput, Pallet, RandomnessResults, Request, RequestType,
};
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, Zero};
use frame_support::{
	dispatch::DispatchResult,
	traits::{Currency, Get, OnInitialize},
};
use frame_system::RawOrigin;
use nimbus_primitives::{digests::CompatibleDigestItem as NimbusDigest, NimbusId};
use pallet_author_mapping::BenchmarkSetKeys;
use pallet_evm::AddressMapping;
use parity_scale_codec::Decode;
use session_keys_primitives::{
	digest::CompatibleDigestItem as VrfDigest, vrf::bench::*, PreDigest, VrfId,
};
use sp_core::{ByteArray, Pair, H160, H256};
use sp_runtime::traits::One;

/// Create a funded user from the input
fn fund_user<T: Config>(user: H160, fee: BalanceOf<T>) {
	let total_minted = fee + <<T as Config>::Deposit as Get<BalanceOf<T>>>::get();
	T::Currency::make_free_balance_be(&T::AddressMapping::into_account_id(user), total_minted);
	T::Currency::issue(total_minted);
}

benchmarks! {
	// TODO: causes panic:
	// Thread 'main' panicked at 'set in `set_validation_data`inherent => available before
	// on_initialize', runtime/moonbase/src/lib.rs:1111
	// set_babe_randomness_results {
	// 	// set relay epoch
	// 	// set storage value to read new epoch > current epoch
	// 	//
	// }: _(RawOrigin::None)
	// verify {}

	// Benchmark for VRF verification and everything else in `set_output`, in `on_initialize`
	on_initialize {
		let key_pair = mock_key_pair();
		let transcript = make_transcript::<T::Hash>(LocalVrfOutput::<T>::get().unwrap_or_default());
		let (vrf_output, vrf_proof) = mock_sign_vrf(key_pair.clone(), transcript.clone());
		let nimbus_id: NimbusId = key_pair.public().into();
		let vrf_id: VrfId = nimbus_id.clone().into();
		let nimbus_digest_item = NimbusDigest::nimbus_pre_digest(nimbus_id.clone());
		let vrf_digest_item = VrfDigest::vrf_pre_digest(
			PreDigest { vrf_output: vrf_output.clone(), vrf_proof }
		);
		let digest =  sp_runtime::generic::Digest { logs: vec![nimbus_digest_item, vrf_digest_item] };
		// insert digest into frame_system storage
		frame_system::Pallet::<T>::initialize(&T::BlockNumber::default(), &T::Hash::default(), &digest);
		// set author mapping keys
		T::KeySetter::benchmark_set_keys(nimbus_id, account("key", 0u32, 0u32), vrf_id.clone());
	}: {
		Pallet::<T>::on_initialize(T::BlockNumber::default());
	}
	verify {
		// verify VrfOutput was inserted into storage as expected
		let pubkey = sp_consensus_vrf::schnorrkel::PublicKey::from_bytes(vrf_id.as_slice())
			.expect("Expect VrfId is valid schnorrkel Public key");
		let vrf_output: sp_consensus_vrf::schnorrkel::Randomness = vrf_output
			.attach_input_hash(&pubkey, transcript)
			.ok()
			.map(|inout| inout.make_bytes(&session_keys_primitives::VRF_INOUT_CONTEXT))
			.expect("VRF output encoded in pre-runtime digest must be valid");
		let randomness_output = T::Hash::decode(&mut &vrf_output[..])
			.ok()
			.expect("VRF output bytes can be decode into T::Hash");
		// convert vrf output and check if it matches as expected
		assert_eq!(LocalVrfOutput::<T>::get(), Some(randomness_output));
	}

	request_randomness {
		let more = <<T as Config>::Deposit as Get<BalanceOf<T>>>::get();
		fund_user::<T>(H160::default(), more);
	}: {
		let result = Pallet::<T>::request_randomness(Request {
			refund_address: H160::default(),
			contract_address: H160::default(),
			fee: BalanceOf::<T>::zero(),
			gas_limit: 100u64,
			num_words: 100u8,
			salt: H256::default(),
			info: RequestType::Local(10u32.into())
		});
		assert!(result.is_ok(), "Request Randomness Failed");
	}
	verify {
		assert!(Pallet::<T>::requests(&0u64).is_some());
		assert!(Pallet::<T>::requests(&1u64).is_none());
	}

	prepare_fulfillment {
		let more = <<T as Config>::Deposit as Get<BalanceOf<T>>>::get();
		fund_user::<T>(H160::default(), more);
		let result = Pallet::<T>::request_randomness(Request {
			refund_address: H160::default(),
			contract_address: H160::default(),
			fee: BalanceOf::<T>::zero(),
			gas_limit: 100u64,
			num_words: 100u8,
			salt: H256::default(),
			info: RequestType::Local(10u32.into())
		});
		let mut result = <RandomnessResults<T>>::get(RequestType::Local(10u32.into())).unwrap();
		result.randomness = Some(Default::default());
		RandomnessResults::<T>::insert(RequestType::Local(10u32.into()), result);
		frame_system::Pallet::<T>::set_block_number(10u32.into());
	}: {
		let result = Pallet::<T>::prepare_fulfillment(0u64);
		assert!(result.is_ok(), "Prepare Fulfillment Failed");
	}
	verify { }

	finish_fulfillment {
		let more = <<T as Config>::Deposit as Get<BalanceOf<T>>>::get();
		fund_user::<T>(H160::default(), more);
		let result = Pallet::<T>::request_randomness(Request {
			refund_address: H160::default(),
			contract_address: H160::default(),
			fee: BalanceOf::<T>::zero(),
			gas_limit: 100u64,
			num_words: 100u8,
			salt: H256::default(),
			info: RequestType::Local(10u32.into())
		});
		let mut result = <RandomnessResults<T>>::get(RequestType::Local(10u32.into())).unwrap();
		result.randomness = Some(Default::default());
		RandomnessResults::<T>::insert(RequestType::Local(10u32.into()), result);
		frame_system::Pallet::<T>::set_block_number(10u32.into());
		let result = Pallet::<T>::prepare_fulfillment(0u64);
		assert!(result.is_ok(), "Prepare Fulfillment Failed");
	}: {
		let result = Pallet::<T>::finish_fulfillment(0u64, Request {
			refund_address: H160::default(),
			contract_address: H160::default(),
			fee: BalanceOf::<T>::zero(),
			gas_limit: 100u64,
			num_words: 100u8,
			salt: H256::default(),
			info: RequestType::Local(10u32.into()).into()
		}, more, &H160::default(), BalanceOf::<T>::zero());
		assert!(Pallet::<T>::requests(0u64).is_none());
	}
	verify { }

	increase_fee {
		let more = <<T as Config>::Deposit as Get<BalanceOf<T>>>::get();
		fund_user::<T>(H160::default(), more);
		let result = Pallet::<T>::request_randomness(Request {
			refund_address: H160::default(),
			contract_address: H160::default(),
			fee: BalanceOf::<T>::zero(),
			gas_limit: 100u64,
			num_words: 100u8,
			salt: H256::default(),
			info: RequestType::Local(10u32.into()).into()
		});
	}: {
		let result = Pallet::<T>::increase_request_fee(&H160::default(), 0u64, BalanceOf::<T>::one());
		assert_eq!(result, DispatchResult::Ok(()));
	}
	verify { }

	execute_request_expiration {
		let more = <<T as Config>::Deposit as Get<BalanceOf<T>>>::get();
		fund_user::<T>(H160::default(), more);
		let result = Pallet::<T>::request_randomness(Request {
			refund_address: H160::default(),
			contract_address: H160::default(),
			fee: BalanceOf::<T>::zero(),
			gas_limit: 100u64,
			num_words: 100u8,
			salt: H256::default(),
			info: RequestType::Local(10u32.into())
		});
		frame_system::Pallet::<T>::set_block_number(10_001u32.into());
	}: {
		let result = Pallet::<T>::execute_request_expiration(&H160::default(), 0u64);
		assert_eq!(result, DispatchResult::Ok(()));
	}
	verify { }
}

#[cfg(test)]
mod tests {
	use crate::mock::Test;
	use sp_io::TestExternalities;

	pub fn new_test_ext() -> TestExternalities {
		let t = frame_system::GenesisConfig::default()
			.build_storage::<Test>()
			.unwrap();
		TestExternalities::new(t)
	}
}

impl_benchmark_test_suite!(
	Pallet,
	crate::benchmarks::tests::new_test_ext(),
	crate::mock::Test
);
