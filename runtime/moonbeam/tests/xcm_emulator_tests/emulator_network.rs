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
//! Wires a Westend relay chain and the real Moonbeam runtime into a single
//! test network using the `decl_test_*` macros from `xcm-emulator`.

use crate::emulator_relay;

// The emulator macros expand code that calls `OnInitialize` / `OnFinalize`
// on `AllPalletsWithoutSystem`, so these traits must be in scope.
use frame_support::traits::OnInitialize;
use xcm_emulator::decl_test_networks;
use xcm_emulator::decl_test_parachains;
use xcm_emulator::decl_test_relay_chains;
use xcm_emulator::Parachain;
use xcm_emulator::TestExt;

pub const MOONBEAM_PARA_ID: u32 = 2004;

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
		}
	}
}

// ---------------------------------------------------------------------------
// Moonbeam parachain declaration
// ---------------------------------------------------------------------------
decl_test_parachains! {
	pub struct MoonbeamPara {
		genesis = moonbeam_genesis(),
		on_init = {
			// Satisfy Moonbeam's mandatory inherent checks for the
			// very first block created during `Parachain::init()`.
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
		],
		bridge = ()
	}
}

// ---------------------------------------------------------------------------
// Moonbeam per-block workaround
// ---------------------------------------------------------------------------

/// Execute a closure on the Moonbeam parachain, automatically satisfying
/// mandatory inherent checks before the closure returns.
///
/// **Always use this instead of `MoonbeamPara::execute_with` directly.**
/// Moonbeam's `pallet_author_inherent` and `pallet_randomness` assert in
/// `on_finalize` that their inherents were dispatched. The emulator doesn't
/// dispatch them, so every block would panic without the fixup. This wrapper
/// ensures the fixup is never forgotten.
pub fn moonbeam_execute_with<R>(f: impl FnOnce() -> R) -> R {
	MoonbeamPara::<PolkadotMoonbeamNet>::execute_with(|| {
		satisfy_moonbeam_inherents();
		f()
	})
}

/// Patch storage to satisfy Moonbeam's mandatory inherent checks.
///
/// Called automatically by [`moonbeam_execute_with`]. You should not need to
/// call this directly.
fn satisfy_moonbeam_inherents() {
	// Author inherent
	pallet_author_inherent::Author::<moonbeam_runtime::Runtime>::put(
		moonbeam_runtime::AccountId::from([1u8; 20]),
	);
	pallet_author_inherent::InherentIncluded::<moonbeam_runtime::Runtime>::put(true);

	// Randomness inherent (storage is pub(crate), write directly)
	frame_support::storage::unhashed::put(
		&frame_support::storage::storage_prefix(b"Randomness", b"InherentIncluded"),
		&(),
	);

	// Reset `NotFirstBlock` so the NEXT block's `on_initialize` takes the
	// genesis path and skips VRF verification (which requires a VRF pre-
	// digest we cannot inject through the emulator).
	frame_support::storage::unhashed::kill(&frame_support::storage::storage_prefix(
		b"Randomness",
		b"NotFirstBlock",
	));
}

// ---------------------------------------------------------------------------
// Moonbeam genesis helper
// ---------------------------------------------------------------------------

/// Build a minimal `Storage` for Moonbeam in the emulator network.
///
/// We replicate the essentials from `ExtBuilder` but return raw `Storage`
/// instead of `TestExternalities`, as required by the emulator macros.
fn moonbeam_genesis() -> sp_core::storage::Storage {
	use moonbeam_runtime::{currency::GLMR, AccountId, Runtime};
	use sp_runtime::BuildStorage;

	let mut t = frame_system::GenesisConfig::<Runtime>::default()
		.build_storage()
		.unwrap();

	parachain_info::GenesisConfig::<Runtime> {
		parachain_id: MOONBEAM_PARA_ID.into(),
		_config: Default::default(),
	}
	.assimilate_storage(&mut t)
	.unwrap();

	pallet_balances::GenesisConfig::<Runtime> {
		balances: vec![
			(AccountId::from([1u8; 20]), GLMR * 1000),
			(AccountId::from([2u8; 20]), GLMR * 1000),
			(AccountId::from([3u8; 20]), GLMR * 1000),
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
