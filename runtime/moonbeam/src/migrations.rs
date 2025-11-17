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
use moonbeam_core_primitives::AssetId;
use pallet_xcm_transactor::migrations::UpdateRelayChainIndices;
use pallet_xcm_transactor::relay_indices;
use sp_core::parameter_types;

parameter_types! {
	pub RelayAssetId: AssetId = AssetType::Xcm(xcm::v3::Location::parent()).into();
	pub RelayChainIndices: relay_indices::RelayChainIndices = relay_indices::RelayChainIndices {
		// Pallet indices
		staking: 89,
		utility: 40,
		hrmp: 60,
		// Staking indices
		bond: 0,
		bond_extra: 1,
		unbond: 2,
		withdraw_unbonded: 3,
		validate: 4,
		nominate: 5,
		chill: 6,
		set_payee: 7,
		set_controller: 8,
		rebond: 19,
		// Utility indices
		as_derivative: 1,
		// Hrmp indices
		init_open_channel: 0,
		accept_open_channel: 1,
		close_channel: 2,
		cancel_open_request: 6,
	};
}

type MoonbeamMigrations = (UpdateRelayChainIndices<crate::Runtime, RelayChainIndices>,);

/// List of single block migrations to be executed by frame executive.
pub type SingleBlockMigrations<Runtime> = (
	// Common migrations applied on all Moonbeam runtime
	moonbeam_runtime_common::migrations::SingleBlockMigrations<Runtime>,
	// Moonbeam specific migrations
	MoonbeamMigrations,
);

/// List of multi block migrations to be executed by the pallet_migrations.
#[cfg(not(feature = "runtime-benchmarks"))]
pub type MultiBlockMigrationList<Runtime> = (
	// Common multiblock migrations applied on all Moonbeam runtime
	moonbeam_runtime_common::migrations::MultiBlockMigrations<Runtime>,
	// ... Moonbeam specific multiblock migrations
);
