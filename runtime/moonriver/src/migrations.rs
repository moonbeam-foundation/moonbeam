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

//! # Moonriver specific Migrations

use pallet_migrations::{GetMigrations, Migration};
use sp_std::{prelude::*, vec};

pub struct MoonriverMigrations;

impl GetMigrations for MoonriverMigrations {
	fn get_migrations() -> Vec<Box<dyn Migration>> {
		vec![
			// Runtime 3000
			// Box::new(PalletStakingMultiplyRoundLenBy2)
		]
	}
}
