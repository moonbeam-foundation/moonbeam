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
// use crate::{
// 	BalanceOf, Call, Config, InherentIncluded, LocalVrfOutput, NotFirstBlock, Pallet,
// 	RandomnessResult, RandomnessResults, RelayEpoch, Request, RequestType,
// };
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, Zero};
use frame_support::{
	dispatch::DispatchResult,
	pallet,
	traits::{Currency, Get, OnInitialize},
};
use frame_system::RawOrigin;
use pallet_evm::AddressMapping;
use pallet_referenda::{BalanceOf, Call};
use parity_scale_codec::alloc::string::ToString;
use parity_scale_codec::Decode;
use sp_core::{
	crypto::{ByteArray, UncheckedFrom},
	sr25519, H160, H256,
};
use sp_runtime::traits::One;
use sp_std::{mem::size_of, vec};

pub use dummy_pallet::*;
#[pallet]
pub mod dummy_pallet {
	#[pallet::pallet]
	pub struct Pallet<T>(frame_support::pallet_prelude::PhantomData<T>);
	#[pallet::config]
	pub trait Config: frame_system::Config {}
}

// TODO: make tracks in mock.rs realistic
benchmarks! {
	track_ids {/*set up*/}: {
		// search
		// let track_ids: Vec<u16> = Runtime::Tracks::tracks()
			// .into_iter()
			// .filter_map(|x| {
			// 	if let Ok(track_id) = x.0.try_into() {
			// 		Some(track_id)
			// 	} else {
			// 		None
			// 	}
			// })
			// .collect();
		// let result = Pallet::<T>::finish_fulfillment(0u64, Request {
		// 	refund_address: H160::default(),
		// 	contract_address: H160::default(),
		// 	fee: BalanceOf::<T>::zero(),
		// 	gas_limit: 100u64,
		// 	num_words: 100u8,
		// 	salt: H256::default(),
		// 	info: RequestType::Local(10u32.into()).into()
		// }, more, &H160::default(), BalanceOf::<T>::zero());
		// assert!(Pallet::<T>::requests(0u64).is_none());
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
	crate::mock::Runtime
);
