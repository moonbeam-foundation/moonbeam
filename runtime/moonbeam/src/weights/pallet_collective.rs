// Copyright 2019-2025 PureStake Inc.
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
//! Autogenerated weights for `pallet_collective`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-11-01, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `ip-10-0-0-176`, CPU: `Intel(R) Xeon(R) Platinum 8375C CPU @ 2.90GHz`
//! EXECUTION: None, WASM-EXECUTION: Compiled, CHAIN: Some("moonbase-dev"), DB CACHE: 1024

// Executed Command:
// ./target/release/moonbeam
// benchmark
// pallet
// --chain=moonbase-dev
// --steps=50
// --repeat=20
// --pallet=pallet_collective
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

/// Weight functions for `pallet_collective`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_collective::WeightInfo for WeightInfo<T> {
	/// Storage: CouncilCollective Members (r:1 w:1)
	/// Proof Skipped: CouncilCollective Members (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: CouncilCollective Proposals (r:1 w:0)
	/// Proof Skipped: CouncilCollective Proposals (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: CouncilCollective Voting (r:100 w:100)
	/// Proof Skipped: CouncilCollective Voting (max_values: None, max_size: None, mode: Measured)
	/// Storage: CouncilCollective Prime (r:0 w:1)
	/// Proof Skipped: CouncilCollective Prime (max_values: Some(1), max_size: None, mode: Measured)
	/// The range of component `m` is `[0, 100]`.
	/// The range of component `n` is `[0, 100]`.
	/// The range of component `p` is `[0, 100]`.
	fn set_members(m: u32, _n: u32, p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0 + m * (2021 ±0) + p * (2026 ±0)`
		//  Estimated: `12238 + m * (1231 ±14) + p * (3660 ±14)`
		// Minimum execution time: 13_785_000 picoseconds.
		Weight::from_parts(14_085_000, 0)
			.saturating_add(Weight::from_parts(0, 12238))
			// Standard Error: 34_315
			.saturating_add(Weight::from_parts(2_414_144, 0).saturating_mul(m.into()))
			// Standard Error: 34_315
			.saturating_add(Weight::from_parts(5_345_562, 0).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(p.into())))
			.saturating_add(T::DbWeight::get().writes(2))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(p.into())))
			.saturating_add(Weight::from_parts(0, 1231).saturating_mul(m.into()))
			.saturating_add(Weight::from_parts(0, 3660).saturating_mul(p.into()))
	}
	/// Storage: CouncilCollective Members (r:1 w:0)
	/// Proof Skipped: CouncilCollective Members (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: MaintenanceMode MaintenanceMode (r:1 w:0)
	/// Proof Skipped: MaintenanceMode MaintenanceMode (max_values: Some(1), max_size: None, mode: Measured)
	/// The range of component `b` is `[2, 1024]`.
	/// The range of component `m` is `[1, 100]`.
	fn execute(b: u32, m: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `149 + m * (20 ±0)`
		//  Estimated: `1634 + m * (20 ±0)`
		// Minimum execution time: 12_726_000 picoseconds.
		Weight::from_parts(12_294_605, 0)
			.saturating_add(Weight::from_parts(0, 1634))
			// Standard Error: 18
			.saturating_add(Weight::from_parts(910, 0).saturating_mul(b.into()))
			// Standard Error: 192
			.saturating_add(Weight::from_parts(13_235, 0).saturating_mul(m.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(Weight::from_parts(0, 20).saturating_mul(m.into()))
	}
	/// Storage: CouncilCollective Members (r:1 w:0)
	/// Proof Skipped: CouncilCollective Members (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: CouncilCollective ProposalOf (r:1 w:0)
	/// Proof Skipped: CouncilCollective ProposalOf (max_values: None, max_size: None, mode: Measured)
	/// Storage: MaintenanceMode MaintenanceMode (r:1 w:0)
	/// Proof Skipped: MaintenanceMode MaintenanceMode (max_values: Some(1), max_size: None, mode: Measured)
	/// The range of component `b` is `[2, 1024]`.
	/// The range of component `m` is `[1, 100]`.
	fn propose_execute(b: u32, m: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `149 + m * (20 ±0)`
		//  Estimated: `3614 + m * (20 ±0)`
		// Minimum execution time: 14_722_000 picoseconds.
		Weight::from_parts(14_487_427, 0)
			.saturating_add(Weight::from_parts(0, 3614))
			// Standard Error: 20
			.saturating_add(Weight::from_parts(944, 0).saturating_mul(b.into()))
			// Standard Error: 215
			.saturating_add(Weight::from_parts(25_619, 0).saturating_mul(m.into()))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(Weight::from_parts(0, 20).saturating_mul(m.into()))
	}
	/// Storage: CouncilCollective Members (r:1 w:0)
	/// Proof Skipped: CouncilCollective Members (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: CouncilCollective ProposalOf (r:1 w:1)
	/// Proof Skipped: CouncilCollective ProposalOf (max_values: None, max_size: None, mode: Measured)
	/// Storage: CouncilCollective Proposals (r:1 w:1)
	/// Proof Skipped: CouncilCollective Proposals (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: CouncilCollective ProposalCount (r:1 w:1)
	/// Proof Skipped: CouncilCollective ProposalCount (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: CouncilCollective Voting (r:0 w:1)
	/// Proof Skipped: CouncilCollective Voting (max_values: None, max_size: None, mode: Measured)
	/// The range of component `b` is `[2, 1024]`.
	/// The range of component `m` is `[2, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn propose_proposed(b: u32, m: u32, p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `395 + m * (20 ±0) + p * (36 ±0)`
		//  Estimated: `3789 + m * (21 ±0) + p * (36 ±0)`
		// Minimum execution time: 17_107_000 picoseconds.
		Weight::from_parts(15_172_014, 0)
			.saturating_add(Weight::from_parts(0, 3789))
			// Standard Error: 104
			.saturating_add(Weight::from_parts(3_191, 0).saturating_mul(b.into()))
			// Standard Error: 1_090
			.saturating_add(Weight::from_parts(31_145, 0).saturating_mul(m.into()))
			// Standard Error: 1_077
			.saturating_add(Weight::from_parts(131_855, 0).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(4))
			.saturating_add(Weight::from_parts(0, 21).saturating_mul(m.into()))
			.saturating_add(Weight::from_parts(0, 36).saturating_mul(p.into()))
	}
	/// Storage: CouncilCollective Members (r:1 w:0)
	/// Proof Skipped: CouncilCollective Members (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: CouncilCollective Voting (r:1 w:1)
	/// Proof Skipped: CouncilCollective Voting (max_values: None, max_size: None, mode: Measured)
	/// The range of component `m` is `[5, 100]`.
	fn vote(m: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `870 + m * (40 ±0)`
		//  Estimated: `4334 + m * (40 ±0)`
		// Minimum execution time: 21_423_000 picoseconds.
		Weight::from_parts(22_311_906, 0)
			.saturating_add(Weight::from_parts(0, 4334))
			// Standard Error: 415
			.saturating_add(Weight::from_parts(32_990, 0).saturating_mul(m.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
			.saturating_add(Weight::from_parts(0, 40).saturating_mul(m.into()))
	}
	/// Storage: CouncilCollective Voting (r:1 w:1)
	/// Proof Skipped: CouncilCollective Voting (max_values: None, max_size: None, mode: Measured)
	/// Storage: CouncilCollective Members (r:1 w:0)
	/// Proof Skipped: CouncilCollective Members (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: CouncilCollective Proposals (r:1 w:1)
	/// Proof Skipped: CouncilCollective Proposals (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: CouncilCollective ProposalOf (r:0 w:1)
	/// Proof Skipped: CouncilCollective ProposalOf (max_values: None, max_size: None, mode: Measured)
	/// The range of component `m` is `[4, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn close_early_disapproved(m: u32, p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `447 + m * (40 ±0) + p * (36 ±0)`
		//  Estimated: `3892 + m * (41 ±0) + p * (36 ±0)`
		// Minimum execution time: 20_010_000 picoseconds.
		Weight::from_parts(20_008_777, 0)
			.saturating_add(Weight::from_parts(0, 3892))
			// Standard Error: 1_391
			.saturating_add(Weight::from_parts(39_960, 0).saturating_mul(m.into()))
			// Standard Error: 1_356
			.saturating_add(Weight::from_parts(126_447, 0).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(3))
			.saturating_add(Weight::from_parts(0, 41).saturating_mul(m.into()))
			.saturating_add(Weight::from_parts(0, 36).saturating_mul(p.into()))
	}
	/// Storage: CouncilCollective Voting (r:1 w:1)
	/// Proof Skipped: CouncilCollective Voting (max_values: None, max_size: None, mode: Measured)
	/// Storage: CouncilCollective Members (r:1 w:0)
	/// Proof Skipped: CouncilCollective Members (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: CouncilCollective ProposalOf (r:1 w:1)
	/// Proof Skipped: CouncilCollective ProposalOf (max_values: None, max_size: None, mode: Measured)
	/// Storage: MaintenanceMode MaintenanceMode (r:1 w:0)
	/// Proof Skipped: MaintenanceMode MaintenanceMode (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: CouncilCollective Proposals (r:1 w:1)
	/// Proof Skipped: CouncilCollective Proposals (max_values: Some(1), max_size: None, mode: Measured)
	/// The range of component `b` is `[2, 1024]`.
	/// The range of component `m` is `[4, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn close_early_approved(b: u32, m: u32, p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `791 + b * (1 ±0) + m * (40 ±0) + p * (40 ±0)`
		//  Estimated: `4108 + b * (1 ±0) + m * (42 ±0) + p * (40 ±0)`
		// Minimum execution time: 30_647_000 picoseconds.
		Weight::from_parts(30_084_699, 0)
			.saturating_add(Weight::from_parts(0, 4108))
			// Standard Error: 123
			.saturating_add(Weight::from_parts(2_876, 0).saturating_mul(b.into()))
			// Standard Error: 1_307
			.saturating_add(Weight::from_parts(31_661, 0).saturating_mul(m.into()))
			// Standard Error: 1_274
			.saturating_add(Weight::from_parts(154_567, 0).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(3))
			.saturating_add(Weight::from_parts(0, 1).saturating_mul(b.into()))
			.saturating_add(Weight::from_parts(0, 42).saturating_mul(m.into()))
			.saturating_add(Weight::from_parts(0, 40).saturating_mul(p.into()))
	}
	/// Storage: CouncilCollective Voting (r:1 w:1)
	/// Proof Skipped: CouncilCollective Voting (max_values: None, max_size: None, mode: Measured)
	/// Storage: CouncilCollective Members (r:1 w:0)
	/// Proof Skipped: CouncilCollective Members (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: CouncilCollective Prime (r:1 w:0)
	/// Proof Skipped: CouncilCollective Prime (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: CouncilCollective Proposals (r:1 w:1)
	/// Proof Skipped: CouncilCollective Proposals (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: CouncilCollective ProposalOf (r:0 w:1)
	/// Proof Skipped: CouncilCollective ProposalOf (max_values: None, max_size: None, mode: Measured)
	/// The range of component `m` is `[4, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn close_disapproved(m: u32, p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `516 + m * (30 ±0) + p * (36 ±0)`
		//  Estimated: `3958 + m * (31 ±0) + p * (36 ±0)`
		// Minimum execution time: 20_735_000 picoseconds.
		Weight::from_parts(22_649_363, 0)
			.saturating_add(Weight::from_parts(0, 3958))
			// Standard Error: 1_082
			.saturating_add(Weight::from_parts(32_331, 0).saturating_mul(m.into()))
			// Standard Error: 1_055
			.saturating_add(Weight::from_parts(122_034, 0).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(3))
			.saturating_add(Weight::from_parts(0, 31).saturating_mul(m.into()))
			.saturating_add(Weight::from_parts(0, 36).saturating_mul(p.into()))
	}
	/// Storage: CouncilCollective Voting (r:1 w:1)
	/// Proof Skipped: CouncilCollective Voting (max_values: None, max_size: None, mode: Measured)
	/// Storage: CouncilCollective Members (r:1 w:0)
	/// Proof Skipped: CouncilCollective Members (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: CouncilCollective Prime (r:1 w:0)
	/// Proof Skipped: CouncilCollective Prime (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: CouncilCollective ProposalOf (r:1 w:1)
	/// Proof Skipped: CouncilCollective ProposalOf (max_values: None, max_size: None, mode: Measured)
	/// Storage: MaintenanceMode MaintenanceMode (r:1 w:0)
	/// Proof Skipped: MaintenanceMode MaintenanceMode (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: CouncilCollective Proposals (r:1 w:1)
	/// Proof Skipped: CouncilCollective Proposals (max_values: Some(1), max_size: None, mode: Measured)
	/// The range of component `b` is `[2, 1024]`.
	/// The range of component `m` is `[4, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn close_approved(b: u32, m: u32, p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `811 + b * (1 ±0) + m * (40 ±0) + p * (40 ±0)`
		//  Estimated: `4128 + b * (1 ±0) + m * (42 ±0) + p * (40 ±0)`
		// Minimum execution time: 32_927_000 picoseconds.
		Weight::from_parts(32_086_367, 0)
			.saturating_add(Weight::from_parts(0, 4128))
			// Standard Error: 122
			.saturating_add(Weight::from_parts(2_962, 0).saturating_mul(b.into()))
			// Standard Error: 1_299
			.saturating_add(Weight::from_parts(32_167, 0).saturating_mul(m.into()))
			// Standard Error: 1_266
			.saturating_add(Weight::from_parts(154_131, 0).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(6))
			.saturating_add(T::DbWeight::get().writes(3))
			.saturating_add(Weight::from_parts(0, 1).saturating_mul(b.into()))
			.saturating_add(Weight::from_parts(0, 42).saturating_mul(m.into()))
			.saturating_add(Weight::from_parts(0, 40).saturating_mul(p.into()))
	}
	/// Storage: CouncilCollective Proposals (r:1 w:1)
	/// Proof Skipped: CouncilCollective Proposals (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: CouncilCollective Voting (r:0 w:1)
	/// Proof Skipped: CouncilCollective Voting (max_values: None, max_size: None, mode: Measured)
	/// Storage: CouncilCollective ProposalOf (r:0 w:1)
	/// Proof Skipped: CouncilCollective ProposalOf (max_values: None, max_size: None, mode: Measured)
	/// The range of component `p` is `[1, 100]`.
	fn disapprove_proposal(p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `264 + p * (32 ±0)`
		//  Estimated: `1749 + p * (32 ±0)`
		// Minimum execution time: 10_334_000 picoseconds.
		Weight::from_parts(11_413_201, 0)
			.saturating_add(Weight::from_parts(0, 1749))
			// Standard Error: 1_033
			.saturating_add(Weight::from_parts(95_458, 0).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(3))
			.saturating_add(Weight::from_parts(0, 32).saturating_mul(p.into()))
	}
}
