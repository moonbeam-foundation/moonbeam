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
use ::pallet_bridge_messages::WeightInfoExt as MessagesWeightInfoExt;
use ::pallet_bridge_parachains::WeightInfoExt as ParachainsWeightInfoExt;

impl GrandpaWeightInfoExt for super::pallet_bridge_grandpa::WeightInfo<Runtime> {
	fn submit_finality_proof_overhead_from_runtime() -> Weight {
		// Update this value if pallet_bridge_relayers is added to the runtime.
		Weight::zero()
	}
}

impl ParachainsWeightInfoExt for super::pallet_bridge_parachains::WeightInfo<Runtime> {
	fn expected_extra_storage_proof_size() -> u32 {
		::pallet_bridge_parachains::weights_ext::EXTRA_STORAGE_PROOF_SIZE
	}

	fn submit_parachain_heads_overhead_from_runtime() -> Weight {
		// Update this value if pallet_bridge_relayers is added to the runtime.
		Weight::zero()
	}
}

impl MessagesWeightInfoExt for super::pallet_bridge_messages::WeightInfo<Runtime> {
	fn expected_extra_storage_proof_size() -> u32 {
		::pallet_bridge_messages::EXTRA_STORAGE_PROOF_SIZE
	}

	fn receive_messages_proof_overhead_from_runtime() -> Weight {
		// Update this value if pallet_bridge_relayers is added to the runtime.
		Weight::zero()
	}

	fn receive_messages_delivery_proof_overhead_from_runtime() -> Weight {
		// Update this value if pallet_bridge_relayers is added to the runtime.
		Weight::zero()
	}
}
