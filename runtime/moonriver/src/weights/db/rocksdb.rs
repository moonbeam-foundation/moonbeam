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

//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 48.0.0
//! DATE: 2025-09-22 (Y/M/D)
//! HOSTNAME: `ip-10-0-0-176`, CPU: `Intel(R) Xeon(R) Platinum 8375C CPU @ 2.90GHz`
//!
//! DATABASE: `RocksDb`, RUNTIME: `Moonbeam`
//! BLOCK-NUM: `BlockId::Number(12630729)`
//! SKIP-WRITE: `false`, SKIP-READ: `false`, WARMUPS: `1`
//! STATE-VERSION: `V1`, STATE-CACHE-SIZE: ``
//! WEIGHT-PATH: `./benchmarks/storage/20250922-082315/disk-weights-rocksdb-moonbeam.rs`
//! METRIC: `Average`, WEIGHT-MUL: `1.1`, WEIGHT-ADD: `0`

// Executed Command:
//   ./moonbeam
//   benchmark
//   storage
//   --db=rocksdb
//   --state-version=1
//   --mul=1.1
//   --weight-path
//   ./benchmarks/storage/20250922-082315/disk-weights-rocksdb-moonbeam.rs
//   --chain
//   moonbeam
//   --base-path
//   /mnt/disk3-6000-256/rocksdb-moonbeam-data
//   --keys-limit
//   50000000
//   --random-seed
//   1024

/// Storage DB weights for the `Moonbeam` runtime and `RocksDb`.
pub mod constants {
	use frame_support::weights::constants;
	use sp_core::parameter_types;
	use sp_weights::RuntimeDbWeight;

	parameter_types! {
		/// By default, Substrate uses `RocksDB`, so this will be the weight used throughout
		/// the runtime.
		pub const RocksDbWeight: RuntimeDbWeight = RuntimeDbWeight {
			// Time to read one storage item.
			// Calculated by multiplying the *Average* of all values with `1.1` and adding `0`.
			//
			// Stats nanoseconds:
			//   Min, Max: 1_774, 4_131_758
			//   Average:  53_833
			//   Median:   47_991
			//   Std-Dev:  44586.1
			//
			// Percentiles nanoseconds:
			//   99th: 236_090
			//   95th: 67_897
			//   75th: 54_501
			read: 59_217 * constants::WEIGHT_REF_TIME_PER_NANOS,

			// Time to write one storage item.
			// Calculated by multiplying the *Average* of all values with `1.1` and adding `0`.
			//
			// Stats nanoseconds:
			//   Min, Max: 10_807, 13_782_646
			//   Average:  87_559
			//   Median:   73_293
			//   Std-Dev:  191482.81
			//
			// Percentiles nanoseconds:
			//   99th: 212_681
			//   95th: 111_877
			//   75th: 82_079
			write: 96_315 * constants::WEIGHT_REF_TIME_PER_NANOS,
		};
	}

	#[cfg(test)]
	mod test_db_weights {
		use super::constants::RocksDbWeight as W;
		use sp_weights::constants;

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
