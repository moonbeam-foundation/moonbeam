// Copyright 2019-2025 PureStake Inc.
// This file is part of Moonbeam.

//! Hooks for `pallet_xcm` extrinsic benchmarks shared across Moonbeam runtimes.

#[cfg(feature = "runtime-benchmarks")]
use xcm::latest::{Asset, Location};

/// Per-runtime configuration for `pallet_xcm::benchmarking::Config::teleportable_asset_and_dest`.
///
/// When this returns `None`, the upstream benchmark records `Weight::MAX` for
/// `teleport_assets` / `limited_teleport_assets`. Runtimes may override this to
/// return a concrete asset/dest and enable re-benchmarking; Moonbase currently
/// keeps the default `None` and instead assigns a conservative
/// `teleport_assets` weight (`transfer_assets` + worst-case ERC-20 transfer).
/// Production runtimes (Moonbeam / Moonriver) also keep `None` while teleport
/// remains gated off.
#[cfg(feature = "runtime-benchmarks")]
pub trait XcmPalletTeleportBenchmark {
	fn teleportable_asset_and_dest() -> Option<(Asset, Location)> {
		None
	}
}
