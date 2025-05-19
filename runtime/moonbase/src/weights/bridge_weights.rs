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

use crate::{Runtime, Weight};
use ::pallet_bridge_grandpa::WeightInfoExt as GrandpaWeightInfoExt;
use ::pallet_bridge_parachains::WeightInfoExt as ParachainsWeightInfoExt;

impl GrandpaWeightInfoExt for super::pallet_bridge_grandpa::WeightInfo<Runtime> {
	fn submit_finality_proof_overhead_from_runtime() -> Weight {
		// our signed extension:
		// 1) checks whether relayer registration is active from validate/pre_dispatch;
		// 2) may slash and deregister relayer from post_dispatch
		// (2) includes (1), so (2) is the worst case
		Weight::zero() // TODO(Rodrigo): Confirm later
	}
}

impl ParachainsWeightInfoExt for super::pallet_bridge_parachains::WeightInfo<Runtime> {
	fn submit_parachain_heads_overhead_from_runtime() -> Weight {
		Weight::zero() // TODO(Rodrigo): Confirm later
	}

	fn expected_extra_storage_proof_size() -> u32 {
		0 // TODO(Rodrigo): Confirm later
	}
}
