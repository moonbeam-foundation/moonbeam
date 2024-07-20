// Copyright Moonsong Labs
// This file is part of Moonkit.

// Moonkit is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Moonkit is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Moonkit.  If not, see <http://www.gnu.org/licenses/>.

#![cfg(feature = "runtime-benchmarks")]

use crate::{Call, Config, Pallet};
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite};
use frame_system::RawOrigin;
use sp_runtime::traits::ConstU32;
use sp_runtime::BoundedVec;
use xcm::latest::prelude::*;

fn str_to_bv(str_: &str) -> BoundedVec<u8, ConstU32<256>> {
	str_.as_bytes().to_vec().try_into().expect("too long")
}

benchmarks! {
	create_foreign_asset {
	}: _(RawOrigin::Root, 1, Location::parent(), 18, str_to_bv("MT"), str_to_bv("Mytoken"))
	verify {
		assert_eq!(
			Pallet::<T>::assets_by_id(1),
			Some(Location::parent())
		);
	}

	change_existing_asset_type {
		Pallet::<T>::create_foreign_asset(
			RawOrigin::Root.into(),
			1,
			Location::parent(),
			18,
			str_to_bv("MT"),
			str_to_bv("Mytoken")
		)?;

		assert_eq!(
			Pallet::<T>::assets_by_id(1),
			Some(Location::parent())
		);
	}: _(RawOrigin::Root, 1, Location::here())
	verify {
		assert_eq!(
			Pallet::<T>::assets_by_id(1),
			Some(Location::here())
		);
	}
}

#[cfg(test)]
mod tests {
	use crate::mock::Test;
	use sp_io::TestExternalities;
	use sp_runtime::BuildStorage;

	pub fn new_test_ext() -> TestExternalities {
		let t = frame_system::GenesisConfig::<Test>::default()
			.build_storage()
			.unwrap();
		TestExternalities::new(t)
	}
}

impl_benchmark_test_suite!(
	Pallet,
	crate::benchmarks::tests::new_test_ext(),
	crate::mock::Test
);
