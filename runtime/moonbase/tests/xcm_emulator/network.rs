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
//! Wires a Westend relay chain, the real Moonbase runtime (para 2004),
//! and a sibling Moonbase instance (para 2005) into a single test network.

use crate::relay;

use frame_support::traits::OnInitialize;
use nimbus_primitives::{NimbusId, NIMBUS_ENGINE_ID};
use parity_scale_codec::Encode;
use sp_runtime::{Digest, DigestItem};
use xcm_emulator::decl_test_networks;
use xcm_emulator::decl_test_parachains;
use xcm_emulator::decl_test_relay_chains;
use xcm_emulator::BlockProducer;
use xcm_emulator::TestExt;

/// `BlockProducer` for Moonbase's Nimbus-based consensus.
///
/// xcm-emulator defaults to an Aura-backed producer; Moonbase does not use
/// pallet-aura, so we plug in a Nimbus pre-runtime digest with a 6s block
/// interval (matching `moonbase_runtime::MILLISECS_PER_BLOCK`).
pub struct MoonbaseBlockProducer;

impl BlockProducer for MoonbaseBlockProducer {
	fn slot_duration() -> u64 {
		moonbase_runtime::MILLISECS_PER_BLOCK
	}

	fn pre_runtime_digest(_relay_block_number: u32) -> Digest {
		let author = NimbusId::from(sp_core::sr25519::Public::from_raw([1u8; 32]));
		Digest {
			logs: vec![DigestItem::PreRuntime(NIMBUS_ENGINE_ID, author.encode())],
		}
	}
}

pub const ASSET_HUB_PARA_ID: u32 = 1000;
pub const MOONBASE_PARA_ID: u32 = 2004;
pub const SIBLING_PARA_ID: u32 = 2005;

// ---- Well-known test accounts (20-byte) ------------------------------------
pub const ALITH: [u8; 20] = [1u8; 20];
pub const BALTATHAR: [u8; 20] = [2u8; 20];
// ---- Well-known relay accounts (32-byte) -----------------------------------
pub const RELAY_ALICE: sp_runtime::AccountId32 = sp_runtime::AccountId32::new([1u8; 32]);

// ---- Asset ID constants ----------------------------------------------------
pub const DOT_ASSET_ID: u128 = 1;
pub const UNIT_ASSET_ID: u128 = 2;

// ---- DOT constants ---------------------------------------------------------
pub const ONE_DOT: u128 = 10_000_000_000; // 10 decimals

// ---------------------------------------------------------------------------
// Relay chain declaration (Westend runtime)
// ---------------------------------------------------------------------------
decl_test_relay_chains! {
	#[api_version(15)]
	pub struct WestendRelay {
		genesis = relay::relay_genesis(),
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
// Moonbase parachain declaration (para 2004)
// ---------------------------------------------------------------------------
decl_test_parachains! {
	pub struct MoonbasePara {
		genesis = moonbase_genesis(MOONBASE_PARA_ID),
		on_init = {
			crate::network::satisfy_moonbase_inherents();
		},
		runtime = moonbase_runtime,
		core = {
			XcmpMessageHandler: moonbase_runtime::XcmpQueue,
			LocationToAccountId: moonbase_runtime::xcm_config::LocationToAccountId,
			ParachainInfo: moonbase_runtime::ParachainInfo,
			MessageOrigin: cumulus_primitives_core::AggregateMessageOrigin,
			BlockProducer: crate::network::MoonbaseBlockProducer,
		},
		pallets = {
			PolkadotXcm: moonbase_runtime::PolkadotXcm,
			Balances: moonbase_runtime::Balances,
			EvmForeignAssets: moonbase_runtime::EvmForeignAssets,
			XcmWeightTrader: moonbase_runtime::XcmWeightTrader,
			XcmTransactor: moonbase_runtime::XcmTransactor,
			Treasury: moonbase_runtime::Treasury,
			EthereumXcm: moonbase_runtime::EthereumXcm,
			Proxy: moonbase_runtime::Proxy,
			EVM: moonbase_runtime::EVM,
		}
	}
}

// ---------------------------------------------------------------------------
// Sibling parachain declaration (para 2005) — another Moonbase instance
// ---------------------------------------------------------------------------
decl_test_parachains! {
	pub struct SiblingPara {
		genesis = moonbase_genesis(SIBLING_PARA_ID),
		on_init = {
			crate::network::satisfy_moonbase_inherents();
		},
		runtime = moonbase_runtime,
		core = {
			XcmpMessageHandler: moonbase_runtime::XcmpQueue,
			LocationToAccountId: moonbase_runtime::xcm_config::LocationToAccountId,
			ParachainInfo: moonbase_runtime::ParachainInfo,
			MessageOrigin: cumulus_primitives_core::AggregateMessageOrigin,
			BlockProducer: crate::network::MoonbaseBlockProducer,
		},
		pallets = {
			PolkadotXcm: moonbase_runtime::PolkadotXcm,
			Balances: moonbase_runtime::Balances,
			EvmForeignAssets: moonbase_runtime::EvmForeignAssets,
			XcmWeightTrader: moonbase_runtime::XcmWeightTrader,
			XcmTransactor: moonbase_runtime::XcmTransactor,
			Treasury: moonbase_runtime::Treasury,
			EthereumXcm: moonbase_runtime::EthereumXcm,
			Proxy: moonbase_runtime::Proxy,
			EVM: moonbase_runtime::EVM,
		}
	}
}

// ---------------------------------------------------------------------------
// Asset Hub Westend declaration (para 1000, real asset-hub-westend-runtime)
// ---------------------------------------------------------------------------
decl_test_parachains! {
	pub struct AssetHubPara {
		genesis = asset_hub_genesis(),
		on_init = {
			asset_hub_westend_runtime::AuraExt::on_initialize(1);
		},
		runtime = asset_hub_westend_runtime,
		core = {
			XcmpMessageHandler: asset_hub_westend_runtime::XcmpQueue,
			LocationToAccountId: asset_hub_westend_runtime::xcm_config::LocationToAccountId,
			ParachainInfo: asset_hub_westend_runtime::ParachainInfo,
			MessageOrigin: cumulus_primitives_core::AggregateMessageOrigin,
		},
		pallets = {
			PolkadotXcm: asset_hub_westend_runtime::PolkadotXcm,
			Balances: asset_hub_westend_runtime::Balances,
			Assets: asset_hub_westend_runtime::Assets,
			ForeignAssets: asset_hub_westend_runtime::ForeignAssets,
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
			AssetHubPara,
			MoonbasePara,
			SiblingPara,
		],
		bridge = ()
	}
}

// ===========================================================================
// Helpers
// ===========================================================================

/// Execute a closure on the Moonbase parachain (para 2004), automatically
/// satisfying mandatory inherent checks.
pub fn moonbase_execute_with<R>(f: impl FnOnce() -> R) -> R {
	MoonbasePara::<PolkadotMoonbeamNet>::execute_with(|| {
		satisfy_moonbase_inherents();
		f()
	})
}

/// Execute a closure on the Sibling parachain (para 2005), automatically
/// satisfying mandatory inherent checks.
pub fn sibling_execute_with<R>(f: impl FnOnce() -> R) -> R {
	SiblingPara::<PolkadotMoonbeamNet>::execute_with(|| {
		satisfy_moonbase_inherents();
		f()
	})
}

/// Execute a closure on Asset Hub (para 1000).
pub fn asset_hub_execute_with<R>(f: impl FnOnce() -> R) -> R {
	AssetHubPara::<PolkadotMoonbeamNet>::execute_with(f)
}

/// Patch storage to satisfy Moonbase's mandatory inherent checks.
/// Called automatically by [`moonbase_execute_with`] / [`sibling_execute_with`].
pub(crate) fn satisfy_moonbase_inherents() {
	pallet_author_inherent::Author::<moonbase_runtime::Runtime>::put(
		moonbase_runtime::AccountId::from([1u8; 20]),
	);
	pallet_author_inherent::InherentIncluded::<moonbase_runtime::Runtime>::put(true);

	frame_support::storage::unhashed::put(
		&frame_support::storage::storage_prefix(b"Randomness", b"InherentIncluded"),
		&(),
	);
	frame_support::storage::unhashed::kill(&frame_support::storage::storage_prefix(
		b"Randomness",
		b"NotFirstBlock",
	));
}

/// Initialise network and clear `NotFirstBlock` on all parachains.
pub fn init_network() {
	// Trigger `Parachain::init()` on every chain by executing on relay.
	WestendRelay::<PolkadotMoonbeamNet>::execute_with(|| {});

	// Clear NotFirstBlock so VRF verification is skipped in subsequent blocks.
	MoonbasePara::<PolkadotMoonbeamNet>::ext_wrapper(|| {
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
/// its price in the XCM weight trader. Call inside `moonbase_execute_with` or
/// `sibling_execute_with`.
pub fn register_dot_asset(asset_id: u128) {
	let dot_location = xcm::latest::Location::parent();

	frame_support::assert_ok!(moonbase_runtime::EvmForeignAssets::create_foreign_asset(
		moonbase_runtime::RuntimeOrigin::root(),
		asset_id,
		dot_location.clone(),
		10,
		b"DOT".to_vec().try_into().unwrap(),
		b"Polkadot".to_vec().try_into().unwrap(),
	));

	// relative_price large enough so that 10 DOT covers XCM execution fees.
	frame_support::assert_ok!(moonbase_runtime::XcmWeightTrader::add_asset(
		moonbase_runtime::RuntimeOrigin::root(),
		dot_location,
		10_000_000_000_000_000_000_000_000_000u128, // 10^28
	));
}

/// Configure `pallet_xcm_transactor` relay indices for Westend.
/// Call inside `moonbase_execute_with` or `sibling_execute_with`.
pub fn set_westend_relay_indices() {
	use frame_support::traits::PalletInfoAccess;
	use pallet_xcm_transactor::relay_indices::RelayChainIndices;

	// Validate pallet indices against the Westend runtime so we fail fast if
	// they drift after a relay upgrade.
	let staking_idx = westend_runtime::Staking::index() as u8;
	let utility_idx = westend_runtime::Utility::index() as u8;
	let hrmp_idx = westend_runtime::Hrmp::index() as u8;

	assert_eq!(staking_idx, 6u8, "Westend Staking pallet index has changed");
	assert_eq!(
		utility_idx, 16u8,
		"Westend Utility pallet index has changed"
	);
	assert_eq!(hrmp_idx, 51u8, "Westend Hrmp pallet index has changed");

	let indices = RelayChainIndices {
		staking: staking_idx,
		utility: utility_idx,
		hrmp: hrmp_idx,
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

	pallet_xcm_transactor::RelayIndices::<moonbase_runtime::Runtime>::put(indices);
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

fn asset_hub_genesis() -> sp_core::storage::Storage {
	use sp_runtime::BuildStorage;

	let endowment: u128 = 1_000_000_000_000_000; // 1 000 WND (12 decimals)

	let mut t = frame_system::GenesisConfig::<asset_hub_westend_runtime::Runtime>::default()
		.build_storage()
		.unwrap();

	parachain_info::GenesisConfig::<asset_hub_westend_runtime::Runtime> {
		parachain_id: ASSET_HUB_PARA_ID.into(),
		_config: Default::default(),
	}
	.assimilate_storage(&mut t)
	.unwrap();

	pallet_balances::GenesisConfig::<asset_hub_westend_runtime::Runtime> {
		balances: vec![
			(sp_runtime::AccountId32::new([1u8; 32]), endowment),
			(sp_runtime::AccountId32::new([2u8; 32]), endowment),
		],
		..Default::default()
	}
	.assimilate_storage(&mut t)
	.unwrap();

	pallet_xcm::GenesisConfig::<asset_hub_westend_runtime::Runtime> {
		safe_xcm_version: Some(xcm::latest::VERSION),
		..Default::default()
	}
	.assimilate_storage(&mut t)
	.unwrap();

	t
}

fn moonbase_genesis(para_id: u32) -> sp_core::storage::Storage {
	use moonbase_runtime::{currency::UNIT, AccountId, Runtime};
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
			(AccountId::from(ALITH), UNIT * 10_000),
			(AccountId::from(BALTATHAR), UNIT * 10_000),
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
