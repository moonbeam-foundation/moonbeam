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

//! # Functions for handling foreign asset migrations

use super::*;
use frame_support::sp_runtime::Saturating;
use frame_support::traits::{fungibles::metadata::Inspect, ReservableCurrency};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use sp_core::U256;
use xcm_primitives::AssetTypeGetter;

#[derive(Encode, Decode, scale_info::TypeInfo, PartialEq, MaxEncodedLen)]
pub enum ForeignAssetMigrationStatus {
	/// No migration in progress
	Idle,
	/// Migrating a foreign asset in progress
	Migrating(ForeignAssetMigreationInfo),
}

impl Default for ForeignAssetMigrationStatus {
	fn default() -> Self {
		ForeignAssetMigrationStatus::Idle
	}
}

#[derive(Encode, Decode, scale_info::TypeInfo, PartialEq, MaxEncodedLen)]
pub(super) struct ForeignAssetMigreationInfo {
	asset_id: u128,
	remaining_balances: u32,
	remaining_approvals: u32,
}

impl<T: Config> Pallet<T>
where
	<T as pallet_assets::Config>::Balance: Into<U256>,
	<T as pallet_asset_manager::Config>::ForeignAssetType: Into<Option<Location>>,
{
	/// Start a foreign asset migration by freezing the asset and creating the SC with the moonbeam
	/// foreign assets pallet.
	pub(super) fn do_start_foreign_asset_migration(
		origin: OriginFor<T>,
		asset_id: u128,
	) -> DispatchResult {
		ForeignAssetMigrationStatusValue::<T>::try_mutate(|status| -> DispatchResult {
			ensure!(
				*status == ForeignAssetMigrationStatus::Idle,
				Error::<T>::MigrationNotFinished
			);

			let asset =
				pallet_assets::Asset::<T>::get(asset_id).ok_or(Error::<T>::AssetNotFound)?;

			// Freeze the asset
			pallet_assets::Pallet::<T>::freeze_asset(origin.clone(), asset_id.into())?;

			let decimals = pallet_assets::Pallet::<T>::decimals(asset_id);

			let symbol = pallet_assets::Pallet::<T>::symbol(asset_id)
				.try_into()
				.map_err(|_| Error::<T>::SymbolTooLong)?;

			let name = <pallet_assets::Pallet<T> as Inspect<_>>::name(asset_id)
				.try_into()
				.map_err(|_| Error::<T>::NameTooLong)?;

			let xcm_location: Location =
				pallet_asset_manager::Pallet::<T>::get_asset_type(asset_id)
					.ok_or(Error::<T>::AssetTypeNotFound)?
					.into()
					.ok_or(Error::<T>::LocationNotFound)?;

			// Create the SC for the asset with moonbeam foreign assets pallet
			pallet_moonbeam_foreign_assets::Pallet::<T>::create_foreign_asset(
				origin,
				asset_id,
				xcm_location,
				decimals,
				symbol,
				name,
			)?;

			*status = ForeignAssetMigrationStatus::Migrating(ForeignAssetMigreationInfo {
				asset_id,
				remaining_balances: asset.accounts,
				remaining_approvals: asset.approvals,
			});
			Ok(())
		})
	}

	pub(super) fn do_migrate_foreign_asset_balances(limit: u64) -> DispatchResult {
		use pallet_assets::ExistenceReason::*;

		ensure!(limit != 0, Error::<T>::LimitCannotBeZero);

		ForeignAssetMigrationStatusValue::<T>::try_mutate(|status| -> DispatchResult {
			let info = match status {
				ForeignAssetMigrationStatus::Migrating(info) => info,
				_ => return Err(Error::<T>::NoMigrationInProgress.into()),
			};

			pallet_assets::Account::<T>::drain_prefix(info.asset_id)
				.take(limit as usize)
				.try_for_each(|(who, mut asset)| {
					// Unreserve the deposit
					if let Some((depositor, deposit)) = asset.reason.take_deposit_from() {
						<T as pallet_assets::Config>::Currency::unreserve(&depositor, deposit);
					} else if let Some(deposit) = asset.reason.take_deposit() {
						<T as pallet_assets::Config>::Currency::unreserve(&who, deposit);
					}

					match asset.reason {
						Consumer => frame_system::Pallet::<T>::dec_consumers(&who),
						Sufficient => {
							frame_system::Pallet::<T>::dec_sufficients(&who);
						}
						_ => {}
					};

					pallet_moonbeam_foreign_assets::Pallet::<T>::mint_into(
						info.asset_id,
						who.clone(),
						asset.balance.into(),
					)
					.map_err(|_| Error::<T>::MintFailed)?;

					info.remaining_balances = info.remaining_balances.saturating_sub(1);
					Ok::<(), Error<T>>(())
				})?;

			Ok(())
		})
	}

	pub(super) fn do_migrate_foreign_asset_approvals(limit: u64) -> DispatchResult {
		ensure!(limit != 0, Error::<T>::LimitCannotBeZero);

		ForeignAssetMigrationStatusValue::<T>::try_mutate(|status| -> DispatchResult {
			let info = match status {
				ForeignAssetMigrationStatus::Migrating(info) => info,
				_ => return Err(Error::<T>::NoMigrationInProgress.into()),
			};

			pallet_assets::Approvals::<T>::drain_prefix((info.asset_id,))
				.take(limit as usize)
				.try_for_each(|((owner, beneficiary), approval)| {
					<T as pallet_assets::Config>::Currency::unreserve(&owner, approval.deposit);

					pallet_moonbeam_foreign_assets::Pallet::<T>::approve(
						info.asset_id,
						owner.clone(),
						beneficiary,
						approval.amount.into(),
					)
					.map_err(|_| Error::<T>::ApprovalFailed)?;

					info.remaining_approvals = info.remaining_approvals.saturating_sub(1);
					Ok::<(), Error<T>>(())
				})?;

			Ok(())
		})
	}

	/// Finish Migration
	pub(super) fn do_finish_foreign_asset_migration() -> DispatchResult {
		ForeignAssetMigrationStatusValue::<T>::try_mutate(|status| -> DispatchResult {
			let migration_info = match status {
				ForeignAssetMigrationStatus::Migrating(info) => info,
				_ => return Err(Error::<T>::NoMigrationInProgress.into()),
			};

			ensure!(
				migration_info.remaining_balances == 0 && migration_info.remaining_approvals == 0,
				Error::<T>::MigrationNotFinished
			);

			pallet_assets::Asset::<T>::try_mutate_exists(
				migration_info.asset_id,
				|maybe_details| {
					let details = maybe_details.take().ok_or(Error::<T>::AssetNotFound)?;

					let metadata = pallet_assets::Metadata::<T>::take(migration_info.asset_id);
					<T as pallet_assets::Config>::Currency::unreserve(
						&details.owner,
						details.deposit.saturating_add(metadata.deposit),
					);

					Ok::<(), Error<T>>(())
				},
			)?;

			*status = ForeignAssetMigrationStatus::Idle;
			Ok(())
		})
	}
}
