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
	BalanceOf, Call, Config, InherentIncluded, LocalVrfOutput, NotFirstBlock, Pallet,
	RandomnessResult, RandomnessResults, RelayEpoch, Request, RequestType,
};
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, Zero};
use frame_support::{
	dispatch::DispatchResult,
	traits::{Currency, Get, OnInitialize},
};
use frame_system::RawOrigin;
use nimbus_primitives::{digests::CompatibleDigestItem as NimbusDigest, NimbusId};
use pallet_evm::AddressMapping;
use parity_scale_codec::alloc::string::ToString;
use parity_scale_codec::Decode;
use scale_info::prelude::string::String;
use session_keys_primitives::{
	digest::CompatibleDigestItem as VrfDigest, KeysLookup, PreDigest, VrfId,
};
use sp_core::{
	crypto::{ByteArray, UncheckedFrom},
	sr25519, H160, H256,
};
use sp_runtime::traits::One;
use sp_std::{mem::size_of, vec};

/// Create a funded user from the input
fn fund_user<T: Config>(user: H160, fee: BalanceOf<T>) {
	let total_minted = fee + <<T as Config>::Deposit as Get<BalanceOf<T>>>::get();
	T::Currency::make_free_balance_be(&T::AddressMapping::into_account_id(user), total_minted);
	T::Currency::issue(total_minted);
}

benchmarks! {
	where_clause {
		where <T::VrfKeyLookup as KeysLookup<NimbusId, VrfId>>::Account: From<T::AccountId>
	}
	// Benchmark for inherent included in every block
	set_babe_randomness_results {
		// set the current relay epoch as 9, `get_epoch_index` configured to return 10
		const BENCHMARKING_OLD_EPOCH: u64 = 9u64;
		RelayEpoch::<T>::put(BENCHMARKING_OLD_EPOCH);
		let benchmarking_babe_output = T::Hash::default();
		let benchmarking_new_epoch = BENCHMARKING_OLD_EPOCH.saturating_add(1u64);
		RandomnessResults::<T>::insert(
			RequestType::BabeEpoch(benchmarking_new_epoch),
			RandomnessResult::new()
		);
	}: _(RawOrigin::None)
	verify {
		// verify randomness result
		assert_eq!(
			RandomnessResults::<T>::get(
				RequestType::BabeEpoch(benchmarking_new_epoch)
			).unwrap().randomness,
			Some(benchmarking_babe_output)
		);
		assert!(InherentIncluded::<T>::get().is_some());
		assert_eq!(
			RelayEpoch::<T>::get(),
			benchmarking_new_epoch
		);
	}

	// Benchmark for VRF verification and everything else in `set_output`, in `on_initialize`
	on_initialize {
		fn decode_32_bytes(input: String) -> [u8; 32] {
			let output = hex::decode(input).expect("expect to decode input");
			let mut ret: [u8; 32] = Default::default();
			ret.copy_from_slice(&output[0..32]);
			ret
		}
		fn decode_key(input: String) -> sr25519::Public {
			sr25519::Public::unchecked_from(decode_32_bytes(input))
		}
		fn decode_pre_digest(input: String) -> PreDigest {
			let output = hex::decode(input).expect("expect to decode input");
			const PRE_DIGEST_BYTE_LEN: usize = size_of::<PreDigest>() as usize;
			let mut ret: [u8; PRE_DIGEST_BYTE_LEN] = [0u8; PRE_DIGEST_BYTE_LEN];
			ret.copy_from_slice(&output[0..PRE_DIGEST_BYTE_LEN]);
			Decode::decode(&mut ret.as_slice()).expect("expect to decode predigest")
		}
		// Uses values from moonbase alpha storage
		let raw_nimbus_id = "e0d47c4ea4fb92a510327774bd829d85ec64c06e63b3274587dfa4282f0b8262"
			.to_string();
		let raw_vrf_id = "e01d4eb5b3c482df465513ecf17f74154005ed7466166e7d2f049e0fa371ef66"
			.to_string();
		let raw_vrf_input = "33b52f1733a67e1a6b5c62c8be4b8e7be33f019a429fbc8af3336465ab042411"
			.to_string();
		let raw_vrf_pre_digest = "2a2f65f1a132c41fb33f45a282808a46fda89c91575e633fb54c913ad2ef05408\
		27bce4f8dd838e6d0acadb111c7570aaeb37340db4756f822c6d00705b2cd0165059925634e78e936bf29bb149a\
		60e8f171ea8116d035236525293efbe19703".to_string();
		let nimbus_id: NimbusId = decode_key(raw_nimbus_id).into();
		let vrf_id: VrfId = decode_key(raw_vrf_id).into();
		let vrf_input: [u8; 32] = decode_32_bytes(raw_vrf_input);
		let vrf_pre_digest = decode_pre_digest(raw_vrf_pre_digest);
		let last_vrf_output: T::Hash = Decode::decode(&mut vrf_input.as_slice()).ok()
			.expect("decode into same type");
		LocalVrfOutput::<T>::put(Some(last_vrf_output));
		NotFirstBlock::<T>::put(());
		let block_num: T::BlockNumber = frame_system::Pallet::<T>::block_number() + 100u32.into();
		RandomnessResults::<T>::insert(
			RequestType::Local(block_num),
			RandomnessResult::new().increment_request_count()
		);
		let transcript = make_transcript::<T::Hash>(LocalVrfOutput::<T>::get().unwrap_or_default());
		let nimbus_digest_item = NimbusDigest::nimbus_pre_digest(nimbus_id.clone());
		let vrf_digest_item = VrfDigest::vrf_pre_digest(vrf_pre_digest.clone());
		let digest =  sp_runtime::generic::Digest {
			logs: vec![nimbus_digest_item, vrf_digest_item]
		};
		// insert digest into frame_system storage
		frame_system::Pallet::<T>::initialize(
			&block_num,
			&T::Hash::default(),
			&digest
		);
		// set keys in author mapping
		let dummy_account: T::AccountId = account("key", 0u32, 0u32);
		T::VrfKeyLookup::set_keys(nimbus_id, dummy_account.into(), vrf_id.clone());
	}: {
		Pallet::<T>::on_initialize(block_num);
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
		assert_eq!(
			RandomnessResults::<T>::get(RequestType::Local(block_num)).unwrap().randomness,
			Some(randomness_output)
		);
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
		let x in 1..T::MaxRandomWords::get().into();
		let more = <<T as Config>::Deposit as Get<BalanceOf<T>>>::get();
		fund_user::<T>(H160::default(), more);
		let result = Pallet::<T>::request_randomness(Request {
			refund_address: H160::default(),
			contract_address: H160::default(),
			fee: BalanceOf::<T>::zero(),
			gas_limit: 100u64,
			num_words: x as u8,
			salt: H256::default(),
			info: RequestType::Local(10u32.into())
		});
		let mut result = <RandomnessResults<T>>::get(RequestType::Local(10u32.into())).unwrap();
		result.randomness = Some(Default::default());
		RandomnessResults::<T>::insert(RequestType::Local(10u32.into()), result);
		frame_system::Pallet::<T>::set_block_number(10u32.into());
	}: {
		let fulfillment_args = Pallet::<T>::prepare_fulfillment(0u64);
		assert!(fulfillment_args.is_ok(), "Prepare Fulfillment Failed");
		assert_eq!(fulfillment_args.unwrap().randomness.len(), x as usize);
	}
	verify {}

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
		let result = Pallet::<T>::increase_request_fee(
			&H160::default(),
			0u64,
			BalanceOf::<T>::one()
		);
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
