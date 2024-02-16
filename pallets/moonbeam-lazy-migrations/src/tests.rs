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

//! Unit testing
use {
	crate::mock::{ExtBuilder, LazyMigrations},
	frame_support::assert_ok,
};

/// TODO(rodrigo): This test should be removed once LocalAssets pallet storage is removed
#[test]
fn test_call_clear_local_assets_storage() {
	let mut context = ExtBuilder::default().build();

	let pallet_prefix = sp_io::hashing::twox_128("LocalAssets".as_bytes());
	let total_storage_entries: u8 = 5;

	let gen_dummy_entry = |dummy: u8| -> [u8; 32] {
		[pallet_prefix, sp_io::hashing::twox_128(&[dummy])]
			.concat()
			.try_into()
			.unwrap()
	};

	context.execute_with(|| {
		for i in 0u8..total_storage_entries {
			let dummy_data = gen_dummy_entry(i);
			sp_io::storage::set(&dummy_data, &dummy_data);
			dbg!(sp_io::storage::exists(&dummy_data));
		}
	});

	// Commit changes
	let _ = context.commit_all();

	// Next block
	context.execute_with(|| {
		// Check that the storage entries exist before attempting to remove it
		for i in 0u8..total_storage_entries {
			let dummy_data = gen_dummy_entry(i);
			assert!(sp_io::storage::exists(&dummy_data));
		}
		// Clear all storage entries
		assert_ok!(LazyMigrations::clear_local_assets_storage(
			crate::mock::RuntimeOrigin::signed(1),
			total_storage_entries.into()
		));
		// Check that all storage entries got deleted
		for i in 0u8..total_storage_entries {
			let dummy_data = gen_dummy_entry(i);
			assert!(!sp_io::storage::exists(&dummy_data));
		}
	});

	// Commit changes
	let _ = context.commit_all();

	// Next block
	context.execute_with(|| {
		// No more storage entries to be removed (expect failure)
		assert!(LazyMigrations::clear_local_assets_storage(
			crate::mock::RuntimeOrigin::signed(1),
			1
		)
		.is_err())
	});
}
