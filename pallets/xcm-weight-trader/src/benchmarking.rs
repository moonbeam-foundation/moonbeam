// Copyright 2024 Moonbeam foundation
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

use super::*;

use frame_benchmarking::{v2::*, BenchmarkError};
use frame_support::traits::EnsureOrigin;
use frame_system::EventRecord;

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
	let events = frame_system::Pallet::<T>::events();
	let system_event: <T as frame_system::Config>::RuntimeEvent = generic_event.into();
	// compare to the last event record
	let EventRecord { event, .. } = &events[events.len() - 1];
	assert_eq!(event, &system_event);
}

fn setup_one_asset<T: Config>() -> Result<Location, BenchmarkError> {
	let origin = T::AddSupportedAssetOrigin::try_successful_origin()
		.map_err(|_| BenchmarkError::Weightless)?;

	let location = T::NotFilteredLocation::get();

	Pallet::<T>::add_asset(origin, location.clone(), 1_000).expect("fail to setup asset");

	Ok(location)
}

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn add_asset() -> Result<(), BenchmarkError> {
		let origin = T::AddSupportedAssetOrigin::try_successful_origin()
			.map_err(|_| BenchmarkError::Weightless)?;

		let location = T::NotFilteredLocation::get();

		#[extrinsic_call]
		_(origin as T::RuntimeOrigin, location.clone(), 1_000);

		assert_last_event::<T>(
			Event::SupportedAssetAdded {
				location,
				units_for_one_billion_native: 1_000,
			}
			.into(),
		);
		Ok(())
	}

	#[benchmark]
	fn edit_asset() -> Result<(), BenchmarkError> {
		// Setup one asset
		let location = setup_one_asset::<T>()?;

		let origin = T::EditSupportedAssetOrigin::try_successful_origin()
			.map_err(|_| BenchmarkError::Weightless)?;

		#[extrinsic_call]
		_(origin as T::RuntimeOrigin, location.clone(), 2_000);

		assert_last_event::<T>(
			Event::SupportedAssetEdited {
				location,
				units_for_one_billion_native: 2_000,
			}
			.into(),
		);
		Ok(())
	}

	#[benchmark]
	fn resume_asset_support() -> Result<(), BenchmarkError> {
		// Setup one asset
		let location = setup_one_asset::<T>()?;
		let pause_origin = T::PauseSupportedAssetOrigin::try_successful_origin()
			.map_err(|_| BenchmarkError::Weightless)?;
		Pallet::<T>::pause_asset_support(pause_origin, location.clone())
			.expect("fail to pause asset");

		let origin = T::ResumeSupportedAssetOrigin::try_successful_origin()
			.map_err(|_| BenchmarkError::Weightless)?;

		#[extrinsic_call]
		_(origin as T::RuntimeOrigin, location.clone());

		assert_last_event::<T>(Event::ResumeAssetSupport { location }.into());
		Ok(())
	}

	#[benchmark]
	fn pause_asset_support() -> Result<(), BenchmarkError> {
		// Setup one asset
		let location = setup_one_asset::<T>()?;

		let origin = T::PauseSupportedAssetOrigin::try_successful_origin()
			.map_err(|_| BenchmarkError::Weightless)?;

		#[extrinsic_call]
		_(origin as T::RuntimeOrigin, location.clone());

		assert_last_event::<T>(Event::PauseAssetSupport { location }.into());
		Ok(())
	}

	#[benchmark]
	fn remove_asset() -> Result<(), BenchmarkError> {
		// Setup one asset
		let location = setup_one_asset::<T>()?;

		let origin = T::RemoveSupportedAssetOrigin::try_successful_origin()
			.map_err(|_| BenchmarkError::Weightless)?;

		#[extrinsic_call]
		_(origin as T::RuntimeOrigin, location.clone());

		assert_last_event::<T>(Event::SupportedAssetRemoved { location }.into());
		Ok(())
	}

	impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test,);
}
