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

//! # Moonriver specific Migrations

type MoonriverMigrations = ();

/// List of single block migrations to be executed by frame executive.
pub type SingleBlockMigrations<Runtime> = (
	// Common migrations applied on all Moonbeam runtime
	moonbeam_runtime_common::migrations::SingleBlockMigrations<Runtime>,
	// Moonriver specific migrations
	MoonriverMigrations,
);

/// List of multi block migrations to be executed by the pallet_migrations.
#[cfg(not(feature = "runtime-benchmarks"))]
pub type MultiBlockMigrationList<Runtime> = (
	// Common multiblock migrations applied on all Moonbeam runtime
	moonbeam_runtime_common::migrations::MultiBlockMigrations<Runtime>,
	// ... Moonriver specific multiblock migrations
);
