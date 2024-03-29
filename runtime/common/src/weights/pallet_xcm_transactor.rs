// Copyright 2019-2022 PureStake Inc.
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
//! Autogenerated weights for `pallet_xcm_transactor`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2024-02-22, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `ip-10-0-0-176`, CPU: `Intel(R) Xeon(R) Platinum 8375C CPU @ 2.90GHz`
//! WASM-EXECUTION: `Compiled`, CHAIN: `Some("moonbase-dev")`, DB CACHE: 1024

// Executed Command:
// ./target/production/moonbeam
// benchmark
// pallet
// --chain=moonbase-dev
// --steps=50
// --repeat=20
// --pallet=pallet_xcm_transactor
// --extrinsic=*
// --wasm-execution=compiled
// --header=./file_header.txt
// --output=./runtime/common/src/weights/

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::Weight};
use core::marker::PhantomData;

/// Weight functions for `pallet_xcm_transactor`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_xcm_transactor::WeightInfo for WeightInfo<T> {
	/// Storage: `XcmTransactor::IndexToAccount` (r:1 w:1)
	/// Proof: `XcmTransactor::IndexToAccount` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn register() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `114`
		//  Estimated: `3579`
		// Minimum execution time: 10_295_000 picoseconds.
		Weight::from_parts(10_670_000, 0)
			.saturating_add(Weight::from_parts(0, 3579))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `XcmTransactor::IndexToAccount` (r:0 w:1)
	/// Proof: `XcmTransactor::IndexToAccount` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn deregister() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 6_366_000 picoseconds.
		Weight::from_parts(6_590_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `XcmTransactor::TransactInfoWithWeightLimit` (r:0 w:1)
	/// Proof: `XcmTransactor::TransactInfoWithWeightLimit` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn set_transact_info() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 8_704_000 picoseconds.
		Weight::from_parts(8_995_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `XcmTransactor::TransactInfoWithWeightLimit` (r:0 w:1)
	/// Proof: `XcmTransactor::TransactInfoWithWeightLimit` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn remove_transact_info() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 8_048_000 picoseconds.
		Weight::from_parts(8_196_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `XcmTransactor::DestinationAssetFeePerSecond` (r:0 w:1)
	/// Proof: `XcmTransactor::DestinationAssetFeePerSecond` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn set_fee_per_second() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 7_886_000 picoseconds.
		Weight::from_parts(8_275_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `AssetManager::AssetIdType` (r:1 w:0)
	/// Proof: `AssetManager::AssetIdType` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `XcmTransactor::IndexToAccount` (r:1 w:0)
	/// Proof: `XcmTransactor::IndexToAccount` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `XcmTransactor::RelayIndices` (r:1 w:0)
	/// Proof: `XcmTransactor::RelayIndices` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `XcmTransactor::TransactInfoWithWeightLimit` (r:1 w:0)
	/// Proof: `XcmTransactor::TransactInfoWithWeightLimit` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `XcmTransactor::DestinationAssetFeePerSecond` (r:1 w:0)
	/// Proof: `XcmTransactor::DestinationAssetFeePerSecond` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `AssetManager::AssetTypeId` (r:1 w:0)
	/// Proof: `AssetManager::AssetTypeId` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Assets::Asset` (r:1 w:0)
	/// Proof: `Assets::Asset` (`max_values`: None, `max_size`: Some(174), added: 2649, mode: `MaxEncodedLen`)
	fn transact_through_derivative() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `489`
		//  Estimated: `3954`
		// Minimum execution time: 32_015_000 picoseconds.
		Weight::from_parts(32_565_000, 0)
			.saturating_add(Weight::from_parts(0, 3954))
			.saturating_add(T::DbWeight::get().reads(7))
	}
	/// Storage: `AssetManager::AssetIdType` (r:1 w:0)
	/// Proof: `AssetManager::AssetIdType` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `XcmTransactor::TransactInfoWithWeightLimit` (r:1 w:0)
	/// Proof: `XcmTransactor::TransactInfoWithWeightLimit` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `XcmTransactor::DestinationAssetFeePerSecond` (r:1 w:0)
	/// Proof: `XcmTransactor::DestinationAssetFeePerSecond` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `AssetManager::AssetTypeId` (r:1 w:0)
	/// Proof: `AssetManager::AssetTypeId` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Assets::Asset` (r:1 w:0)
	/// Proof: `Assets::Asset` (`max_values`: None, `max_size`: Some(174), added: 2649, mode: `MaxEncodedLen`)
	fn transact_through_sovereign() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `423`
		//  Estimated: `3888`
		// Minimum execution time: 24_600_000 picoseconds.
		Weight::from_parts(25_307_000, 0)
			.saturating_add(Weight::from_parts(0, 3888))
			.saturating_add(T::DbWeight::get().reads(5))
	}
	/// Storage: `AssetManager::AssetIdType` (r:1 w:0)
	/// Proof: `AssetManager::AssetIdType` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `XcmTransactor::TransactInfoWithWeightLimit` (r:1 w:0)
	/// Proof: `XcmTransactor::TransactInfoWithWeightLimit` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `XcmTransactor::DestinationAssetFeePerSecond` (r:1 w:0)
	/// Proof: `XcmTransactor::DestinationAssetFeePerSecond` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `PolkadotXcm::SupportedVersion` (r:1 w:0)
	/// Proof: `PolkadotXcm::SupportedVersion` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `PolkadotXcm::VersionDiscoveryQueue` (r:1 w:1)
	/// Proof: `PolkadotXcm::VersionDiscoveryQueue` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `PolkadotXcm::SafeXcmVersion` (r:1 w:0)
	/// Proof: `PolkadotXcm::SafeXcmVersion` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `ParachainSystem::HostConfiguration` (r:1 w:0)
	/// Proof: `ParachainSystem::HostConfiguration` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `ParachainSystem::PendingUpwardMessages` (r:1 w:1)
	/// Proof: `ParachainSystem::PendingUpwardMessages` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	fn transact_through_signed() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `467`
		//  Estimated: `3932`
		// Minimum execution time: 46_836_000 picoseconds.
		Weight::from_parts(47_576_000, 0)
			.saturating_add(Weight::from_parts(0, 3932))
			.saturating_add(T::DbWeight::get().reads(8))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: `XcmTransactor::RelayIndices` (r:1 w:0)
	/// Proof: `XcmTransactor::RelayIndices` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `AssetManager::AssetIdType` (r:1 w:0)
	/// Proof: `AssetManager::AssetIdType` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `XcmTransactor::TransactInfoWithWeightLimit` (r:1 w:0)
	/// Proof: `XcmTransactor::TransactInfoWithWeightLimit` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `XcmTransactor::DestinationAssetFeePerSecond` (r:1 w:0)
	/// Proof: `XcmTransactor::DestinationAssetFeePerSecond` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `PolkadotXcm::SupportedVersion` (r:1 w:0)
	/// Proof: `PolkadotXcm::SupportedVersion` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `PolkadotXcm::VersionDiscoveryQueue` (r:1 w:1)
	/// Proof: `PolkadotXcm::VersionDiscoveryQueue` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `PolkadotXcm::SafeXcmVersion` (r:1 w:0)
	/// Proof: `PolkadotXcm::SafeXcmVersion` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `ParachainSystem::HostConfiguration` (r:1 w:0)
	/// Proof: `ParachainSystem::HostConfiguration` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `ParachainSystem::PendingUpwardMessages` (r:1 w:1)
	/// Proof: `ParachainSystem::PendingUpwardMessages` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	fn hrmp_manage() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `471`
		//  Estimated: `3936`
		// Minimum execution time: 51_398_000 picoseconds.
		Weight::from_parts(52_067_000, 0)
			.saturating_add(Weight::from_parts(0, 3936))
			.saturating_add(T::DbWeight::get().reads(9))
			.saturating_add(T::DbWeight::get().writes(2))
	}
}
