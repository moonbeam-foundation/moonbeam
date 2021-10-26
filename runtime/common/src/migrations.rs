// Copyright 2019-2020 PureStake Inc.
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

//! # Migrations

use frame_support::{
	dispatch::GetStorageVersion,
	pallet_prelude::Get,
	traits::{OnRuntimeUpgrade, PalletInfoAccess},
	weights::Weight,
};
use pallet_author_mapping::{migrations::TwoXToBlake, Config as AuthorMappingConfig};
use pallet_migrations::Migration;
use sp_std::{marker::PhantomData, prelude::*};

/// This module acts as a registry where each migration is defined. Each migration should implement
/// the "Migration" trait declared in the pallet-migrations crate.

/// A moonbeam migration wrapping the similarly named migration in pallet-author-mapping
pub struct AuthorMappingTwoXToBlake<T>(PhantomData<T>);
impl<T: AuthorMappingConfig> Migration for AuthorMappingTwoXToBlake<T> {
	fn friendly_name(&self) -> &str {
		"MM_Author_Mapping_TwoXToBlake"
	}

	fn migrate(&self, _available_weight: Weight) -> Weight {
		TwoXToBlake::<T>::on_runtime_upgrade()
	}

	/// Run a standard pre-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade(&self) -> Result<(), &'static str> {
		TwoXToBlake::<T>::pre_upgrade()
	}

	/// Run a standard post-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn post_upgrade(&self) -> Result<(), &'static str> {
		TwoXToBlake::<T>::post_upgrade()
	}
}

const COUNCIL_OLD_PREFIX: &str = "Instance1Collective";
const TECH_OLD_PREFIX: &str = "Instance2Collective";

pub struct MigrateCollectivePallets<Runtime, Council, Tech>(PhantomData<(Runtime, Council, Tech)>);
impl<Runtime, Council, Tech> Migration for MigrateCollectivePallets<Runtime, Council, Tech>
where
	Runtime: frame_system::Config,
	Council: GetStorageVersion + PalletInfoAccess,
	Tech: GetStorageVersion + PalletInfoAccess,
{
	fn friendly_name(&self) -> &str {
		"MM_Collective_Pallets_v0.9.11_Prefixes"
	}

	fn migrate(&self, _available_weight: Weight) -> Weight {
		pallet_collective::migrations::v4::migrate::<Runtime, Council, _>(COUNCIL_OLD_PREFIX)
			+ pallet_collective::migrations::v4::migrate::<Runtime, Tech, _>(TECH_OLD_PREFIX)
	}

	/// Run a standard pre-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade(&self) -> Result<(), &'static str> {
		pallet_collective::migrations::v4::pre_migrate::<Council, _>(COUNCIL_OLD_PREFIX);
		pallet_collective::migrations::v4::pre_migrate::<Tech, _>(TECH_OLD_PREFIX);
		Ok(())
	}

	/// Run a standard post-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn post_upgrade(&self) -> Result<(), &'static str> {
		pallet_collective::migrations::v4::post_migrate::<Council, _>(COUNCIL_OLD_PREFIX);
		pallet_collective::migrations::v4::post_migrate::<Tech, _>(TECH_OLD_PREFIX);
		Ok(())
	}
}

pub struct CommonMigrations<Runtime, Council, Tech>(PhantomData<(Runtime, Council, Tech)>);

impl<Runtime, Council, Tech> Get<Vec<Box<dyn Migration>>>
	for CommonMigrations<Runtime, Council, Tech>
where
	Runtime: pallet_author_mapping::Config,
	Council: GetStorageVersion + PalletInfoAccess + 'static,
	Tech: GetStorageVersion + PalletInfoAccess + 'static,
{
	fn get() -> Vec<Box<dyn Migration>> {
		// let migration_author_mapping_twox_to_blake = AuthorMappingTwoXToBlake::<Runtime> {
		// 	0: Default::default(),
		// };

		let migration_collectives =
			MigrateCollectivePallets::<Runtime, Council, Tech>(Default::default());

		// TODO: this is a lot of allocation to do upon every get() call. this *should* be avoided
		// except when pallet_migrations undergoes a runtime upgrade -- but TODO: review

		vec![
			// completed in runtime 800
			// Box::new(migration_author_mapping_twox_to_blake),
			Box::new(migration_collectives),
		]
	}
}
