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

use crate::{AssetStatus, Call, Config, Pallet};
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite};
use frame_support::pallet_prelude::*;
use frame_system::RawOrigin;
use sp_runtime::traits::ConstU32;
use sp_runtime::BoundedVec;
use xcm::latest::prelude::*;

fn create_n_foreign_asset<T: Config>(n: u32) -> DispatchResult {
	for i in 1..=n {
		Pallet::<T>::create_foreign_asset(
			RawOrigin::Root.into(),
			i as u128,
			location_of(i),
			18,
			str_to_bv(&format!("MT{}", i)),
			str_to_bv(&format!("Mytoken{}", i)),
		)?;
		assert_eq!(Pallet::<T>::assets_by_id(i as u128), Some(location_of(i)));
	}

	Ok(())
}

fn location_of(n: u32) -> Location {
	Location::new(0, [Junction::GeneralIndex(n as u128)])
}

fn str_to_bv(str_: &str) -> BoundedVec<u8, ConstU32<256>> {
	str_.as_bytes().to_vec().try_into().expect("too long")
}

benchmarks! {
	// Worst case scenario: MaxForeignAssets minus one already exists
	create_foreign_asset {
		create_n_foreign_asset::<T>(T::MaxForeignAssets::get().saturating_sub(1))?;
		let asset_id = T::MaxForeignAssets::get() as u128;
	}: _(RawOrigin::Root, asset_id, Location::parent(), 18, str_to_bv("MT"), str_to_bv("Mytoken"))
	verify {
		assert_eq!(
			Pallet::<T>::assets_by_id(asset_id),
			Some(Location::parent())
		);
	}

	// Worst case scenario: MaxForeignAssets already exists
	change_existing_asset_type {
		create_n_foreign_asset::<T>(T::MaxForeignAssets::get())?;
	}: _(RawOrigin::Root, 1, Location::here())
	verify {
		assert_eq!(
			Pallet::<T>::assets_by_id(1),
			Some(Location::here())
		);
	}

	// Worst case scenario: MaxForeignAssets already exists
	freeze_foreign_asset {
		create_n_foreign_asset::<T>(T::MaxForeignAssets::get())?;
	}: _(RawOrigin::Root, 1, true)
	verify {
		assert_eq!(
			Pallet::<T>::assets_by_location(location_of(1)),
			Some((1, AssetStatus::FrozenXcmDepositAllowed))
		);
	}

	// Worst case scenario:
	// - MaxForeignAssets already exists
	// - The asset to unfreeze is already frozen (to avoid early error)
	unfreeze_foreign_asset {
		create_n_foreign_asset::<T>(T::MaxForeignAssets::get())?;
		Pallet::<T>::freeze_foreign_asset(
			RawOrigin::Root.into(),
			1,
			true
		)?;
		assert_eq!(
			Pallet::<T>::assets_by_location(location_of(1)),
			Some((1, AssetStatus::FrozenXcmDepositAllowed))
		);
	}: _(RawOrigin::Root, 1)
	verify {
		assert_eq!(
			Pallet::<T>::assets_by_location(location_of(1)),
			Some((1, AssetStatus::Active))
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
