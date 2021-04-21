// Copyright 2019-2020 PureStake Inc.
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
use super::*;
use crate::Pallet as ParachainStaking;
use frame_support::assert_ok;
use frame_system::RawOrigin;
use frame_benchmarking::{benchmarks};
use sp_runtime::Perbill;

benchmarks! {
	set_inflation {
		let inflation_range: Range<Perbill> = Range {
			min: Perbill::from_perthousand(1),
			ideal: Perbill::from_perthousand(2),
			max: Perbill::from_perthousand(3),
		};

	}: _(RawOrigin::Root, inflation_range)
	verify {
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::mock::Test;
	use sp_io::TestExternalities;

	pub fn new_test_ext() -> TestExternalities {
		let t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
		TestExternalities::new(t)
	}

	#[test]
	fn test_benchmarks() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_set_inflation::<Test>());
		});
	}
}
