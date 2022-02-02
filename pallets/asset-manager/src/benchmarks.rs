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

use crate::{Call, Config, Pallet};
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite};
use frame_system::RawOrigin;
use xcm::latest::prelude::*;

benchmarks! {
	// This where clause allows us to create assetTypes
	where_clause { where T::AssetType: From<MultiLocation> }
	register_asset {
		// does not really matter what we register
		let asset_type = T::AssetType::default();
		let metadata = T::AssetRegistrarMetadata::default();
		let amount = 1u32.into();
		let asset_id: T::AssetId = asset_type.clone().into();

	}: _(RawOrigin::Root, asset_type.clone(), metadata, amount, true)
	verify {
		assert_eq!(Pallet::<T>::asset_id_type(asset_id), Some(asset_type));
	}

	set_asset_units_per_second {
		// does not really matter what we register
		let asset_type = T::AssetType::default();
		let metadata = T::AssetRegistrarMetadata::default();
		let amount = 1u32.into();
		let asset_id: T::AssetId = asset_type.clone().into();
		Pallet::<T>::register_asset(RawOrigin::Root.into(), asset_type.clone(), metadata, amount, true)?;

	}: _(RawOrigin::Root, asset_type.clone(), 1)
	verify {
		assert_eq!(Pallet::<T>::asset_type_units_per_second(asset_type), Some(1));
	}

	change_existing_asset_type {
		// does not really matter what we register
		let asset_type = T::AssetType::default();
		let new_asset_type: T::AssetType = MultiLocation::new(
			0,
			X1(GeneralIndex(0))
		).into();

		let metadata = T::AssetRegistrarMetadata::default();
		let amount = 1u32.into();
		let asset_id: T::AssetId = asset_type.clone().into();
		Pallet::<T>::register_asset(RawOrigin::Root.into(), asset_type.clone(), metadata, amount, true)?;
		// Worst case: we also set assets units per second
		Pallet::<T>::set_asset_units_per_second(RawOrigin::Root.into(), asset_type.clone(), 1)?;

	}: _(RawOrigin::Root, asset_id, new_asset_type.clone())
	verify {
		assert_eq!(Pallet::<T>::asset_id_type(asset_id), Some(new_asset_type.clone()));
		assert_eq!(Pallet::<T>::asset_type_units_per_second(&new_asset_type), Some(1));
	}
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
