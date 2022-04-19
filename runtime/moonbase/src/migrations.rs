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

//! Migrations specific to moonbase runtime

use super::{Balance, Runtime};
use frame_support::pallet_prelude::*;
use frame_support::traits::OnRuntimeUpgrade;
use sp_std::boxed::Box;
use sp_std::vec::Vec;

frame_support::parameter_types! {
		pub const MinOrbiterDeposit: Balance = 40_000 * super::currency::UNIT;
}

pub struct InitMinOrbiterDeposit;
impl pallet_migrations::Migration for InitMinOrbiterDeposit {
	fn friendly_name(&self) -> &str {
		"MM_MoonbeamOrbiters_InitMinOrbiterDeposit"
	}

	fn migrate(&self, _available_weight: Weight) -> Weight {
		pallet_moonbeam_orbiters::migrations::InitMinOrbiterDeposit::<
			Runtime,
			Balance,
			MinOrbiterDeposit,
		>::on_runtime_upgrade()
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade(&self) -> Result<(), &'static str> {
		pallet_moonbeam_orbiters::migrations::InitMinOrbiterDeposit::<
			Runtime,
			Balance,
			MinOrbiterDeposit,
		>::pre_upgrade()
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(&self) -> Result<(), &'static str> {
		pallet_moonbeam_orbiters::migrations::InitMinOrbiterDeposit::<
			Runtime,
			Balance,
			MinOrbiterDeposit,
		>::post_upgrade()
	}
}

pub struct MoonbaseMigrations;
impl pallet_migrations::GetMigrations for MoonbaseMigrations {
	fn get_migrations() -> Vec<Box<dyn pallet_migrations::Migration>> {
		sp_std::vec![Box::new(InitMinOrbiterDeposit)]
	}
}
