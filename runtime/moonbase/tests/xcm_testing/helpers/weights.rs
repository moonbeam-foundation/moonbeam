// Copyright 2019-2025 PureStake Inc.
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

//! Weight calculation helpers for XCM tests

use sp_weights::Weight;
use xcm::latest::prelude::WeightLimit;
use xcm_primitives::DEFAULT_PROOF_SIZE;

// Weight limit helpers for different test scenarios

pub fn standard_transfer_weight() -> WeightLimit {
	WeightLimit::Limited(Weight::from_parts(80u64, DEFAULT_PROOF_SIZE))
}

pub fn standard_heavy_weight() -> WeightLimit {
	WeightLimit::Limited(Weight::from_parts(800000u64, DEFAULT_PROOF_SIZE))
}

pub fn medium_transfer_weight() -> WeightLimit {
	WeightLimit::Limited(Weight::from_parts(40000u64, DEFAULT_PROOF_SIZE))
}
