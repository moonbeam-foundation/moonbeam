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
use crate::{BalanceOf, Config, LocalVrfOutput, Pallet, RandomnessResults, Request, RequestType};
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, Zero};
use frame_support::{
	dispatch::DispatchResult,
	traits::{Currency, Get, OnInitialize},
};
use nimbus_primitives::{digests::CompatibleDigestItem as NimbusDigest, NimbusId};
use pallet_author_mapping::BenchmarkSetKeys;
use pallet_evm::AddressMapping;
use parity_scale_codec::alloc::string::ToString;
use parity_scale_codec::Decode;
use scale_info::prelude::string::String;
use session_keys_primitives::{digest::CompatibleDigestItem as VrfDigest, PreDigest, VrfId};
use sp_core::{
	crypto::{ByteArray, UncheckedFrom},
	sr25519, H160, H256,
};
use sp_runtime::traits::One;
use sp_std::vec;

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
		//  Uses moonbase alpha values from blocks
		// VRF input: 0x7d2b74fab7f37c93344abcc282d428985ddee49e494a8950c76df0342bfe6f02
		// logs: [
		// 	PreRuntime: [
		// 	  nmbs
		// 	  0x4a3017130aa08a05121b6e1b23f9db471e32da06acddba1bfa0be25c2748bb52
		// 	]
		// 	PreRuntime: [
		// 	  rand
		// 	  0x708859b63dd8284e829de2cbd90cb9b7de7eb9bec2d15ec45523e9039216fd362aacbc19ff00abbed38bac4be9acebef32e5d5ab4b1b3aa60bcbc7ca3b22880824f93050f4d51798a0ea41e8be7f0e1a541b60df46103f7785a9129e425da107
		// 	]
		// VRFId 0x4ceed6a5aaa5377723234853ab0862d24de182b4cc66302f94229888f84adc7b
		fn hex_decode(input: String) -> [u8; 32] {
			let output = hex::decode(input).expect("expect to decode input");
			let mut ret: [u8; 32] = Default::default();
			ret.copy_from_slice(&output[0..32]);
			ret
		}
		fn predigest_decode(input: String) -> PreDigest {
			let output = hex::decode(input).expect("expect to decode input");
			let mut ret: [u8; 64] = [0u8; 64];
			ret.copy_from_slice(&output[0..64]);
			Decode::decode(&mut ret.as_slice()).expect("expect to decode predigest")
		}
		let nimbus_id: NimbusId = sr25519::Public::unchecked_from(hex_decode("4a3017130aa08a05121b6e1b23f9db471e32da06acddba1bfa0be25c2748bb52".to_string())).into();
		let vrf_id: VrfId = sr25519::Public::unchecked_from(hex_decode("4ceed6a5aaa5377723234853ab0862d24de182b4cc66302f94229888f84adc7b".to_string())).into();
		let vrf_input: [u8; 32] = hex_decode("7d2b74fab7f37c93344abcc282d428985ddee49e494a8950c76df0342bfe6f02".to_string());
		let vrf_pre_digest = predigest_decode("708859b63dd8284e829de2cbd90cb9b7de7eb9bec2d15ec45523e9039216fd362aacbc19ff00abbed38bac4be9acebef32e5d5ab4b1b3aa60bcbc7ca3b22880824f93050f4d51798a0ea41e8be7f0e1a541b60df46103f7785a9129e425da107".to_string());
		let last_vrf_output: T::Hash = Decode::decode(&mut vrf_input.as_slice()).ok().expect("decode into same type");
		LocalVrfOutput::<T>::put(Some(last_vrf_output));
		let transcript = make_transcript::<T::Hash>(LocalVrfOutput::<T>::get().unwrap_or_default());

		let nimbus_digest_item = NimbusDigest::nimbus_pre_digest(nimbus_id.clone());
		let vrf_digest_item = VrfDigest::vrf_pre_digest(vrf_pre_digest.clone());
		let digest =  sp_runtime::generic::Digest { logs: vec![nimbus_digest_item, vrf_digest_item] };
		// insert digest into frame_system storage
		frame_system::Pallet::<T>::initialize(&T::BlockNumber::default(), &T::Hash::default(), &digest);
		// set keys in author mapping
		T::KeySetter::benchmark_set_keys(nimbus_id, account("key", 0u32, 0u32), vrf_id.clone());
	}: {
		Pallet::<T>::on_initialize(T::BlockNumber::default());
	}
	verify {
		// verify VrfOutput was inserted into storage as expected
		let pubkey = sp_consensus_vrf::schnorrkel::PublicKey::from_bytes(vrf_id.as_slice())
			.expect("Expect VrfId is valid schnorrkel Public key");
		let vrf_output: sp_consensus_vrf::schnorrkel::Randomness = vrf_pre_digest.vrf_output
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
