// Copyright 2019-2025 Moonbeam Foundation.
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

//! Relay chain genesis for xcm-emulator tests.
//!
//! Uses the full `westend_runtime` so we get real DMP routing, HRMP, and
//! the `ParachainHost` runtime API the emulator requires.

pub use westend_runtime;

use parity_scale_codec::Encode;
use sp_core::storage::Storage;
use sp_runtime::{traits::AccountIdConversion, AccountId32, BuildStorage};

use crate::network::{ASSET_HUB_PARA_ID, MOONBEAM_PARA_ID, SIBLING_PARA_ID};

/// Build relay `Storage` with both parachains registered and funded.
pub fn relay_genesis() -> Storage {
	let asset_hub_sovereign: AccountId32 =
		polkadot_parachain::primitives::Id::from(ASSET_HUB_PARA_ID).into_account_truncating();
	let moonbase_sovereign: AccountId32 =
		polkadot_parachain::primitives::Id::from(MOONBEAM_PARA_ID).into_account_truncating();
	let sibling_sovereign: AccountId32 =
		polkadot_parachain::primitives::Id::from(SIBLING_PARA_ID).into_account_truncating();
	let endowment: u128 = 1_000_000_000_000_000; // 100 000 DOT

	let mut host_config = polkadot_runtime_parachains::configuration::HostConfiguration::default();
	host_config.max_downward_message_size = 1 << 20;
	host_config.max_upward_message_size = 1 << 16;
	host_config.max_upward_queue_count = 100;
	host_config.max_upward_message_num_per_candidate = 10;
	host_config.hrmp_max_message_num_per_candidate = 10;
	host_config.hrmp_channel_max_capacity = 8;
	host_config.hrmp_channel_max_total_size = 8 * 1024;
	host_config.hrmp_channel_max_message_size = 1024;
	host_config.hrmp_max_parachain_outbound_channels = 10;
	host_config.hrmp_max_parachain_inbound_channels = 10;

	let genesis_config = westend_runtime::RuntimeGenesisConfig {
		balances: westend_runtime::BalancesConfig {
			balances: vec![
				(AccountId32::new([1u8; 32]), endowment),
				(AccountId32::new([2u8; 32]), endowment),
				(asset_hub_sovereign, endowment),
				(moonbase_sovereign, endowment),
				(sibling_sovereign, endowment),
			],
			..Default::default()
		},
		configuration: westend_runtime::ConfigurationConfig {
			config: host_config,
		},
		..Default::default()
	};

	let mut storage = genesis_config
		.build_storage()
		.expect("Should build relay genesis storage");

	// Register both parachains so DMP and HRMP consider them valid.
	use frame_support::storage::generator::StorageMap;
	for para_id in [ASSET_HUB_PARA_ID, MOONBEAM_PARA_ID, SIBLING_PARA_ID] {
		let pid = polkadot_parachain::primitives::Id::from(para_id);
		let head_data = polkadot_parachain::primitives::HeadData(vec![0u8; 32]);
		let key = polkadot_runtime_parachains::paras::Heads::<westend_runtime::Runtime>::storage_map_final_key(pid);
		storage.top.insert(key, head_data.encode());

		// Also register the ParaLifecycle as Parachain so HRMP considers them valid.
		// ParaLifecycles is a StorageMap with Twox64Concat hasher on ParaId.
		// prefix = twox128("Paras") ++ twox128("ParaLifecycles")
		// key suffix = twox64(ParaId.encode()) ++ ParaId.encode()
		let prefix = frame_support::storage::storage_prefix(b"Paras", b"ParaLifecycles");
		let encoded_id = pid.encode();
		let mut full_key = prefix.to_vec();
		full_key.extend(&sp_io::hashing::twox_64(&encoded_id));
		full_key.extend(&encoded_id);
		// ParaLifecycle::Parachain is the third variant (index 2)
		// Onboarding=0, Parathread=1, Parachain=2
		storage.top.insert(full_key, 2u8.encode());
	}

	storage
}
