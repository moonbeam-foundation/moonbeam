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
use frame_benchmarking::{account, benchmarks};
use frame_support::traits::Currency;
use frame_system::RawOrigin;
use sp_core::{Get, U256};
use sp_runtime::traits::StaticLookup;
use sp_runtime::Saturating;
use sp_std::vec::Vec;
use xcm::latest::prelude::*;

fn setup_foreign_asset<T: Config>(n_accounts: u32) -> T::AssetIdParameter {
	let asset_type = T::ForeignAssetType::default();
	let metadata = T::AssetRegistrarMetadata::default();
	let asset_id = asset_type.clone().into();

	let caller: T::AccountId = pallet_asset_manager::Pallet::<T>::account_id();
	let caller_lookup = T::Lookup::unlookup(caller.clone());
	let root: T::RuntimeOrigin = RawOrigin::Root.into();

	// Register in asset manager
	let _ = pallet_asset_manager::Pallet::<T>::register_foreign_asset(
		root.clone(),
		asset_type,
		metadata,
		<T as pallet_asset_manager::Config>::Balance::from(1u32),
		true,
	)
	.unwrap();

	let _ = <T as pallet_assets::Config>::Currency::deposit_creating(
		&caller,
		<T as pallet_assets::Config>::MetadataDepositBase::get()
			.saturating_add(
				<T as pallet_assets::Config>::MetadataDepositPerByte::get()
					.saturating_mul((T::StringLimit::get() as u32).into()),
			)
			.saturating_mul(2u32.into()),
	);

	let dummy = Vec::from_iter((0..T::StringLimit::get() as usize).map(|_| 0u8));
	let _ = pallet_assets::Pallet::<T>::set_metadata(
		RawOrigin::Signed(caller.clone()).into(),
		asset_id.clone().into(),
		dummy.clone(),
		dummy,
		18,
	)
	.unwrap();

	// Create approval
	pallet_assets::Pallet::<T>::mint(
		RawOrigin::Signed(caller.clone()).into(),
		asset_id.clone().into(),
		caller_lookup,
		(100 * (n_accounts + 1)).into(),
	)
	.unwrap();

	// Setup n accounts with balances and approvals
	for i in 0..n_accounts {
		let user: T::AccountId = account("user", i, 0);
		let user_lookup = T::Lookup::unlookup(user.clone());

		// Mint assets
		let _ = pallet_assets::Pallet::<T>::mint(
			RawOrigin::Signed(caller.clone()).into(),
			asset_id.clone().into(),
			user_lookup,
			100u32.into(),
		)
		.unwrap();

		let spender: T::AccountId = account("spender", i, 0);
		let spender_lookup = T::Lookup::unlookup(spender.clone());
		let enough = <T as pallet_assets::Config>::Currency::minimum_balance();
		<T as pallet_assets::Config>::Currency::make_free_balance_be(&spender, enough);

		let _ = pallet_assets::Pallet::<T>::approve_transfer(
			RawOrigin::Signed(caller.clone()).into(),
			asset_id.clone().into(),
			spender_lookup,
			5u32.into(),
		)
		.unwrap();
	}

	asset_id.into()
}

benchmarks! {
	where_clause {
		where
		<T as pallet_assets::Config>::Balance: Into<U256>,
		T::ForeignAssetType: Into<Option<Location>>,
	}
	start_foreign_assets_migration {
		let asset_id = setup_foreign_asset::<T>(1);
	}: _(RawOrigin::Root, asset_id.into())
	verify {
		assert!(matches!(
			crate::pallet::ForeignAssetMigrationStatusValue::<T>::get(),
			ForeignAssetMigrationStatus::Migrating(_)
		));
	}

	migrate_foreign_asset_balances {
		let n in 1 .. 1000u32;
		let asset_id = setup_foreign_asset::<T>(n);
		Pallet::<T>::start_foreign_assets_migration(
			RawOrigin::Root.into(),
			asset_id.into()
		)?;
	}: _(RawOrigin::Signed(account("caller", 0, 0)), n + 1)
	verify {
		if let ForeignAssetMigrationStatus::Migrating(info) = crate::pallet::ForeignAssetMigrationStatusValue::<T>::get()  {
			assert_eq!(info.remaining_balances, 0);
		}
	}

	migrate_foreign_asset_approvals {
		let n in 1 .. 1000u32;
		let asset_id = setup_foreign_asset::<T>(n);
		Pallet::<T>::start_foreign_assets_migration(
			RawOrigin::Root.into(),
			asset_id.into()
		)?;
		Pallet::<T>::migrate_foreign_asset_balances(
			RawOrigin::Signed(account("caller", 0, 0)).into(),
			n + 1
		)?;
	}: _(RawOrigin::Signed(account("caller", 0, 0)), n)
	verify {
		if let ForeignAssetMigrationStatus::Migrating(info) = crate::pallet::ForeignAssetMigrationStatusValue::<T>::get()  {
			assert_eq!(info.remaining_approvals, 0);
		}
	}

	finish_foreign_assets_migration {
		let n = 100u32;
		let asset_id = setup_foreign_asset::<T>(n);
		Pallet::<T>::start_foreign_assets_migration(
			RawOrigin::Root.into(),
			asset_id.into()
		)?;
		Pallet::<T>::migrate_foreign_asset_balances(
			RawOrigin::Signed(account("caller", 0, 0)).into(),
			n + 1
		)?;
		Pallet::<T>::migrate_foreign_asset_approvals(
			RawOrigin::Signed(account("caller", 0, 0)).into(),
			n + 1
		)?;
	}: _(RawOrigin::Signed(account("caller", 0, 0)))
	verify {
		assert_eq!(
			crate::pallet::ForeignAssetMigrationStatusValue::<T>::get(),
			ForeignAssetMigrationStatus::Idle
		);
	}
}
