// Copyright 2024 Moonbeam foundation
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

//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 32.0.0
//! DATE: 2024-04-27 (Y/M/D)
//! HOSTNAME: `ip-10-0-0-176`, CPU: `Intel(R) Xeon(R) Platinum 8375C CPU @ 2.90GHz`
//!
//! DATABASE: `RocksDb`, RUNTIME: `Moonbeam`
//! BLOCK-NUM: `BlockId::Number(5962022)`
//! SKIP-WRITE: `false`, SKIP-READ: `false`, WARMUPS: `1`
//! STATE-VERSION: `V0`, STATE-CACHE-SIZE: ``
//! WEIGHT-PATH: `/home/ubuntu/projects/moonbeam/weights-rocksdb-moonbeam.rs`
//! METRIC: `Average`, WEIGHT-MUL: `1.1`, WEIGHT-ADD: `0`

// Executed Command:
//   /home/ubuntu/projects/moonbeam/target/release/moonbeam
//   benchmark
//   storage
//   --db=rocksdb
//   --state-version=0
//   --mul=1.1
//   --weight-path
//   /home/ubuntu/projects/moonbeam/weights-rocksdb-moonbeam.rs
//   --chain
//   moonbeam
//   --base-path
//   /var/lib/rocksdb-moonbeam-data
//   --keys-limit
//   10000000
//   --random-seed
//   1024

/// Storage DB weights for the `Moonbeam` runtime and `RocksDb`.
pub mod constants {
	use frame_support::weights::{constants, RuntimeDbWeight};
	use sp_core::parameter_types;

	parameter_types! {
		/// By default, Substrate uses `RocksDB`, so this will be the weight used throughout
		/// the runtime.
		pub const RocksDbWeight: RuntimeDbWeight = RuntimeDbWeight {
			// Time to read one storage item.
			// Calculated by multiplying the *Average* of all values with `1.1` and adding `0`.
			//
			// Stats nanoseconds:
			//   Min, Max: 2_300, 2_841_169
			//   Average:  37_947
			//   Median:   38_669
			//   Std-Dev:  7331.86
			//
			// Percentiles nanoseconds:
			//   99th: 55_974
			//   95th: 49_824
			//   75th: 42_570
			read: 41_742 * constants::WEIGHT_REF_TIME_PER_NANOS,

			// Time to write one storage item.
			// Calculated by multiplying the *Average* of all values with `1.1` and adding `0`.
			//
			// Stats nanoseconds:
			//   Min, Max: 18_981, 16_772_373
			//   Average:  73_893
			//   Median:   72_807
			//   Std-Dev:  24543.58
			//
			// Percentiles nanoseconds:
			//   99th: 97_152
			//   95th: 85_751
			//   75th: 77_392
			write: 81_283 * constants::WEIGHT_REF_TIME_PER_NANOS,
		};
	}

	#[cfg(test)]
	mod test_db_weights {
		use super::constants::RocksDbWeight as W;
		use frame_support::weights::constants;

		/// Checks that all weights exist and have sane values.
		// NOTE: If this test fails but you are sure that the generated values are fine,
		// you can delete it.
		#[test]
		fn bound() {
			// At least 1 µs.
			assert!(
				W::get().reads(1).ref_time() >= constants::WEIGHT_REF_TIME_PER_MICROS,
				"Read weight should be at least 1 µs."
			);
			assert!(
				W::get().writes(1).ref_time() >= constants::WEIGHT_REF_TIME_PER_MICROS,
				"Write weight should be at least 1 µs."
			);
			// At most 1 ms.
			assert!(
				W::get().reads(1).ref_time() <= constants::WEIGHT_REF_TIME_PER_MILLIS,
				"Read weight should be at most 1 ms."
			);
			assert!(
				W::get().writes(1).ref_time() <= constants::WEIGHT_REF_TIME_PER_MILLIS,
				"Write weight should be at most 1 ms."
			);
		}
	}
}
