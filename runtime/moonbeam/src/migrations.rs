// Copyright 2025 Moonbeam Foundation.Inc.
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

//! # Moonbeam specific Migrations

use crate::xcm_config::AssetType;
use core::marker::PhantomData;
use frame_support::{storage::storage_prefix, traits::OnRuntimeUpgrade};
use moonbeam_core_primitives::AssetId;
#[cfg(feature = "try-runtime")]
use parity_scale_codec::{Decode, Encode};
use sp_core::{parameter_types, Get};

parameter_types! {
	pub RelayAssetId: AssetId = AssetType::Xcm(xcm::v3::Location::parent()).into();
}

pub struct PreserveXcmRelativePrices<Runtime>(PhantomData<Runtime>);

impl<Runtime> PreserveXcmRelativePrices<Runtime> {
	fn migration_key() -> [u8; 32] {
		storage_prefix(b"MoonbeamMigrations", b"PreserveXcmRelativePricesV1")
	}
}

impl<Runtime> OnRuntimeUpgrade for PreserveXcmRelativePrices<Runtime>
where
	Runtime: frame_system::Config + pallet_xcm_weight_trader::Config,
{
	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		if frame_support::storage::unhashed::get::<bool>(&Self::migration_key()).unwrap_or(false) {
			log::info!(
				target: "runtime::moonbeam",
				"PreserveXcmRelativePrices already ran; skipping",
			);
			return Runtime::DbWeight::get().reads(1);
		}

		let mut reads = 0u64;
		let mut writes = 0u64;

		for (location, (enabled, relative_price)) in
			pallet_xcm_weight_trader::SupportedAssets::<Runtime>::iter()
		{
			reads = reads.saturating_add(1);

			let new_relative_price = relative_price.saturating_mul(1_000);
			pallet_xcm_weight_trader::SupportedAssets::<Runtime>::insert(
				location,
				(enabled, new_relative_price),
			);
			writes = writes.saturating_add(1);
		}

		log::info!(
			target: "runtime::moonbeam",
			"PreserveXcmRelativePrices updated {writes} XCM weight trader relative prices",
		);

		frame_support::storage::unhashed::put(&Self::migration_key(), &true);

		reads = reads.saturating_add(1);
		writes = writes.saturating_add(1);

		Runtime::DbWeight::get().reads_writes(reads, writes)
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<sp_std::vec::Vec<u8>, sp_runtime::TryRuntimeError> {
		let before = pallet_xcm_weight_trader::SupportedAssets::<Runtime>::iter()
			.collect::<sp_std::vec::Vec<_>>();
		let already_ran =
			frame_support::storage::unhashed::get::<bool>(&Self::migration_key()).unwrap_or(false);

		log::info!(
			target: "runtime::moonbeam",
			"PreserveXcmRelativePrices will update {} XCM weight trader relative prices",
			before.len(),
		);

		Ok((already_ran, before).encode())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(state: sp_std::vec::Vec<u8>) -> Result<(), sp_runtime::TryRuntimeError> {
		let (already_ran, before): (bool, sp_std::vec::Vec<(xcm::v5::Location, (bool, u128))>) =
			Decode::decode(&mut state.as_ref())
				.map_err(|_| "PreserveXcmRelativePrices failed to decode pre-upgrade state")?;

		let marker =
			frame_support::storage::unhashed::get::<bool>(&Self::migration_key()).unwrap_or(false);

		if already_ran {
			if !marker {
				Err("PreserveXcmRelativePrices marker unexpectedly missing")?;
			}
			return Ok(());
		}

		if !marker {
			Err("PreserveXcmRelativePrices marker was not set")?;
		}

		for (location, (_, previous_relative_price)) in before {
			let (_, current_relative_price) =
				pallet_xcm_weight_trader::SupportedAssets::<Runtime>::get(&location)
					.ok_or("PreserveXcmRelativePrices missing asset after migration")?;
			let expected_relative_price = previous_relative_price.saturating_mul(1_000);

			if current_relative_price != expected_relative_price {
				log::error!(
					target: "runtime::moonbeam",
					"PreserveXcmRelativePrices failed for location {:?}: expected {}, got {}",
					location,
					expected_relative_price,
					current_relative_price,
				);
				Err("PreserveXcmRelativePrices relative price mismatch")?;
			}
		}

		Ok(())
	}
}

type MoonbeamMigrations<Runtime> = (PreserveXcmRelativePrices<Runtime>,);

/// List of single block migrations to be executed by frame executive.
pub type SingleBlockMigrations<Runtime> = (
	// Common migrations applied on all Moonbeam runtime
	moonbeam_runtime_common::migrations::SingleBlockMigrations<Runtime>,
	// Moonbeam specific migrations
	MoonbeamMigrations<Runtime>,
);

/// List of multi block migrations to be executed by the pallet_migrations.
#[cfg(not(feature = "runtime-benchmarks"))]
pub type MultiBlockMigrationList<Runtime> = (
	// Common multiblock migrations applied on all Moonbeam runtimes
	moonbeam_runtime_common::migrations::MultiBlockMigrations<Runtime>,
	// ... Moonbeam specific multiblock migrations
);

#[cfg(test)]
mod tests {
	use super::*;
	use crate::Runtime;

	#[test]
	fn preserve_xcm_relative_prices_scales_supported_assets() {
		sp_io::TestExternalities::default().execute_with(|| {
			let parent = xcm::v5::Location::parent();
			let sibling = xcm::v5::Location::new(1, [xcm::v5::Junction::Parachain(2000)]);

			pallet_xcm_weight_trader::SupportedAssets::<Runtime>::insert(
				parent.clone(),
				(true, 123),
			);
			pallet_xcm_weight_trader::SupportedAssets::<Runtime>::insert(
				sibling.clone(),
				(false, 0),
			);

			PreserveXcmRelativePrices::<Runtime>::on_runtime_upgrade();

			assert_eq!(
				pallet_xcm_weight_trader::SupportedAssets::<Runtime>::get(parent),
				Some((true, 123_000))
			);
			assert_eq!(
				pallet_xcm_weight_trader::SupportedAssets::<Runtime>::get(sibling),
				Some((false, 0))
			);

			PreserveXcmRelativePrices::<Runtime>::on_runtime_upgrade();

			assert_eq!(
				pallet_xcm_weight_trader::SupportedAssets::<Runtime>::get(
					xcm::v5::Location::parent()
				),
				Some((true, 123_000))
			);
		});
	}
}
