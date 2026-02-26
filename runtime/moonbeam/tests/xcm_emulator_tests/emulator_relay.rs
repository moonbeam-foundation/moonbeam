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

//! Relay chain setup using `westend_runtime` for xcm-emulator tests.
//!
//! The emulator's `decl_test_relay_chains!` macro requires a relay runtime
//! that implements the `ParachainHost` runtime API (specifically
//! `dmq_contents`). A minimal mock relay cannot satisfy this, so we use the
//! full Westend runtime as the relay chain.
//!
//! Genesis is kept minimal: a few funded accounts and Moonbeam's sovereign
//! account. No validators/sessions/staking are configured because the
//! emulator does not need active consensus — it drives blocks manually.

pub use westend_runtime;

use sp_core::storage::Storage;
use sp_runtime::{traits::AccountIdConversion, AccountId32, BuildStorage};

/// Build a minimal relay `Storage` with funded accounts.
///
/// **Note**: Because we skip the full validator/session/staking genesis, the
/// relay is only usable for XCM routing — not for consensus or parachain
/// validation. This is fine for emulator tests.
pub fn relay_genesis() -> Storage {
	let moonbeam_sovereign: AccountId32 =
		polkadot_parachain::primitives::Id::from(crate::emulator_network::MOONBEAM_PARA_ID)
			.into_account_truncating();

	let endowment: u128 = 1_000_000_000_000_000; // 100_000 DOT

	// Build a host configuration with generous DMP limits so messages can
	// be routed to parachains.
	let mut host_config = polkadot_runtime_parachains::configuration::HostConfiguration::default();
	host_config.max_downward_message_size = 1 << 20; // 1 MiB
	host_config.max_upward_message_size = 1 << 16;
	host_config.max_upward_queue_count = 100;
	host_config.max_upward_message_num_per_candidate = 10;
	host_config.hrmp_max_message_num_per_candidate = 10;

	let genesis_config = westend_runtime::RuntimeGenesisConfig {
		balances: westend_runtime::BalancesConfig {
			balances: vec![
				(AccountId32::new([1u8; 32]), endowment),
				(AccountId32::new([2u8; 32]), endowment),
				(moonbeam_sovereign, endowment),
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
		.expect("Failed to build relay genesis storage");

	// Register Moonbeam in `paras::Heads` so the DMP router considers it
	// a valid destination (it checks `Heads::contains_key(para)`).
	use frame_support::storage::generator::StorageMap;
	let para_id =
		polkadot_parachain::primitives::Id::from(crate::emulator_network::MOONBEAM_PARA_ID);
	let head_data = polkadot_parachain::primitives::HeadData(vec![0u8; 32]);
	let key = polkadot_runtime_parachains::paras::Heads::<westend_runtime::Runtime>::storage_map_final_key(para_id);
	storage.top.insert(key, head_data.encode());

	storage
}

use parity_scale_codec::Encode;
