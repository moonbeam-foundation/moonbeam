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

use crate::*;

/// This module acts as a registry where each migration is defined. Each migration should implement
/// the "Migration" trait declared in this crate.

pub struct MM_001_AuthorMappingAddDeposit;
impl Migration for MM_001_AuthorMappingAddDeposit {
	/*
	fn friendly_name() -> &str {
		"AuthorMappingAddDeposit"
	}
	*/
	fn apply() -> Weight {
		/// reviewer note: this isn't meant to imply that migration code must live here. As noted
		/// elsewhere, I would expect migration code to live close to the pallet it affects.
		0u64.into()
	}
}

pub struct MM_002_StakingFixTotalBalance;
impl Migration for MM_002_StakingFixTotalBalance {
	/*
	fn friendly_name() -> &str {
		"StakingFixTotalBalance"
	}
	*/
	fn apply() -> Weight {
		0u64.into()
	}
}

pub struct MM_003_StakingTransitionBoundedSet; // TODO: better name
impl Migration for MM_003_StakingTransitionBoundedSet {
	/*
	fn friendly_name() -> &str {
		"StakingTransitionBoundedSet"
	}
	*/
	fn apply() -> Weight {
		0u64.into()
	}
}
