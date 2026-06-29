// Copyright 2019-2025 PureStake Inc.
// This file is part of Moonbeam.

//! Hooks for `pallet_xcm` extrinsic benchmarks shared across Moonbeam runtimes.

#[cfg(feature = "runtime-benchmarks")]
use xcm::latest::{Asset, Location};

/// Per-runtime configuration for `pallet_xcm::benchmarking::Config::teleportable_asset_and_dest`.
///
/// When this returns `None`, the upstream benchmark records `Weight::MAX` for
/// `teleport_assets` / `limited_teleport_assets`. Moonbase overrides this to enable
/// re-benchmarking; production weights alias `teleport_assets` to `transfer_assets`.
#[cfg(feature = "runtime-benchmarks")]
pub trait XcmPalletTeleportBenchmark {
	fn teleportable_asset_and_dest() -> Option<(Asset, Location)> {
		None
	}
}
