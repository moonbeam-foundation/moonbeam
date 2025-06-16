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

//! Moonbeam common weights.

pub mod cumulus_pallet_parachain_system;
pub mod cumulus_pallet_xcmp_queue;
pub mod db;
pub mod frame_system;
pub mod frame_system_extensions;
pub mod pallet_asset_manager;
pub mod pallet_assets;
pub mod pallet_author_inherent;
pub mod pallet_author_mapping;
pub mod pallet_author_slot_filter;
pub mod pallet_balances;
pub mod pallet_collective_open_tech_committee_collective;
pub mod pallet_collective_treasury_council_collective;
pub mod pallet_conviction_voting;
pub mod pallet_crowdloan_rewards;
pub mod pallet_evm;
pub mod pallet_identity;
pub mod pallet_message_queue;
pub mod pallet_moonbeam_foreign_assets;
pub mod pallet_moonbeam_lazy_migrations;
pub mod pallet_moonbeam_orbiters;
pub mod pallet_multiblock_migrations;
pub mod pallet_multisig;
pub mod pallet_parachain_staking;
pub mod pallet_parameters;
pub mod pallet_precompile_benchmarks;
pub mod pallet_preimage;
pub mod pallet_proxy;
pub mod pallet_randomness;
pub mod pallet_referenda;
pub mod pallet_relay_storage_roots;
pub mod pallet_scheduler;
pub mod pallet_sudo;
pub mod pallet_timestamp;
pub mod pallet_transaction_payment;
pub mod pallet_treasury;
pub mod pallet_utility;
pub mod pallet_whitelist;
pub mod pallet_xcm;
pub mod pallet_xcm_transactor;
pub mod pallet_xcm_weight_trader;

#[cfg(any(feature = "bridge-stagenet", feature = "bridge-betanet"))]
mod bridge_weights;
#[cfg(any(feature = "bridge-stagenet", feature = "bridge-betanet"))]
pub mod pallet_bridge_grandpa;
#[cfg(any(feature = "bridge-stagenet", feature = "bridge-betanet"))]
pub mod pallet_bridge_messages;
#[cfg(any(feature = "bridge-stagenet", feature = "bridge-betanet"))]
pub mod pallet_bridge_parachains;
#[cfg(any(feature = "bridge-stagenet", feature = "bridge-betanet"))]
pub mod pallet_xcm_bridge;

#[cfg(any(feature = "bridge-stagenet", feature = "bridge-betanet"))]
impl ::pallet_bridge_messages::WeightInfoExt for pallet_bridge_messages::WeightInfo<Runtime> {
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
