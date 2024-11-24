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
// #![cfg(feature = "runtime-benchmarks")]

use crate::{foreign_asset::ForeignAssetMigrationStatus, Call, Config, Pallet};
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::traits::Currency;
use frame_system::RawOrigin;
use sp_core::U256;
use sp_runtime::traits::StaticLookup;
use xcm::latest::prelude::*;

fn setup_foreign_asset<T: Config>(n_accounts: u32) -> (T::AssetIdParameter, Location) {
	let asset_id: T::AssetIdParameter = T::AssetIdParameter::from(1u128);
	let metadata = T::AssetRegistrarMetadata::default();
	let location = Location::new(1, [Junction::Parachain(1000)]);
	let caller: T::AccountId = whitelisted_caller();
	let caller_lookup = T::Lookup::unlookup(caller.clone());

	// Register in asset manager
	let _ = pallet_asset_manager::Pallet::<T>::register_foreign_asset(
		RawOrigin::Root.into(),
		T::ForeignAssetType::default(),
		metadata,
		<T as pallet_asset_manager::Config>::Balance::from(1u32),
		true,
	);

	// Create asset and set metadata
	let _ = pallet_assets::Pallet::<T>::force_create(
		RawOrigin::Root.into(),
		asset_id.clone(),
		caller_lookup.clone(),
		true,
		<T as pallet_assets::Config>::Balance::from(1u32),
	);

	let _ = pallet_assets::Pallet::<T>::set_metadata(
		RawOrigin::Signed(caller.clone()).into(),
		asset_id.clone(),
		b"Test".to_vec(),
		b"TEST".to_vec(),
		12,
	);

	// Setup n accounts with balances and approvals
	for i in 0..n_accounts {
		let user: T::AccountId = account("user", i, 0);
		let account_lookup = T::Lookup::unlookup(user.clone());
		// Ensure account exists
		let _ = <T as pallet_assets::Config>::Currency::deposit_creating(&user, 100u32.into());

		// Mint assets
		let _ = pallet_assets::Pallet::<T>::mint(
			RawOrigin::Signed(caller.clone()).into(),
			asset_id.clone(),
			account_lookup,
			100u32.into(),
		);

		// Create approval
		let spender: T::AccountId = account("spender", i, 0);
		let spender_lookup = T::Lookup::unlookup(spender.clone());
		let _ = pallet_assets::Pallet::<T>::approve_transfer(
			RawOrigin::Signed(user).into(),
			asset_id.clone(),
			spender_lookup,
			50u32.into(),
		);
	}

	(asset_id, location)
}

benchmarks! {
	where_clause {
		where
		<T as pallet_assets::Config>::Balance: Into<U256>,
		<T as pallet_asset_manager::Config>::ForeignAssetType: Into<Option<Location>>,
	}
	start_foreign_assets_migration {
		let n = 100u32;
		let (asset_id, _) = setup_foreign_asset::<T>(n);
	}: _(RawOrigin::Root, asset_id.into())
	verify {
		assert!(matches!(
			crate::pallet::ForeignAssetMigrationStatusValue::<T>::get(),
			ForeignAssetMigrationStatus::Migrating(_)
		));
	}

	migrate_foreign_asset_balances {
		let n in 1 .. 1000u32;
		let (asset_id, _) = setup_foreign_asset::<T>(n);
		Pallet::<T>::start_foreign_assets_migration(
			RawOrigin::Root.into(),
			asset_id.into()
		)?;
	}: _(RawOrigin::Signed(account("caller", 0, 0)), n as u64)
	verify {
		if let ForeignAssetMigrationStatus::Migrating(info) = crate::pallet::ForeignAssetMigrationStatusValue::<T>::get()  {
			assert_eq!(info.remaining_balances, 0);
		}
	}

	migrate_foreign_asset_approvals {
		let n in 1 .. 1000u32;
		let (asset_id, _) = setup_foreign_asset::<T>(n);
		Pallet::<T>::start_foreign_assets_migration(
			RawOrigin::Root.into(),
			asset_id.into()
		)?;
		Pallet::<T>::migrate_foreign_asset_balances(
			RawOrigin::Signed(account("caller", 0, 0)).into(),
			n as u64
		)?;
	}: _(RawOrigin::Signed(account("caller", 0, 0)), n as u64)
	verify {
		if let ForeignAssetMigrationStatus::Migrating(info) = crate::pallet::ForeignAssetMigrationStatusValue::<T>::get()  {
			assert_eq!(info.remaining_approvals, 0);
		}
	}

	finish_foreign_assets_migration {
		let n = 100u32;
		let (asset_id, _) = setup_foreign_asset::<T>(n);
		Pallet::<T>::start_foreign_assets_migration(
			RawOrigin::Root.into(),
			asset_id.into()
		)?;
		Pallet::<T>::migrate_foreign_asset_balances(
			RawOrigin::Signed(account("caller", 0, 0)).into(),
			n as u64
		)?;
		Pallet::<T>::migrate_foreign_asset_approvals(
			RawOrigin::Signed(account("caller", 0, 0)).into(),
			n as u64
		)?;
	}: _(RawOrigin::Signed(account("caller", 0, 0)))
	verify {
		assert_eq!(
			crate::pallet::ForeignAssetMigrationStatusValue::<T>::get(),
			ForeignAssetMigrationStatus::Idle
		);
	}
}

impl_benchmark_test_suite!(
	Pallet,
	crate::mock::ExtBuilder::default().build(),
	crate::mock::Test
);
