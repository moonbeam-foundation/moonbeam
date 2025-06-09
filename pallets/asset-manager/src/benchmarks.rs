// Copyright 2019-2025 PureStake Inc.
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
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use xcm::v3::prelude::*;

#[benchmarks(
	where T::ForeignAssetType: From<Location>
)]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn register_foreign_asset() -> Result<(), BenchmarkError> {
		// does not really matter what we register
		let asset_type = T::ForeignAssetType::default();
		let metadata = T::AssetRegistrarMetadata::default();
		let amount = 1u32.into();
		let asset_id: T::AssetId = asset_type.clone().into();

		#[extrinsic_call]
		_(RawOrigin::Root, asset_type.clone(), metadata, amount, true);

		assert_eq!(Pallet::<T>::asset_id_type(asset_id), Some(asset_type));
		Ok(())
	}

	#[benchmark]
	fn change_existing_asset_type() -> Result<(), BenchmarkError> {
		let asset_type: T::ForeignAssetType = Location::new(0, X1(GeneralIndex(1 as u128))).into();
		let metadata = T::AssetRegistrarMetadata::default();
		let amount = 1u32.into();
		Pallet::<T>::register_foreign_asset(
			RawOrigin::Root.into(),
			asset_type.clone(),
			metadata,
			amount,
			true,
		)?;

		let new_asset_type = T::ForeignAssetType::default();
		let asset_id_to_be_changed = asset_type.clone().into();

		#[extrinsic_call]
		_(
			RawOrigin::Root,
			asset_id_to_be_changed,
			new_asset_type.clone(),
			1,
		);

		assert_eq!(
			Pallet::<T>::asset_id_type(asset_id_to_be_changed),
			Some(new_asset_type.clone())
		);
		assert_eq!(
			Pallet::<T>::asset_type_id(new_asset_type.clone()),
			Some(asset_id_to_be_changed)
		);
		assert!(Pallet::<T>::asset_type_id(asset_type).is_none());
		Ok(())
	}

	#[benchmark]
	fn remove_existing_asset_type() -> Result<(), BenchmarkError> {
		let asset_type: T::ForeignAssetType = Location::new(0, X1(GeneralIndex(1 as u128))).into();
		let metadata = T::AssetRegistrarMetadata::default();
		let amount = 1u32.into();
		Pallet::<T>::register_foreign_asset(
			RawOrigin::Root.into(),
			asset_type.clone(),
			metadata,
			amount,
			true,
		)?;
		let asset_id: T::AssetId = asset_type.clone().into();

		#[extrinsic_call]
		_(RawOrigin::Root, asset_id, 1);

		assert!(Pallet::<T>::asset_id_type(asset_id).is_none());
		assert!(Pallet::<T>::asset_type_id(asset_type).is_none());
		Ok(())
	}

	impl_benchmark_test_suite!(
		Pallet,
		crate::benchmarks::tests::new_test_ext(),
		crate::mock::Test
	);
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
