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
pub mod pallet_asset_manager;
pub mod pallet_assets;
pub mod pallet_author_inherent;
pub mod pallet_author_mapping;
pub mod pallet_author_slot_filter;
pub mod pallet_balances;
pub mod pallet_collective_open_tech_committee_collective_benchmark;
pub mod pallet_collective_treasury_council_collective_benchmark;
pub mod pallet_conviction_voting;
pub mod pallet_crowdloan_rewards;
pub mod pallet_evm;
pub mod pallet_identity;
pub mod pallet_message_queue;
pub mod pallet_moonbeam_foreign_assets;
pub mod pallet_moonbeam_lazy_migrations;
pub mod pallet_moonbeam_orbiters;
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
pub mod pallet_timestamp;
pub mod pallet_treasury;
pub mod pallet_utility;
pub mod pallet_whitelist;
pub mod pallet_xcm;
pub mod pallet_xcm_transactor;
pub mod pallet_xcm_weight_trader;
