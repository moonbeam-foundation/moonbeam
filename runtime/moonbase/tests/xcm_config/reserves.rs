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

//! Tests for IsReserve (Reserves) configuration.
//!
//! The Reserves type determines which assets are recognized as reserve assets
//! and which origin is allowed to act as reserve for those assets.
//!
//! Moonbase's Reserves configuration allows:
//! - IsBridgedConcreteAssetFrom<AssetHubLocation>: Bridged assets from Asset Hub
//! - IsBridgedConcreteAssetFrom<bp_moonriver::GlobalConsensusLocation>: Assets from Moonriver
//! - Case<RelayChainNativeAssetFromAssetHub>: DOT from Asset Hub
//! - MultiNativeAsset<AbsoluteAndRelativeReserve<SelfLocationAbsolute>>: Self-reserve

use crate::xcm_common::*;
use frame_support::traits::ContainsPair;
use moonbase_runtime::xcm_config::{AssetHubLocation, XcmExecutorConfig};
use xcm::latest::prelude::*;
use xcm_primitives::IsBridgedConcreteAssetFrom;

/// The actual `IsReserve` type wired into the XCM executor.
type IsReserve = <XcmExecutorConfig as xcm_executor::Config>::IsReserve;

const ASSET_HUB_PARA_ID: u32 = 1001;

#[test]
fn reserves_accepts_dot_from_asset_hub() {
	ExtBuilder::default().build().execute_with(|| {
		// DOT asset coming from Asset Hub should be accepted
		let dot_asset = Asset {
			id: AssetId(Location::parent()),
			fun: Fungible(ONE_DOT),
		};
		let asset_hub_origin = Location::new(1, [Parachain(ASSET_HUB_PARA_ID)]);

		// RelayChainNativeAssetFromAssetHub case should match this
		type RelayFromAssetHub =
			xcm_builder::Case<moonbase_runtime::xcm_config::RelayChainNativeAssetFromAssetHub>;

		assert!(
			RelayFromAssetHub::contains(&dot_asset, &asset_hub_origin),
			"DOT from Asset Hub should be accepted as reserve"
		);
		assert!(
			IsReserve::contains(&dot_asset, &asset_hub_origin),
			"IsReserve must accept DOT from Asset Hub (runtime wiring)"
		);
	});
}

#[test]
fn reserves_accepts_bridged_assets_from_asset_hub() {
	ExtBuilder::default().build().execute_with(|| {
		// Bridged asset from another consensus (parents: 2)
		let bridged_asset = Asset {
			id: AssetId(Location::new(
				2,
				[GlobalConsensus(NetworkId::Kusama), Parachain(1000)],
			)),
			fun: Fungible(1_000_000),
		};
		let asset_hub_origin = AssetHubLocation::get();

		// IsBridgedConcreteAssetFrom<AssetHubLocation> should match
		assert!(
			IsBridgedConcreteAssetFrom::<AssetHubLocation>::contains(
				&bridged_asset,
				&asset_hub_origin
			),
			"Bridged assets from Asset Hub should be accepted"
		);
		assert!(
			IsReserve::contains(&bridged_asset, &asset_hub_origin),
			"IsReserve must accept bridged assets from Asset Hub (runtime wiring)"
		);
	});
}

#[test]
fn reserves_rejects_bridged_assets_from_wrong_origin() {
	ExtBuilder::default().build().execute_with(|| {
		// Bridged asset from another consensus
		let bridged_asset = Asset {
			id: AssetId(Location::new(
				2,
				[GlobalConsensus(NetworkId::Kusama), Parachain(1000)],
			)),
			fun: Fungible(1_000_000),
		};
		// Wrong origin - not Asset Hub
		let wrong_origin = Location::new(1, [Parachain(2000)]);

		assert!(
			!IsBridgedConcreteAssetFrom::<AssetHubLocation>::contains(
				&bridged_asset,
				&wrong_origin
			),
			"Bridged assets from wrong origin should be rejected"
		);
	});
}

#[test]
fn reserves_rejects_non_bridged_assets_via_bridged_filter() {
	ExtBuilder::default().build().execute_with(|| {
		// Non-bridged asset (parents: 1, not 2)
		let local_asset = Asset {
			id: AssetId(Location::new(1, [Parachain(1000)])),
			fun: Fungible(1_000_000),
		};
		let asset_hub_origin = AssetHubLocation::get();

		// IsBridgedConcreteAssetFrom requires parents > 1
		assert!(
			!IsBridgedConcreteAssetFrom::<AssetHubLocation>::contains(
				&local_asset,
				&asset_hub_origin
			),
			"Non-bridged assets should not match bridged asset filter"
		);
	});
}

#[test]
fn reserves_accepts_self_reserve() {
	ExtBuilder::default().build().execute_with(|| {
		use frame_support::traits::PalletInfoAccess;
		use moonbase_runtime::xcm_config::SelfLocationAbsolute;
		use moonbase_runtime::Balances;
		use xcm_primitives::{AbsoluteAndRelativeReserve, MultiNativeAsset};

		let self_reserve = Location::new(0, [PalletInstance(Balances::index() as u8)]);

		let native_asset = Asset {
			id: AssetId(self_reserve),
			fun: Fungible(1_000_000_000_000_000_000), // 1 UNIT
		};

		// MultiNativeAsset accepts an asset when the origin matches the asset's
		// reserve. For our own native token the reserve is ourselves
		// (Location::here()), so origin = here() must pass.
		let self_origin = Location::here();

		assert!(
			MultiNativeAsset::<AbsoluteAndRelativeReserve<SelfLocationAbsolute>>::contains(
				&native_asset,
				&self_origin
			),
			"Self reserve asset should be accepted when origin is here()"
		);
		assert!(
			IsReserve::contains(&native_asset, &self_origin),
			"IsReserve must accept self-reserve (runtime wiring)"
		);
	});
}

#[test]
fn reserves_accepts_sibling_native_asset() {
	ExtBuilder::default().build().execute_with(|| {
		// Native asset from a sibling parachain
		let sibling_asset = Asset {
			id: AssetId(Location::new(1, [Parachain(2000), PalletInstance(10)])),
			fun: Fungible(1_000_000),
		};
		let sibling_origin = Location::new(1, [Parachain(2000)]);

		// MultiNativeAsset should accept this - the origin matches the asset's reserve
		// This is checked by matching asset's reserve location to the origin
		use moonbase_runtime::xcm_config::SelfLocationAbsolute;
		use xcm_primitives::{AbsoluteAndRelativeReserve, MultiNativeAsset};

		// The reserve of sibling asset is the sibling chain itself
		// MultiNativeAsset will check if origin matches reserve
		assert!(
			MultiNativeAsset::<AbsoluteAndRelativeReserve<SelfLocationAbsolute>>::contains(
				&sibling_asset,
				&sibling_origin
			),
			"Sibling native asset should be accepted when origin matches reserve"
		);
		assert!(
			IsReserve::contains(&sibling_asset, &sibling_origin),
			"IsReserve must accept sibling native asset (runtime wiring)"
		);
	});
}

#[test]
fn reserves_rejects_asset_with_mismatched_origin() {
	ExtBuilder::default().build().execute_with(|| {
		// Asset claims to be from parachain 2000
		let asset = Asset {
			id: AssetId(Location::new(1, [Parachain(2000), PalletInstance(10)])),
			fun: Fungible(1_000_000),
		};
		// But origin is from parachain 3000
		let wrong_origin = Location::new(1, [Parachain(3000)]);

		use moonbase_runtime::xcm_config::SelfLocationAbsolute;
		use xcm_primitives::{AbsoluteAndRelativeReserve, MultiNativeAsset};

		assert!(
			!MultiNativeAsset::<AbsoluteAndRelativeReserve<SelfLocationAbsolute>>::contains(
				&asset,
				&wrong_origin
			),
			"Asset from mismatched origin should be rejected"
		);
	});
}

#[test]
fn reserves_accepts_dot_from_relay() {
	ExtBuilder::default().build().execute_with(|| {
		let dot_asset = Asset {
			id: AssetId(Location::parent()),
			fun: Fungible(ONE_DOT),
		};
		let relay_origin = Location::parent();

		use moonbase_runtime::xcm_config::SelfLocationAbsolute;
		use xcm_primitives::{AbsoluteAndRelativeReserve, MultiNativeAsset};

		assert!(
			MultiNativeAsset::<AbsoluteAndRelativeReserve<SelfLocationAbsolute>>::contains(
				&dot_asset,
				&relay_origin
			),
			"DOT from relay should be accepted as reserve"
		);
		assert!(
			IsReserve::contains(&dot_asset, &relay_origin),
			"IsReserve must accept DOT from relay (runtime wiring)"
		);
	});
}

#[test]
fn teleport_always_rejected() {
	ExtBuilder::default().build().execute_with(|| {
		type IsTeleporter = <XcmExecutorConfig as xcm_executor::Config>::IsTeleporter;

		let dot = Asset {
			id: AssetId(Location::parent()),
			fun: Fungible(ONE_DOT),
		};
		let relay_origin = Location::parent();

		assert!(
			!IsTeleporter::contains(&dot, &relay_origin),
			"IsTeleporter should reject every asset/origin pair"
		);
	});
}
