// Copyright 2019-2021 PureStake Inc.
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

//! Tests for implementation of pallet-transaction-fee

use pallet_transaction_payment::Multiplier;

#[cfg(test)]
mod multiplier_tests {
	use super::*;
	use frame_support::{
		parameter_types,
		weights::{DispatchClass, Weight},
	};
	use sp_core::H256;
	use sp_runtime::{
		testing::Header,
		traits::{BlakeTwo256, Convert, IdentityLookup, One},
		Perbill,
	};

	type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
	type Block = frame_system::mocking::MockBlock<Runtime>;

	frame_support::construct_runtime!(
		pub enum Runtime where
			Block = Block,
			NodeBlock = Block,
			UncheckedExtrinsic = UncheckedExtrinsic,
		{
			System: frame_system::{Pallet, Call, Config, Storage, Event<T>}
		}
	);

	parameter_types! {
		pub const BlockHashCount: u64 = 250;
		pub const AvailableBlockRatio: Perbill = Perbill::one();
		pub BlockLength: frame_system::limits::BlockLength =
			frame_system::limits::BlockLength::max(2 * 1024);
		pub BlockWeights: frame_system::limits::BlockWeights =
			frame_system::limits::BlockWeights::simple_max(1024);
	}

	impl frame_system::Config for Runtime {
		type BaseCallFilter = ();
		type BlockWeights = BlockWeights;
		type BlockLength = ();
		type DbWeight = ();
		type Origin = Origin;
		type Index = u64;
		type BlockNumber = u64;
		type Call = Call;
		type Hash = H256;
		type Hashing = BlakeTwo256;
		type AccountId = u64;
		type Lookup = IdentityLookup<Self::AccountId>;
		type Header = Header;
		type Event = Event;
		type BlockHashCount = BlockHashCount;
		type Version = ();
		type PalletInfo = PalletInfo;
		type AccountData = ();
		type OnNewAccount = ();
		type OnKilledAccount = ();
		type SystemWeightInfo = ();
		type SS58Prefix = ();
		type OnSetCode = ();
	}

	fn run_with_system_weight<F>(w: Weight, mut assertions: F)
	where
		F: FnMut() -> (),
	{
		let mut t: sp_io::TestExternalities = frame_system::GenesisConfig::default()
			.build_storage::<Runtime>()
			.unwrap()
			.into();
		t.execute_with(|| {
			System::set_block_consumed_resources(w, 0);
			assertions()
		});
	}

	#[test]
	fn multiplier_can_grow_from_zero() {
		let minimum_multiplier = moonshadow_runtime::MinimumMultiplier::get();
		let target = moonshadow_runtime::TargetBlockFullness::get()
			* BlockWeights::get()
				.get(DispatchClass::Normal)
				.max_total
				.unwrap();
		// if the min is too small, then this will not change, and we are doomed forever.
		// the weight is 1/100th bigger than target.
		run_with_system_weight(target * 101 / 100, || {
			let next = moonshadow_runtime::SlowAdjustingFeeUpdate::<Runtime>::convert(minimum_multiplier);
			assert!(
				next > minimum_multiplier,
				"{:?} !>= {:?}",
				next,
				minimum_multiplier
			);
		})
	}

	#[test]
	#[ignore] // test runs for a very long time
	fn multiplier_growth_simulator() {
		// assume the multiplier is initially set to its minimum. We update it with values twice the
		//target (target is 25%, thus 50%) and we see at which point it reaches 1.
		let mut multiplier = moonshadow_runtime::MinimumMultiplier::get();
		let block_weight = moonshadow_runtime::TargetBlockFullness::get()
			* BlockWeights::get()
				.get(DispatchClass::Normal)
				.max_total
				.unwrap() * 2;
		let mut blocks = 0;
		while multiplier <= Multiplier::one() {
			run_with_system_weight(block_weight, || {
				let next = moonshadow_runtime::SlowAdjustingFeeUpdate::<Runtime>::convert(multiplier);
				// ensure that it is growing as well.
				assert!(next > multiplier, "{:?} !>= {:?}", next, multiplier);
				multiplier = next;
			});
			blocks += 1;
			println!("block = {} multiplier {:?}", blocks, multiplier);
		}
	}
}
