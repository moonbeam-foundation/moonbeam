// Copyright 2025 Moonbeam Foundation.
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
use frame_benchmarking::v2::*;
use frame_support::pallet_prelude::*;
use frame_system::RawOrigin;
use sp_runtime::traits::ConstU32;
use sp_runtime::BoundedVec;
use xcm::latest::prelude::*;

fn location_of(n: u128) -> Location {
	Location::new(0, [Junction::GeneralIndex(n)])
}

fn str_to_bv(str_: &str) -> BoundedVec<u8, ConstU32<256>> {
	str_.as_bytes().to_vec().try_into().expect("too long")
}

#[benchmarks(
	where T: Config + pallet_ethereum::Config
)]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn create_foreign_asset() -> Result<(), BenchmarkError> {
		let max_assets = T::MaxForeignAssets::get() as u128;

		for i in 1..max_assets {
			let symbol = sp_runtime::format!("MT{}", i);
			let name = sp_runtime::format!("Mytoken{}", i);
			Pallet::<T>::create_foreign_asset(
				RawOrigin::Root.into(),
				i,
				location_of(i),
				18,
				str_to_bv(&symbol),
				str_to_bv(&name),
			)?;
		}

		let asset_id = max_assets;
		let symbol = sp_runtime::format!("MT{}", asset_id);
		let name = sp_runtime::format!("Mytoken{}", asset_id);

		#[extrinsic_call]
		_(
			RawOrigin::Root,
			asset_id,
			Location::parent(),
			18,
			str_to_bv(&symbol),
			str_to_bv(&name),
		);

		assert_eq!(
			Pallet::<T>::assets_by_id(asset_id),
			Some(Location::parent())
		);

		Ok(())
	}

	#[benchmark]
	fn change_xcm_location() -> Result<(), BenchmarkError> {
		let max_assets = T::MaxForeignAssets::get() as u128;
		for i in 1..=max_assets {
			let symbol = sp_runtime::format!("MT{}", i);
			let name = sp_runtime::format!("Mytoken{}", i);
			Pallet::<T>::create_foreign_asset(
				RawOrigin::Root.into(),
				i,
				location_of(i),
				18,
				str_to_bv(&symbol),
				str_to_bv(&name),
			)?;
		}

		let asset_id = max_assets;

		#[extrinsic_call]
		_(RawOrigin::Root, asset_id, Location::here());

		assert_eq!(Pallet::<T>::assets_by_id(asset_id), Some(Location::here()));

		Ok(())
	}

	#[benchmark]
	fn freeze_foreign_asset() -> Result<(), BenchmarkError> {
		let max_assets = T::MaxForeignAssets::get() as u128;
		for i in 1..=max_assets {
			let symbol = sp_runtime::format!("MT{}", i);
			let name = sp_runtime::format!("Mytoken{}", i);
			Pallet::<T>::create_foreign_asset(
				RawOrigin::Root.into(),
				i,
				location_of(i),
				18,
				str_to_bv(&symbol),
				str_to_bv(&name),
			)?;
		}

		let asset_id = max_assets;

		#[extrinsic_call]
		_(RawOrigin::Root, asset_id, true);

		assert_eq!(
			Pallet::<T>::assets_by_location(location_of(asset_id)),
			Some((asset_id, AssetStatus::FrozenXcmDepositAllowed))
		);

		Ok(())
	}

	#[benchmark]
	fn unfreeze_foreign_asset() -> Result<(), BenchmarkError> {
		let max_assets = T::MaxForeignAssets::get() as u128;
		for i in 1..=max_assets {
			let symbol = sp_runtime::format!("MT{}", i);
			let name = sp_runtime::format!("Mytoken{}", i);
			Pallet::<T>::create_foreign_asset(
				RawOrigin::Root.into(),
				i,
				location_of(i),
				18,
				str_to_bv(&symbol),
				str_to_bv(&name),
			)?;

			let _ = Pallet::<T>::freeze_foreign_asset(RawOrigin::Root.into(), i, true);
		}

		let asset_id = max_assets;

		#[extrinsic_call]
		_(RawOrigin::Root, asset_id);

		assert_eq!(
			Pallet::<T>::assets_by_location(location_of(asset_id)),
			Some((asset_id, AssetStatus::Active))
		);

		Ok(())
	}

	impl_benchmark_test_suite! {
		Pallet,
		crate::benchmarks::tests::new_test_ext(),
		crate::mock::Test
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
