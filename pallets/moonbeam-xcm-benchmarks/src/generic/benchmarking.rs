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

use super::*;
use frame_benchmarking::benchmarks;
use pallet_xcm_benchmarks::{new_executor, XcmCallOf};
use sp_std::vec;
use sp_std::vec::Vec;
use xcm::latest::prelude::*;

benchmarks! {
	buy_execution {
		let holding = T::worst_case_holding().into();

		let mut executor = new_executor::<T>(Default::default());
		executor.holding = holding;

		let fee_asset = Concrete(MultiLocation::parent());

		let instruction = Instruction::<XcmCallOf<T>>::BuyExecution {
			fees: (fee_asset, 100_000_000).into(), // should be something inside of holding
			weight_limit: WeightLimit::Limited(1u64),
		};

		let xcm = Xcm(vec![instruction]);

	} : {
		executor.execute(xcm)?;
	} verify {

	}

	impl_benchmark_test_suite!(
		Pallet,
		crate::generic::mock::new_test_ext(),
		crate::generic::mock::Test
	);

}

pub struct XcmGenericBenchmarks<T>(sp_std::marker::PhantomData<T>);

// We only need to implement benchmarks for the runtime-benchmarks feature or testing.
impl<T: Config> frame_benchmarking::Benchmarking for XcmGenericBenchmarks<T> {
	fn benchmarks(extra: bool) -> Vec<frame_benchmarking::BenchmarkMetadata> {
		// Assuming we are overwritting, we only need to return the generics
		use pallet_xcm_benchmarks::generic::Pallet as PalletXcmGenericBench;
		PalletXcmGenericBench::<T>::benchmarks(extra)
	}
	fn run_benchmark(
		extrinsic: &[u8],
		c: &[(frame_benchmarking::BenchmarkParameter, u32)],
		whitelist: &[frame_benchmarking::TrackedStorageKey],
		verify: bool,
		internal_repeats: u32,
	) -> Result<Vec<frame_benchmarking::BenchmarkResult>, frame_benchmarking::BenchmarkError> {
		use pallet_xcm_benchmarks::generic::Pallet as PalletXcmGenericBench;

		use crate::generic::Pallet as MoonbeamXcmGenericBench;
		if MoonbeamXcmGenericBench::<T>::benchmarks(true)
			.iter()
			.find(|&x| x.name == extrinsic)
			.is_some()
		{
			MoonbeamXcmGenericBench::<T>::run_benchmark(
				extrinsic,
				c,
				whitelist,
				verify,
				internal_repeats,
			)
		} else {
			PalletXcmGenericBench::<T>::run_benchmark(
				extrinsic,
				c,
				whitelist,
				verify,
				internal_repeats,
			)
		}
	}
}
