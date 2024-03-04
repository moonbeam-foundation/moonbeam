// Copyright 2024 Moonbeam Foundation Inc.
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

//! # Moonbase specific Migrations

use crate::Runtime;
use frame_support::{traits::OnRuntimeUpgrade, weights::Weight};
use pallet_migrations::{GetMigrations, Migration};
use pallet_parachain_staking::migrations::MultiplyRoundLenBy2;
use sp_std::{prelude::*, vec};

pub struct MoonbaseMigrations;

impl GetMigrations for MoonbaseMigrations {
	fn get_migrations() -> Vec<Box<dyn Migration>> {
		vec![Box::new(PalletStakingMultiplyRoundLenBy2)]
	}
}

// This migration should only be applied to runtimes with async backing enabled
pub struct PalletStakingMultiplyRoundLenBy2;
impl Migration for PalletStakingMultiplyRoundLenBy2 {
	fn friendly_name(&self) -> &str {
		"MM_MultiplyRoundLenBy2"
	}

	fn migrate(&self, _available_weight: Weight) -> Weight {
		MultiplyRoundLenBy2::<Runtime>::on_runtime_upgrade()
	}
}
