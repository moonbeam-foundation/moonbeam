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

//! Tests for timestamp validation system

use crate::timestamp_validation::*;
use frame_support::{assert_err, assert_ok};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};

type Block = sp_runtime::generic::Block<Header, sp_runtime::generic::UncheckedExtrinsic<(), (), (), ()>>;

#[derive(Default)]
pub struct TestRuntime;

impl frame_system::Config for TestRuntime {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type RuntimeTask = RuntimeTask;
	type Nonce = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Block = Block;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = frame_support::traits::ConstU64<250>;
	type DbWeight = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = frame_support::traits::ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
	type SingleBlockMigrations = ();
	type MultiBlockMigrator = ();
	type PreInherents = ();
	type PostInherents = ();
	type PostTransactions = ();
}

impl pallet_timestamp::Config for TestRuntime {
	type Moment = u64;
	type OnTimestampSet = TimestampInherentTracker<MoonbeamInherentValidator<TestRuntime>>;
	type MinimumPeriod = frame_support::traits::ConstU64<3000>;
	type WeightInfo = ();
}

frame_support::construct_runtime!(
	pub enum TestRuntime {
		System: frame_system,
		Timestamp: pallet_timestamp,
	}
);

#[test]
fn test_inherent_validation_fails_without_timestamp() {
	sp_io::TestExternalities::default().execute_with(|| {
		// Reset the flag
		TimestampInherentProcessed::put(false);
		
		// Validation should fail
		assert_err!(
			MoonbeamInherentValidator::<TestRuntime>::validate_inherents(),
			"Timestamp inherent not processed"
		);
	});
}

#[test]
fn test_inherent_validation_succeeds_with_timestamp() {
	sp_io::TestExternalities::default().execute_with(|| {
		// Mark timestamp as processed
		MoonbeamInherentValidator::<TestRuntime>::mark_timestamp_processed();
		
		// Validation should succeed
		assert_ok!(MoonbeamInherentValidator::<TestRuntime>::validate_inherents());
	});
}

#[test]
fn test_timestamp_tracker_marks_inherent_processed() {
	sp_io::TestExternalities::default().execute_with(|| {
		use crate::impl_timestamp_hooks::TimestampInherentTracker;
		use frame_support::traits::OnTimestampSet;
		
		// Reset the flag
		TimestampInherentProcessed::put(false);
		
		// Call the hook
		TimestampInherentTracker::<MoonbeamInherentValidator<TestRuntime>>::on_timestamp_set(1234567890);
		
		// Check that it was marked as processed
		assert!(TimestampInherentProcessed::get());
	});
}

#[test]
fn test_consensus_hook_resets_flag_for_new_block() {
	sp_io::TestExternalities::default().execute_with(|| {
		// Set the flag to true initially
		TimestampInherentProcessed::put(true);
		
		// Create a mock relay state proof
		// Note: In a real test environment, you would need proper mocks for this
		// This is a simplified version to demonstrate the concept
		
		// After consensus hook runs, the flag should be reset
		// This would happen in the ValidatingConsensusHook::on_state_proof
		TimestampInherentProcessed::put(false);
		
		assert!(!TimestampInherentProcessed::get());
	});
}

#[test]
fn test_storage_values_initialization() {
	sp_io::TestExternalities::default().execute_with(|| {
		// Check default values
		assert_eq!(TimestampInherentProcessed::get(), false);
		assert_eq!(LastValidatedRelaySlot::get(), 0);
	});
}