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
use xcm::v3::prelude::*;

benchmarks! {
	// This where clause allows us to create ForeignAssetTypes
	where_clause { where T::ForeignAssetType: From<Location> }
	register_foreign_asset {
		// does not really matter what we register
		let asset_type = T::ForeignAssetType::default();
		let metadata = T::AssetRegistrarMetadata::default();
		let amount = 1u32.into();
		let asset_id: T::AssetId = asset_type.clone().into();

	}: _(RawOrigin::Root, asset_type.clone(), metadata, amount, true)
	verify {
		assert_eq!(Pallet::<T>::asset_id_type(asset_id), Some(asset_type));
	}

	set_asset_units_per_second {
		// We make it dependent on the number of existing assets already
		let x in 5..100;
		for i in 0..x {
			let asset_type: T::ForeignAssetType = Location::new(
				0,
				X1(GeneralIndex(i as u128))
			).into();
			let metadata = T::AssetRegistrarMetadata::default();
			let amount = 1u32.into();
			Pallet::<T>::register_foreign_asset(
				RawOrigin::Root.into(),
				asset_type.clone(),
				metadata,
				amount,
				true
			)?;
			Pallet::<T>::set_asset_units_per_second(RawOrigin::Root.into(), asset_type.clone(), 1, i)?;
		}

		// does not really matter what we register, as long as it is different than the previous
		let asset_type = T::ForeignAssetType::default();
		let metadata = T::AssetRegistrarMetadata::default();
		let amount = 1u32.into();
		let asset_id: T::AssetId = asset_type.clone().into();
		Pallet::<T>::register_foreign_asset(
			RawOrigin::Root.into(),
			asset_type.clone(),
			metadata,
			amount,
			true
		)?;

	}: _(RawOrigin::Root, asset_type.clone(), 1, x)
	verify {
		assert!(Pallet::<T>::supported_fee_payment_assets().contains(&asset_type));
		assert_eq!(Pallet::<T>::asset_type_units_per_second(asset_type), Some(1));
	}

	change_existing_asset_type {
		// We make it dependent on the number of existing assets already
		let x in 5..100;
		for i in 0..x {
			let asset_type: T::ForeignAssetType = Location::new(0, X1(GeneralIndex(i as u128))).into();
			let metadata = T::AssetRegistrarMetadata::default();
			let amount = 1u32.into();
			Pallet::<T>::register_foreign_asset(
				RawOrigin::Root.into(),
				asset_type.clone(),
				metadata,
				amount,
				true
			)?;
			Pallet::<T>::set_asset_units_per_second(RawOrigin::Root.into(), asset_type.clone(), 1, i)?;
		}

		let new_asset_type = T::ForeignAssetType::default();
		let asset_type_to_be_changed: T::ForeignAssetType = Location::new(
			0,
			X1(GeneralIndex((x-1) as u128))
		).into();
		let asset_id_to_be_changed = asset_type_to_be_changed.into();
	}: _(RawOrigin::Root, asset_id_to_be_changed, new_asset_type.clone(), x)
	verify {
		assert_eq!(Pallet::<T>::asset_id_type(asset_id_to_be_changed), Some(new_asset_type.clone()));
		assert_eq!(Pallet::<T>::asset_type_units_per_second(&new_asset_type), Some(1));
		assert!(Pallet::<T>::supported_fee_payment_assets().contains(&new_asset_type));
	}

	remove_supported_asset {
		// We make it dependent on the number of existing assets already
		let x in 5..100;
		for i in 0..x {
			let asset_type: T::ForeignAssetType = Location::new(0, X1(GeneralIndex(i as u128))).into();
			let metadata = T::AssetRegistrarMetadata::default();
			let amount = 1u32.into();
			Pallet::<T>::register_foreign_asset(
				RawOrigin::Root.into(),
				asset_type.clone(),
				metadata,
				amount,
				true
			)?;
			Pallet::<T>::set_asset_units_per_second(RawOrigin::Root.into(), asset_type.clone(), 1, i)?;
		}
		let asset_type_to_be_removed: T::ForeignAssetType = Location::new(
			0,
			X1(GeneralIndex((x-1) as u128))
		).into();
		// We try to remove the last asset type
	}: _(RawOrigin::Root, asset_type_to_be_removed.clone(), x)
	verify {
		assert!(!Pallet::<T>::supported_fee_payment_assets().contains(&asset_type_to_be_removed));
		assert_eq!(Pallet::<T>::asset_type_units_per_second(asset_type_to_be_removed), None);
	}

	remove_existing_asset_type {
		// We make it dependent on the number of existing assets already
		// Worst case is we need to remove it from SupportedAAssetsFeePayment too
		let x in 5..100;
		for i in 0..x {
			let asset_type: T::ForeignAssetType = Location::new(0, X1(GeneralIndex(i as u128))).into();
			let metadata = T::AssetRegistrarMetadata::default();
			let amount = 1u32.into();
			Pallet::<T>::register_foreign_asset(
				RawOrigin::Root.into(),
				asset_type.clone(),
				metadata,
				amount,
				true
			)?;
			Pallet::<T>::set_asset_units_per_second(RawOrigin::Root.into(), asset_type.clone(), 1, i)?;
		}

		let asset_type_to_be_removed: T::ForeignAssetType = Location::new(
			0,
			X1(GeneralIndex((x-1) as u128))
		).into();
		let asset_id: T::AssetId = asset_type_to_be_removed.clone().into();
	}: _(RawOrigin::Root, asset_id, x)
	verify {
		assert!(Pallet::<T>::asset_id_type(asset_id).is_none());
		assert!(Pallet::<T>::asset_type_units_per_second(&asset_type_to_be_removed).is_none());
		assert!(!Pallet::<T>::supported_fee_payment_assets().contains(&asset_type_to_be_removed));
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
