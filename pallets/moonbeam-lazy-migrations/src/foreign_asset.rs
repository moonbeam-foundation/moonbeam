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
use sp_core::{H160, U256};

#[derive(Debug, Encode, Decode, scale_info::TypeInfo, PartialEq, MaxEncodedLen)]
pub enum ForeignAssetMigrationStatus {
	/// No migration in progress
	Idle,
	/// Migrating a foreign asset in progress
	Migrating(ForeignAssetMigrationInfo),
}

impl Default for ForeignAssetMigrationStatus {
	fn default() -> Self {
		ForeignAssetMigrationStatus::Idle
	}
}

#[derive(Debug, Encode, Decode, scale_info::TypeInfo, PartialEq, MaxEncodedLen)]
pub(super) struct ForeignAssetMigrationInfo {
	pub(super) asset_id: u128,
	pub(super) remaining_balances: u32,
	pub(super) remaining_approvals: u32,
}

impl<T: Config> Pallet<T>
where
	<T as pallet_assets::Config>::Balance: Into<U256>,
	<T as pallet_asset_manager::Config>::ForeignAssetType: Into<Option<Location>>,
	<T as frame_system::Config>::AccountId: Into<H160> + From<H160>,
{
	/// Start a foreign asset migration by freezing the asset and creating the SC with the moonbeam
	/// foreign assets pallet.
	pub(super) fn do_start_foreign_asset_migration(asset_id: u128) -> DispatchResult {
		ForeignAssetMigrationStatusValue::<T>::try_mutate(|status| -> DispatchResult {
			ensure!(
				*status == ForeignAssetMigrationStatus::Idle,
				Error::<T>::MigrationNotFinished
			);

			// ensure asset_id is in the approved list
			ensure!(
				ApprovedForeignAssets::<T>::contains_key(asset_id),
				Error::<T>::AssetNotFound
			);

			// Freeze the asset
			pallet_assets::Asset::<T>::try_mutate_exists(asset_id, |maybe_details| {
				let details = maybe_details.as_mut().ok_or(Error::<T>::AssetNotFound)?;

				details.status = pallet_assets::AssetStatus::Frozen;

				let decimals = pallet_assets::Pallet::<T>::decimals(asset_id);
				let symbol = pallet_assets::Pallet::<T>::symbol(asset_id)
					.try_into()
					.map_err(|_| Error::<T>::SymbolTooLong)?;
				let name = <pallet_assets::Pallet<T> as Inspect<_>>::name(asset_id)
					.try_into()
					.map_err(|_| Error::<T>::NameTooLong)?;
				let asset_type = pallet_asset_manager::AssetIdType::<T>::take(asset_id)
					.ok_or(Error::<T>::AssetTypeNotFound)?;
				let xcm_location: Location =
					asset_type.into().ok_or(Error::<T>::LocationNotFound)?;

				// Remove the precompile for the old foreign asset.
				// Cleaning the precompile is done by removing the code and metadata
				let contract_addr =
					pallet_moonbeam_foreign_assets::Pallet::<T>::contract_address_from_asset_id(
						asset_id,
					);
				pallet_evm::AccountCodes::<T>::remove(contract_addr);
				pallet_evm::AccountCodesMetadata::<T>::remove(contract_addr);

				// Create the SC for the asset with moonbeam foreign assets pallet
				pallet_moonbeam_foreign_assets::Pallet::<T>::register_foreign_asset(
					asset_id,
					xcm_location,
					decimals,
					symbol,
					name,
				)?;

				*status = ForeignAssetMigrationStatus::Migrating(ForeignAssetMigrationInfo {
					asset_id,
					remaining_balances: details.accounts,
					remaining_approvals: details.approvals,
				});

				Ok(())
			})
		})
	}

	pub(super) fn do_migrate_foreign_asset_balances(limit: u32) -> DispatchResult {
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

					let zero_address = T::AccountId::from(H160::zero());
					if who.clone() != zero_address {
						MIGRATING_FOREIGN_ASSETS::using_once(&mut true, || {
							pallet_moonbeam_foreign_assets::Pallet::<T>::mint_into(
								info.asset_id,
								who.clone(),
								asset.balance.into(),
							)
						})
						.map_err(|err| {
							log::debug!("Error: {err:?}");
							Error::<T>::MintFailed
						})?;
					}

					info.remaining_balances = info.remaining_balances.saturating_sub(1);
					Ok::<(), Error<T>>(())
				})?;

			Ok(())
		})
	}

	pub(super) fn call_without_metadata() {}

	pub(super) fn do_migrate_foreign_asset_approvals(limit: u32) -> DispatchResult {
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

					MIGRATING_FOREIGN_ASSETS::using_once(&mut true, || {
						let address: H160 = owner.clone().into();

						// Temporarily remove metadata
						let meta = pallet_evm::AccountCodesMetadata::<T>::take(address.clone());

						let result = pallet_moonbeam_foreign_assets::Pallet::<T>::approve(
							info.asset_id,
							owner.clone(),
							beneficiary,
							approval.amount.into(),
						);

						// Re-add metadata
						if let Some(metadata) = meta {
							pallet_evm::AccountCodesMetadata::<T>::insert(address, metadata);
						}

						result
					})
					.map_err(|err| {
						log::debug!("Error: {err:?}");
						Error::<T>::ApprovalFailed
					})?;

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

			ApprovedForeignAssets::<T>::remove(migration_info.asset_id);
			*status = ForeignAssetMigrationStatus::Idle;
			Ok(())
		})
	}
}
