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


//! Autogenerated weights for moonbeam_xcm_benchmarks_generic
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-04-28, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `benchmarker`, CPU: `Intel(R) Core(TM) i7-7700K CPU @ 4.20GHz`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: None, DB CACHE: 1024

// Executed Command:
// ./target/release/moonbeam
// benchmark
// pallet
// --execution=wasm
// --wasm-execution=compiled
// --pallet
// *
// --extrinsic
// *
// --steps
// 50
// --repeat
// 20
// --template=./benchmarking/frame-weight-template.hbs
// --json-file
// raw.json
// --output
// weights/

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for moonbeam_xcm_benchmarks_generic.
pub trait WeightInfo {
	fn report_holding() -> Weight;
	fn buy_execution() -> Weight;
	fn query_response() -> Weight;
	fn transact() -> Weight;
	fn refund_surplus() -> Weight;
	fn set_error_handler() -> Weight;
	fn set_appendix() -> Weight;
	fn clear_error() -> Weight;
	fn descend_origin() -> Weight;
	fn clear_origin() -> Weight;
	fn report_error() -> Weight;
	fn claim_asset() -> Weight;
	fn trap() -> Weight;
	fn subscribe_version() -> Weight;
	fn unsubscribe_version() -> Weight;
	fn initiate_reserve_withdraw() -> Weight;
	fn burn_asset() -> Weight;
	fn expect_asset() -> Weight;
	fn expect_origin() -> Weight;
	fn expect_error() -> Weight;
	fn expect_transact_status() -> Weight;
	fn query_pallet() -> Weight;
	fn expect_pallet() -> Weight;
	fn report_transact_status() -> Weight;
	fn clear_transact_status() -> Weight;
	fn set_topic() -> Weight;
	fn clear_topic() -> Weight;
	fn exchange_asset() -> Weight;
	fn universal_origin() -> Weight;
	fn set_fees_mode() -> Weight;
	fn lock_asset() -> Weight;
	fn unlock_asset() -> Weight;
	fn note_unlockable() -> Weight;
	fn request_unlock() -> Weight;
	fn unpaid_execution() -> Weight;
}

/// Weights for moonbeam_xcm_benchmarks_generic using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	/// Storage: PolkadotXcm SupportedVersion (r:1 w:0)
	/// Proof Skipped: PolkadotXcm SupportedVersion (max_values: None, max_size: None, mode: Measured)
	/// Storage: PolkadotXcm VersionDiscoveryQueue (r:1 w:1)
	/// Proof Skipped: PolkadotXcm VersionDiscoveryQueue (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: PolkadotXcm SafeXcmVersion (r:1 w:0)
	/// Proof Skipped: PolkadotXcm SafeXcmVersion (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: ParachainSystem HostConfiguration (r:1 w:0)
	/// Proof Skipped: ParachainSystem HostConfiguration (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: ParachainSystem PendingUpwardMessages (r:1 w:1)
	/// Proof Skipped: ParachainSystem PendingUpwardMessages (max_values: Some(1), max_size: None, mode: Measured)
	fn report_holding() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `145`
		//  Estimated: `10130`
		// Minimum execution time: 613_728_000 picoseconds.
		Weight::from_parts(616_143_000, 10130)
			.saturating_add(T::DbWeight::get().reads(5_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	/// Storage: AssetManager SupportedFeePaymentAssets (r:1 w:0)
	/// Proof Skipped: AssetManager SupportedFeePaymentAssets (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: AssetManager AssetTypeUnitsPerSecond (r:1 w:0)
	/// Proof Skipped: AssetManager AssetTypeUnitsPerSecond (max_values: None, max_size: None, mode: Measured)
	/// Storage: AssetManager AssetTypeId (r:1 w:0)
	/// Proof Skipped: AssetManager AssetTypeId (max_values: None, max_size: None, mode: Measured)
	/// Storage: Assets Asset (r:1 w:0)
	/// Proof: Assets Asset (max_values: None, max_size: Some(174), added: 2649, mode: MaxEncodedLen)
	fn buy_execution() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `2334`
		//  Estimated: `19056`
		// Minimum execution time: 180_427_000 picoseconds.
		Weight::from_parts(181_080_000, 19056)
			.saturating_add(T::DbWeight::get().reads(4_u64))
	}
	/// Storage: PolkadotXcm Queries (r:1 w:0)
	/// Proof Skipped: PolkadotXcm Queries (max_values: None, max_size: None, mode: Measured)
	fn query_response() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `69`
		//  Estimated: `3534`
		// Minimum execution time: 17_147_000 picoseconds.
		Weight::from_parts(17_428_000, 3534)
			.saturating_add(T::DbWeight::get().reads(1_u64))
	}
	/// Storage: MaintenanceMode MaintenanceMode (r:1 w:0)
	/// Proof Skipped: MaintenanceMode MaintenanceMode (max_values: Some(1), max_size: None, mode: Measured)
	fn transact() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `42`
		//  Estimated: `1527`
		// Minimum execution time: 24_018_000 picoseconds.
		Weight::from_parts(24_375_000, 1527)
			.saturating_add(T::DbWeight::get().reads(1_u64))
	}
	fn refund_surplus() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 7_851_000 picoseconds.
		Weight::from_parts(7_975_000, 0)
	}
	fn set_error_handler() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 5_072_000 picoseconds.
		Weight::from_parts(5_182_000, 0)
	}
	fn set_appendix() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 4_983_000 picoseconds.
		Weight::from_parts(5_123_000, 0)
	}
	fn clear_error() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 5_017_000 picoseconds.
		Weight::from_parts(5_147_000, 0)
	}
	fn descend_origin() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 5_870_000 picoseconds.
		Weight::from_parts(5_992_000, 0)
	}
	fn clear_origin() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 5_059_000 picoseconds.
		Weight::from_parts(5_194_000, 0)
	}
	/// Storage: PolkadotXcm SupportedVersion (r:1 w:0)
	/// Proof Skipped: PolkadotXcm SupportedVersion (max_values: None, max_size: None, mode: Measured)
	/// Storage: PolkadotXcm VersionDiscoveryQueue (r:1 w:1)
	/// Proof Skipped: PolkadotXcm VersionDiscoveryQueue (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: PolkadotXcm SafeXcmVersion (r:1 w:0)
	/// Proof Skipped: PolkadotXcm SafeXcmVersion (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: ParachainSystem HostConfiguration (r:1 w:0)
	/// Proof Skipped: ParachainSystem HostConfiguration (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: ParachainSystem PendingUpwardMessages (r:1 w:1)
	/// Proof Skipped: ParachainSystem PendingUpwardMessages (max_values: Some(1), max_size: None, mode: Measured)
	fn report_error() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `145`
		//  Estimated: `10130`
		// Minimum execution time: 35_671_000 picoseconds.
		Weight::from_parts(36_231_000, 10130)
			.saturating_add(T::DbWeight::get().reads(5_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	/// Storage: PolkadotXcm AssetTraps (r:1 w:1)
	/// Proof Skipped: PolkadotXcm AssetTraps (max_values: None, max_size: None, mode: Measured)
	fn claim_asset() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `126`
		//  Estimated: `3591`
		// Minimum execution time: 23_093_000 picoseconds.
		Weight::from_parts(23_438_000, 3591)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	fn trap() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 5_182_000 picoseconds.
		Weight::from_parts(5_281_000, 0)
	}
	/// Storage: PolkadotXcm VersionNotifyTargets (r:1 w:1)
	/// Proof Skipped: PolkadotXcm VersionNotifyTargets (max_values: None, max_size: None, mode: Measured)
	/// Storage: PolkadotXcm SupportedVersion (r:1 w:0)
	/// Proof Skipped: PolkadotXcm SupportedVersion (max_values: None, max_size: None, mode: Measured)
	/// Storage: PolkadotXcm VersionDiscoveryQueue (r:1 w:1)
	/// Proof Skipped: PolkadotXcm VersionDiscoveryQueue (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: PolkadotXcm SafeXcmVersion (r:1 w:0)
	/// Proof Skipped: PolkadotXcm SafeXcmVersion (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: ParachainSystem HostConfiguration (r:1 w:0)
	/// Proof Skipped: ParachainSystem HostConfiguration (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: ParachainSystem PendingUpwardMessages (r:1 w:1)
	/// Proof Skipped: ParachainSystem PendingUpwardMessages (max_values: Some(1), max_size: None, mode: Measured)
	fn subscribe_version() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `145`
		//  Estimated: `13740`
		// Minimum execution time: 42_060_000 picoseconds.
		Weight::from_parts(42_607_000, 13740)
			.saturating_add(T::DbWeight::get().reads(6_u64))
			.saturating_add(T::DbWeight::get().writes(3_u64))
	}
	/// Storage: PolkadotXcm VersionNotifyTargets (r:0 w:1)
	/// Proof Skipped: PolkadotXcm VersionNotifyTargets (max_values: None, max_size: None, mode: Measured)
	fn unsubscribe_version() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 8_521_000 picoseconds.
		Weight::from_parts(8_689_000, 0)
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: PolkadotXcm SupportedVersion (r:1 w:0)
	/// Proof Skipped: PolkadotXcm SupportedVersion (max_values: None, max_size: None, mode: Measured)
	/// Storage: PolkadotXcm VersionDiscoveryQueue (r:1 w:1)
	/// Proof Skipped: PolkadotXcm VersionDiscoveryQueue (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: PolkadotXcm SafeXcmVersion (r:1 w:0)
	/// Proof Skipped: PolkadotXcm SafeXcmVersion (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: ParachainSystem HostConfiguration (r:1 w:0)
	/// Proof Skipped: ParachainSystem HostConfiguration (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: ParachainSystem PendingUpwardMessages (r:1 w:1)
	/// Proof Skipped: ParachainSystem PendingUpwardMessages (max_values: Some(1), max_size: None, mode: Measured)
	fn initiate_reserve_withdraw() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `145`
		//  Estimated: `10130`
		// Minimum execution time: 713_056_000 picoseconds.
		Weight::from_parts(715_975_000, 10130)
			.saturating_add(T::DbWeight::get().reads(5_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	fn burn_asset() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 226_643_000 picoseconds.
		Weight::from_parts(227_558_000, 0)
	}
	fn expect_asset() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 19_959_000 picoseconds.
		Weight::from_parts(20_434_000, 0)
	}
	fn expect_origin() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 5_047_000 picoseconds.
		Weight::from_parts(5_211_000, 0)
	}
	fn expect_error() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 5_051_000 picoseconds.
		Weight::from_parts(5_141_000, 0)
	}
	fn expect_transact_status() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 5_189_000 picoseconds.
		Weight::from_parts(5_381_000, 0)
	}
	/// Storage: PolkadotXcm SupportedVersion (r:1 w:0)
	/// Proof Skipped: PolkadotXcm SupportedVersion (max_values: None, max_size: None, mode: Measured)
	/// Storage: PolkadotXcm VersionDiscoveryQueue (r:1 w:1)
	/// Proof Skipped: PolkadotXcm VersionDiscoveryQueue (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: PolkadotXcm SafeXcmVersion (r:1 w:0)
	/// Proof Skipped: PolkadotXcm SafeXcmVersion (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: ParachainSystem HostConfiguration (r:1 w:0)
	/// Proof Skipped: ParachainSystem HostConfiguration (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: ParachainSystem PendingUpwardMessages (r:1 w:1)
	/// Proof Skipped: ParachainSystem PendingUpwardMessages (max_values: Some(1), max_size: None, mode: Measured)
	fn query_pallet() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `145`
		//  Estimated: `10130`
		// Minimum execution time: 50_026_000 picoseconds.
		Weight::from_parts(50_578_000, 10130)
			.saturating_add(T::DbWeight::get().reads(5_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	fn expect_pallet() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 17_409_000 picoseconds.
		Weight::from_parts(17_794_000, 0)
	}
	/// Storage: PolkadotXcm SupportedVersion (r:1 w:0)
	/// Proof Skipped: PolkadotXcm SupportedVersion (max_values: None, max_size: None, mode: Measured)
	/// Storage: PolkadotXcm VersionDiscoveryQueue (r:1 w:1)
	/// Proof Skipped: PolkadotXcm VersionDiscoveryQueue (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: PolkadotXcm SafeXcmVersion (r:1 w:0)
	/// Proof Skipped: PolkadotXcm SafeXcmVersion (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: ParachainSystem HostConfiguration (r:1 w:0)
	/// Proof Skipped: ParachainSystem HostConfiguration (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: ParachainSystem PendingUpwardMessages (r:1 w:1)
	/// Proof Skipped: ParachainSystem PendingUpwardMessages (max_values: Some(1), max_size: None, mode: Measured)
	fn report_transact_status() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `145`
		//  Estimated: `10130`
		// Minimum execution time: 35_835_000 picoseconds.
		Weight::from_parts(36_090_000, 10130)
			.saturating_add(T::DbWeight::get().reads(5_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	fn clear_transact_status() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 5_132_000 picoseconds.
		Weight::from_parts(5_280_000, 0)
	}
	fn set_topic() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 5_095_000 picoseconds.
		Weight::from_parts(5_215_000, 0)
	}
	fn clear_topic() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 5_099_000 picoseconds.
		Weight::from_parts(5_214_000, 0)
	}
	/// Storage: Benchmark Override (r:0 w:0)
	/// Proof Skipped: Benchmark Override (max_values: None, max_size: None, mode: Measured)
	fn exchange_asset() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 500_000_000_000 picoseconds.
		Weight::from_parts(500_000_000_000, 0)
	}
	/// Storage: Benchmark Override (r:0 w:0)
	/// Proof Skipped: Benchmark Override (max_values: None, max_size: None, mode: Measured)
	fn universal_origin() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 500_000_000_000 picoseconds.
		Weight::from_parts(500_000_000_000, 0)
	}
	fn set_fees_mode() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 5_066_000 picoseconds.
		Weight::from_parts(5_215_000, 0)
	}
	/// Storage: Benchmark Override (r:0 w:0)
	/// Proof Skipped: Benchmark Override (max_values: None, max_size: None, mode: Measured)
	fn lock_asset() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 500_000_000_000 picoseconds.
		Weight::from_parts(500_000_000_000, 0)
	}
	/// Storage: Benchmark Override (r:0 w:0)
	/// Proof Skipped: Benchmark Override (max_values: None, max_size: None, mode: Measured)
	fn unlock_asset() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 500_000_000_000 picoseconds.
		Weight::from_parts(500_000_000_000, 0)
	}
	/// Storage: Benchmark Override (r:0 w:0)
	/// Proof Skipped: Benchmark Override (max_values: None, max_size: None, mode: Measured)
	fn note_unlockable() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 500_000_000_000 picoseconds.
		Weight::from_parts(500_000_000_000, 0)
	}
	/// Storage: Benchmark Override (r:0 w:0)
	/// Proof Skipped: Benchmark Override (max_values: None, max_size: None, mode: Measured)
	fn request_unlock() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 500_000_000_000 picoseconds.
		Weight::from_parts(500_000_000_000, 0)
	}
	fn unpaid_execution() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 5_249_000 picoseconds.
		Weight::from_parts(5_371_000, 0)
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	/// Storage: PolkadotXcm SupportedVersion (r:1 w:0)
	/// Proof Skipped: PolkadotXcm SupportedVersion (max_values: None, max_size: None, mode: Measured)
	/// Storage: PolkadotXcm VersionDiscoveryQueue (r:1 w:1)
	/// Proof Skipped: PolkadotXcm VersionDiscoveryQueue (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: PolkadotXcm SafeXcmVersion (r:1 w:0)
	/// Proof Skipped: PolkadotXcm SafeXcmVersion (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: ParachainSystem HostConfiguration (r:1 w:0)
	/// Proof Skipped: ParachainSystem HostConfiguration (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: ParachainSystem PendingUpwardMessages (r:1 w:1)
	/// Proof Skipped: ParachainSystem PendingUpwardMessages (max_values: Some(1), max_size: None, mode: Measured)
	fn report_holding() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `145`
		//  Estimated: `10130`
		// Minimum execution time: 613_728_000 picoseconds.
		Weight::from_parts(616_143_000, 10130)
			.saturating_add(RocksDbWeight::get().reads(5_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
	/// Storage: AssetManager SupportedFeePaymentAssets (r:1 w:0)
	/// Proof Skipped: AssetManager SupportedFeePaymentAssets (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: AssetManager AssetTypeUnitsPerSecond (r:1 w:0)
	/// Proof Skipped: AssetManager AssetTypeUnitsPerSecond (max_values: None, max_size: None, mode: Measured)
	/// Storage: AssetManager AssetTypeId (r:1 w:0)
	/// Proof Skipped: AssetManager AssetTypeId (max_values: None, max_size: None, mode: Measured)
	/// Storage: Assets Asset (r:1 w:0)
	/// Proof: Assets Asset (max_values: None, max_size: Some(174), added: 2649, mode: MaxEncodedLen)
	fn buy_execution() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `2334`
		//  Estimated: `19056`
		// Minimum execution time: 180_427_000 picoseconds.
		Weight::from_parts(181_080_000, 19056)
			.saturating_add(RocksDbWeight::get().reads(4_u64))
	}
	/// Storage: PolkadotXcm Queries (r:1 w:0)
	/// Proof Skipped: PolkadotXcm Queries (max_values: None, max_size: None, mode: Measured)
	fn query_response() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `69`
		//  Estimated: `3534`
		// Minimum execution time: 17_147_000 picoseconds.
		Weight::from_parts(17_428_000, 3534)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
	}
	/// Storage: MaintenanceMode MaintenanceMode (r:1 w:0)
	/// Proof Skipped: MaintenanceMode MaintenanceMode (max_values: Some(1), max_size: None, mode: Measured)
	fn transact() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `42`
		//  Estimated: `1527`
		// Minimum execution time: 24_018_000 picoseconds.
		Weight::from_parts(24_375_000, 1527)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
	}
	fn refund_surplus() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 7_851_000 picoseconds.
		Weight::from_parts(7_975_000, 0)
	}
	fn set_error_handler() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 5_072_000 picoseconds.
		Weight::from_parts(5_182_000, 0)
	}
	fn set_appendix() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 4_983_000 picoseconds.
		Weight::from_parts(5_123_000, 0)
	}
	fn clear_error() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 5_017_000 picoseconds.
		Weight::from_parts(5_147_000, 0)
	}
	fn descend_origin() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 5_870_000 picoseconds.
		Weight::from_parts(5_992_000, 0)
	}
	fn clear_origin() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 5_059_000 picoseconds.
		Weight::from_parts(5_194_000, 0)
	}
	/// Storage: PolkadotXcm SupportedVersion (r:1 w:0)
	/// Proof Skipped: PolkadotXcm SupportedVersion (max_values: None, max_size: None, mode: Measured)
	/// Storage: PolkadotXcm VersionDiscoveryQueue (r:1 w:1)
	/// Proof Skipped: PolkadotXcm VersionDiscoveryQueue (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: PolkadotXcm SafeXcmVersion (r:1 w:0)
	/// Proof Skipped: PolkadotXcm SafeXcmVersion (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: ParachainSystem HostConfiguration (r:1 w:0)
	/// Proof Skipped: ParachainSystem HostConfiguration (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: ParachainSystem PendingUpwardMessages (r:1 w:1)
	/// Proof Skipped: ParachainSystem PendingUpwardMessages (max_values: Some(1), max_size: None, mode: Measured)
	fn report_error() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `145`
		//  Estimated: `10130`
		// Minimum execution time: 35_671_000 picoseconds.
		Weight::from_parts(36_231_000, 10130)
			.saturating_add(RocksDbWeight::get().reads(5_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
	/// Storage: PolkadotXcm AssetTraps (r:1 w:1)
	/// Proof Skipped: PolkadotXcm AssetTraps (max_values: None, max_size: None, mode: Measured)
	fn claim_asset() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `126`
		//  Estimated: `3591`
		// Minimum execution time: 23_093_000 picoseconds.
		Weight::from_parts(23_438_000, 3591)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	fn trap() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 5_182_000 picoseconds.
		Weight::from_parts(5_281_000, 0)
	}
	/// Storage: PolkadotXcm VersionNotifyTargets (r:1 w:1)
	/// Proof Skipped: PolkadotXcm VersionNotifyTargets (max_values: None, max_size: None, mode: Measured)
	/// Storage: PolkadotXcm SupportedVersion (r:1 w:0)
	/// Proof Skipped: PolkadotXcm SupportedVersion (max_values: None, max_size: None, mode: Measured)
	/// Storage: PolkadotXcm VersionDiscoveryQueue (r:1 w:1)
	/// Proof Skipped: PolkadotXcm VersionDiscoveryQueue (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: PolkadotXcm SafeXcmVersion (r:1 w:0)
	/// Proof Skipped: PolkadotXcm SafeXcmVersion (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: ParachainSystem HostConfiguration (r:1 w:0)
	/// Proof Skipped: ParachainSystem HostConfiguration (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: ParachainSystem PendingUpwardMessages (r:1 w:1)
	/// Proof Skipped: ParachainSystem PendingUpwardMessages (max_values: Some(1), max_size: None, mode: Measured)
	fn subscribe_version() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `145`
		//  Estimated: `13740`
		// Minimum execution time: 42_060_000 picoseconds.
		Weight::from_parts(42_607_000, 13740)
			.saturating_add(RocksDbWeight::get().reads(6_u64))
			.saturating_add(RocksDbWeight::get().writes(3_u64))
	}
	/// Storage: PolkadotXcm VersionNotifyTargets (r:0 w:1)
	/// Proof Skipped: PolkadotXcm VersionNotifyTargets (max_values: None, max_size: None, mode: Measured)
	fn unsubscribe_version() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 8_521_000 picoseconds.
		Weight::from_parts(8_689_000, 0)
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: PolkadotXcm SupportedVersion (r:1 w:0)
	/// Proof Skipped: PolkadotXcm SupportedVersion (max_values: None, max_size: None, mode: Measured)
	/// Storage: PolkadotXcm VersionDiscoveryQueue (r:1 w:1)
	/// Proof Skipped: PolkadotXcm VersionDiscoveryQueue (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: PolkadotXcm SafeXcmVersion (r:1 w:0)
	/// Proof Skipped: PolkadotXcm SafeXcmVersion (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: ParachainSystem HostConfiguration (r:1 w:0)
	/// Proof Skipped: ParachainSystem HostConfiguration (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: ParachainSystem PendingUpwardMessages (r:1 w:1)
	/// Proof Skipped: ParachainSystem PendingUpwardMessages (max_values: Some(1), max_size: None, mode: Measured)
	fn initiate_reserve_withdraw() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `145`
		//  Estimated: `10130`
		// Minimum execution time: 713_056_000 picoseconds.
		Weight::from_parts(715_975_000, 10130)
			.saturating_add(RocksDbWeight::get().reads(5_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
	fn burn_asset() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 226_643_000 picoseconds.
		Weight::from_parts(227_558_000, 0)
	}
	fn expect_asset() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 19_959_000 picoseconds.
		Weight::from_parts(20_434_000, 0)
	}
	fn expect_origin() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 5_047_000 picoseconds.
		Weight::from_parts(5_211_000, 0)
	}
	fn expect_error() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 5_051_000 picoseconds.
		Weight::from_parts(5_141_000, 0)
	}
	fn expect_transact_status() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 5_189_000 picoseconds.
		Weight::from_parts(5_381_000, 0)
	}
	/// Storage: PolkadotXcm SupportedVersion (r:1 w:0)
	/// Proof Skipped: PolkadotXcm SupportedVersion (max_values: None, max_size: None, mode: Measured)
	/// Storage: PolkadotXcm VersionDiscoveryQueue (r:1 w:1)
	/// Proof Skipped: PolkadotXcm VersionDiscoveryQueue (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: PolkadotXcm SafeXcmVersion (r:1 w:0)
	/// Proof Skipped: PolkadotXcm SafeXcmVersion (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: ParachainSystem HostConfiguration (r:1 w:0)
	/// Proof Skipped: ParachainSystem HostConfiguration (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: ParachainSystem PendingUpwardMessages (r:1 w:1)
	/// Proof Skipped: ParachainSystem PendingUpwardMessages (max_values: Some(1), max_size: None, mode: Measured)
	fn query_pallet() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `145`
		//  Estimated: `10130`
		// Minimum execution time: 50_026_000 picoseconds.
		Weight::from_parts(50_578_000, 10130)
			.saturating_add(RocksDbWeight::get().reads(5_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
	fn expect_pallet() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 17_409_000 picoseconds.
		Weight::from_parts(17_794_000, 0)
	}
	/// Storage: PolkadotXcm SupportedVersion (r:1 w:0)
	/// Proof Skipped: PolkadotXcm SupportedVersion (max_values: None, max_size: None, mode: Measured)
	/// Storage: PolkadotXcm VersionDiscoveryQueue (r:1 w:1)
	/// Proof Skipped: PolkadotXcm VersionDiscoveryQueue (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: PolkadotXcm SafeXcmVersion (r:1 w:0)
	/// Proof Skipped: PolkadotXcm SafeXcmVersion (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: ParachainSystem HostConfiguration (r:1 w:0)
	/// Proof Skipped: ParachainSystem HostConfiguration (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: ParachainSystem PendingUpwardMessages (r:1 w:1)
	/// Proof Skipped: ParachainSystem PendingUpwardMessages (max_values: Some(1), max_size: None, mode: Measured)
	fn report_transact_status() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `145`
		//  Estimated: `10130`
		// Minimum execution time: 35_835_000 picoseconds.
		Weight::from_parts(36_090_000, 10130)
			.saturating_add(RocksDbWeight::get().reads(5_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
	fn clear_transact_status() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 5_132_000 picoseconds.
		Weight::from_parts(5_280_000, 0)
	}
	fn set_topic() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 5_095_000 picoseconds.
		Weight::from_parts(5_215_000, 0)
	}
	fn clear_topic() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 5_099_000 picoseconds.
		Weight::from_parts(5_214_000, 0)
	}
	/// Storage: Benchmark Override (r:0 w:0)
	/// Proof Skipped: Benchmark Override (max_values: None, max_size: None, mode: Measured)
	fn exchange_asset() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 500_000_000_000 picoseconds.
		Weight::from_parts(500_000_000_000, 0)
	}
	/// Storage: Benchmark Override (r:0 w:0)
	/// Proof Skipped: Benchmark Override (max_values: None, max_size: None, mode: Measured)
	fn universal_origin() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 500_000_000_000 picoseconds.
		Weight::from_parts(500_000_000_000, 0)
	}
	fn set_fees_mode() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 5_066_000 picoseconds.
		Weight::from_parts(5_215_000, 0)
	}
	/// Storage: Benchmark Override (r:0 w:0)
	/// Proof Skipped: Benchmark Override (max_values: None, max_size: None, mode: Measured)
	fn lock_asset() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 500_000_000_000 picoseconds.
		Weight::from_parts(500_000_000_000, 0)
	}
	/// Storage: Benchmark Override (r:0 w:0)
	/// Proof Skipped: Benchmark Override (max_values: None, max_size: None, mode: Measured)
	fn unlock_asset() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 500_000_000_000 picoseconds.
		Weight::from_parts(500_000_000_000, 0)
	}
	/// Storage: Benchmark Override (r:0 w:0)
	/// Proof Skipped: Benchmark Override (max_values: None, max_size: None, mode: Measured)
	fn note_unlockable() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 500_000_000_000 picoseconds.
		Weight::from_parts(500_000_000_000, 0)
	}
	/// Storage: Benchmark Override (r:0 w:0)
	/// Proof Skipped: Benchmark Override (max_values: None, max_size: None, mode: Measured)
	fn request_unlock() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 500_000_000_000 picoseconds.
		Weight::from_parts(500_000_000_000, 0)
	}
	fn unpaid_execution() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 5_249_000 picoseconds.
		Weight::from_parts(5_371_000, 0)
	}
}