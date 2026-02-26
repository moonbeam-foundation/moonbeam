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

//! Network declaration for xcm-emulator.
//!
//! Wires a Westend relay chain, the real Moonbeam runtime (para 2004),
//! and a sibling Moonbeam instance (para 2005) into a single test network.

use crate::emulator_relay;

use frame_support::traits::OnInitialize;
use xcm_emulator::decl_test_networks;
use xcm_emulator::decl_test_parachains;
use xcm_emulator::decl_test_relay_chains;
use xcm_emulator::Parachain;
use xcm_emulator::TestExt;

pub const MOONBEAM_PARA_ID: u32 = 2004;
pub const SIBLING_PARA_ID: u32 = 2005;

// ---- Well-known test accounts (20-byte) ------------------------------------
pub const ALITH: [u8; 20] = [1u8; 20];
pub const BALTATHAR: [u8; 20] = [2u8; 20];
pub const CHARLETH: [u8; 20] = [3u8; 20];

// ---- Well-known relay accounts (32-byte) -----------------------------------
pub const RELAY_ALICE: sp_runtime::AccountId32 = sp_runtime::AccountId32::new([1u8; 32]);

// ---- DOT constants ---------------------------------------------------------
pub const ONE_DOT: u128 = 10_000_000_000; // 10 decimals

// ---------------------------------------------------------------------------
// Relay chain declaration (Westend runtime)
// ---------------------------------------------------------------------------
decl_test_relay_chains! {
	#[api_version(13)]
	pub struct WestendRelay {
		genesis = emulator_relay::relay_genesis(),
		on_init = (),
		runtime = westend_runtime,
		core = {
			SovereignAccountOf: westend_runtime::xcm_config::LocationConverter,
		},
		pallets = {
			XcmPallet: westend_runtime::XcmPallet,
			Balances: westend_runtime::Balances,
			Hrmp: westend_runtime::Hrmp,
			Utility: westend_runtime::Utility,
		}
	}
}

// ---------------------------------------------------------------------------
// Moonbeam parachain declaration (para 2004)
// ---------------------------------------------------------------------------
decl_test_parachains! {
	pub struct MoonbeamPara {
		genesis = moonbeam_genesis(MOONBEAM_PARA_ID),
		on_init = {
			crate::emulator_network::satisfy_moonbeam_inherents();
		},
		runtime = moonbeam_runtime,
		core = {
			XcmpMessageHandler: moonbeam_runtime::XcmpQueue,
			LocationToAccountId: moonbeam_runtime::xcm_config::LocationToAccountId,
			ParachainInfo: moonbeam_runtime::ParachainInfo,
			MessageOrigin: cumulus_primitives_core::AggregateMessageOrigin,
		},
		pallets = {
			PolkadotXcm: moonbeam_runtime::PolkadotXcm,
			Balances: moonbeam_runtime::Balances,
			EvmForeignAssets: moonbeam_runtime::EvmForeignAssets,
			XcmWeightTrader: moonbeam_runtime::XcmWeightTrader,
			XcmTransactor: moonbeam_runtime::XcmTransactor,
			Treasury: moonbeam_runtime::Treasury,
			EthereumXcm: moonbeam_runtime::EthereumXcm,
			Proxy: moonbeam_runtime::Proxy,
			EVM: moonbeam_runtime::EVM,
		}
	}
}

// ---------------------------------------------------------------------------
// Sibling parachain declaration (para 2005) — another Moonbeam instance
// ---------------------------------------------------------------------------
decl_test_parachains! {
	pub struct SiblingPara {
		genesis = moonbeam_genesis(SIBLING_PARA_ID),
		on_init = {
			crate::emulator_network::satisfy_moonbeam_inherents();
		},
		runtime = moonbeam_runtime,
		core = {
			XcmpMessageHandler: moonbeam_runtime::XcmpQueue,
			LocationToAccountId: moonbeam_runtime::xcm_config::LocationToAccountId,
			ParachainInfo: moonbeam_runtime::ParachainInfo,
			MessageOrigin: cumulus_primitives_core::AggregateMessageOrigin,
		},
		pallets = {
			PolkadotXcm: moonbeam_runtime::PolkadotXcm,
			Balances: moonbeam_runtime::Balances,
			EvmForeignAssets: moonbeam_runtime::EvmForeignAssets,
			XcmWeightTrader: moonbeam_runtime::XcmWeightTrader,
			XcmTransactor: moonbeam_runtime::XcmTransactor,
			Treasury: moonbeam_runtime::Treasury,
			EthereumXcm: moonbeam_runtime::EthereumXcm,
			Proxy: moonbeam_runtime::Proxy,
			EVM: moonbeam_runtime::EVM,
		}
	}
}

// ---------------------------------------------------------------------------
// Network declaration
// ---------------------------------------------------------------------------
decl_test_networks! {
	pub struct PolkadotMoonbeamNet {
		relay_chain = WestendRelay,
		parachains = vec![
			MoonbeamPara,
			SiblingPara,
		],
		bridge = ()
	}
}

// ===========================================================================
// Helpers
// ===========================================================================

/// Execute a closure on the Moonbeam parachain (para 2004), automatically
/// satisfying mandatory inherent checks.
pub fn moonbeam_execute_with<R>(f: impl FnOnce() -> R) -> R {
	MoonbeamPara::<PolkadotMoonbeamNet>::execute_with(|| {
		satisfy_moonbeam_inherents();
		f()
	})
}

/// Execute a closure on the Sibling parachain (para 2005), automatically
/// satisfying mandatory inherent checks.
pub fn sibling_execute_with<R>(f: impl FnOnce() -> R) -> R {
	SiblingPara::<PolkadotMoonbeamNet>::execute_with(|| {
		satisfy_moonbeam_inherents();
		f()
	})
}

/// Patch storage to satisfy Moonbeam's mandatory inherent checks.
/// Called automatically by [`moonbeam_execute_with`] / [`sibling_execute_with`].
pub(crate) fn satisfy_moonbeam_inherents() {
	pallet_author_inherent::Author::<moonbeam_runtime::Runtime>::put(
		moonbeam_runtime::AccountId::from([1u8; 20]),
	);
	pallet_author_inherent::InherentIncluded::<moonbeam_runtime::Runtime>::put(true);

	frame_support::storage::unhashed::put(
		&frame_support::storage::storage_prefix(b"Randomness", b"InherentIncluded"),
		&(),
	);
	frame_support::storage::unhashed::kill(&frame_support::storage::storage_prefix(
		b"Randomness",
		b"NotFirstBlock",
	));
}

/// Initialise network and clear `NotFirstBlock` on both parachains.
pub fn init_network() {
	// Trigger `Parachain::init()` on every chain by executing on relay.
	WestendRelay::<PolkadotMoonbeamNet>::execute_with(|| {});

	// Clear NotFirstBlock so VRF verification is skipped in subsequent blocks.
	MoonbeamPara::<PolkadotMoonbeamNet>::ext_wrapper(|| {
		frame_support::storage::unhashed::kill(&frame_support::storage::storage_prefix(
			b"Randomness",
			b"NotFirstBlock",
		));
	});
	SiblingPara::<PolkadotMoonbeamNet>::ext_wrapper(|| {
		frame_support::storage::unhashed::kill(&frame_support::storage::storage_prefix(
			b"Randomness",
			b"NotFirstBlock",
		));
	});
}

/// Register DOT as a foreign asset on a Moonbeam-runtime chain and configure
/// its price in the XCM weight trader. Call inside `moonbeam_execute_with` or
/// `sibling_execute_with`.
///
/// Returns the `asset_id` that was used for registration.
pub fn register_dot_asset(asset_id: u128) {
	let dot_location = xcm::latest::Location::parent();

	frame_support::assert_ok!(moonbeam_runtime::EvmForeignAssets::create_foreign_asset(
		moonbeam_runtime::RuntimeOrigin::root(),
		asset_id,
		dot_location.clone(),
		10,
		b"DOT".to_vec().try_into().unwrap(),
		b"Polkadot".to_vec().try_into().unwrap(),
	));

	// relative_price large enough so that 10 DOT covers XCM execution fees.
	frame_support::assert_ok!(moonbeam_runtime::XcmWeightTrader::add_asset(
		moonbeam_runtime::RuntimeOrigin::root(),
		dot_location,
		10_000_000_000_000_000_000_000_000_000u128, // 10^28
	));
}

/// Configure `pallet_xcm_transactor` relay indices for Westend.
/// Call inside `moonbeam_execute_with` or `sibling_execute_with`.
pub fn set_westend_relay_indices() {
	use pallet_xcm_transactor::relay_indices::RelayChainIndices;

	// Westend pallet indices (from construct_runtime):
	// Staking=6, Utility=16, Hrmp=51, Balances=4
	let indices = RelayChainIndices {
		staking: 6u8,
		utility: 16u8,
		hrmp: 51u8,
		// Call indices within staking pallet:
		bond: 0u8,
		bond_extra: 1u8,
		unbond: 2u8,
		withdraw_unbonded: 3u8,
		validate: 4u8,
		nominate: 5u8,
		chill: 6u8,
		set_payee: 7u8,
		set_controller: 8u8,
		rebond: 19u8,
		// Utility::as_derivative
		as_derivative: 1u8,
		// HRMP call indices:
		init_open_channel: 0u8,
		accept_open_channel: 1u8,
		close_channel: 2u8,
		cancel_open_request: 6u8,
	};

	pallet_xcm_transactor::RelayIndices::<moonbeam_runtime::Runtime>::put(indices);
}

/// Open HRMP channels between two parachains on the relay.
/// Must be called inside `WestendRelay::execute_with`.
pub fn open_hrmp_channels(sender: u32, recipient: u32) {
	use frame_support::assert_ok;

	assert_ok!(westend_runtime::Hrmp::force_open_hrmp_channel(
		westend_runtime::RuntimeOrigin::root(),
		sender.into(),
		recipient.into(),
		8,    // max_capacity
		1024, // max_message_size
	));
	assert_ok!(westend_runtime::Hrmp::force_open_hrmp_channel(
		westend_runtime::RuntimeOrigin::root(),
		recipient.into(),
		sender.into(),
		8,
		1024,
	));
	assert_ok!(westend_runtime::Hrmp::force_process_hrmp_open(
		westend_runtime::RuntimeOrigin::root(),
		2,
	));
}

// ---------------------------------------------------------------------------
// Moonbeam genesis helper
// ---------------------------------------------------------------------------

fn moonbeam_genesis(para_id: u32) -> sp_core::storage::Storage {
	use moonbeam_runtime::{currency::GLMR, AccountId, Runtime};
	use sp_runtime::BuildStorage;

	let mut t = frame_system::GenesisConfig::<Runtime>::default()
		.build_storage()
		.unwrap();

	parachain_info::GenesisConfig::<Runtime> {
		parachain_id: para_id.into(),
		_config: Default::default(),
	}
	.assimilate_storage(&mut t)
	.unwrap();

	pallet_balances::GenesisConfig::<Runtime> {
		balances: vec![
			(AccountId::from(ALITH), GLMR * 10_000),
			(AccountId::from(BALTATHAR), GLMR * 10_000),
			(AccountId::from(CHARLETH), GLMR * 10_000),
		],
		dev_accounts: None,
	}
	.assimilate_storage(&mut t)
	.unwrap();

	pallet_xcm::GenesisConfig::<Runtime> {
		safe_xcm_version: Some(xcm::latest::VERSION),
		..Default::default()
	}
	.assimilate_storage(&mut t)
	.unwrap();

	t
}
