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

use crate::Config;
use frame_support::{
	pallet_prelude::PhantomData,
	storage::migration::*,
	traits::{Get, OnRuntimeUpgrade},
	weights::Weight,
};

/// Removes `NotFirstBlock` storage item from storage and clears it
pub struct RemoveNotFirstBlock<T>(PhantomData<T>);
impl<T: Config> OnRuntimeUpgrade for RemoveNotFirstBlock<T> {
	fn on_runtime_upgrade() -> Weight {
		log::info!(target: "RemoveNotFirstBlock", "running migration");

		let pallet_prefix: &[u8] = b"Randomness";
		let storage_item_prefix: &[u8] = b"NotFirstBlock";

		let _ = clear_storage_prefix(
			pallet_prefix,
			storage_item_prefix,
			&[],
			Some(u32::MAX),
			None,
		);
		T::DbWeight::get().reads_writes(1, 1)
	}
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<(), &'static str> {
		use frame_support::traits::OnRuntimeUpgradeHelpersExt;

		Self::set_temp_storage(
			get_storage_value::<()>(b"Randomness", b"NotFirstBlock", &[]),
			"NotFirstBlock",
		);
		Ok(())
	}
	#[cfg(feature = "try-runtime")]
	fn post_upgrade() -> Result<(), &'static str> {
		use frame_support::traits::OnRuntimeUpgradeHelpersExt;
		assert!(get_storage_value::<()>(b"Randomness", b"NotFirstBlock", &[]).is_none());
		assert_eq!(Self::get_temp_storage("NotFirstBlock"), Some(()));
		Ok(())
	}
}

#[test]
fn remove_not_first_block_migration_works() {
	use crate::mock::*;
	ExtBuilder::default().build().execute_with(|| {
		put_storage_value(b"Randomness", b"NotFirstBlock", &[], ());
		assert_eq!(
			get_storage_value::<()>(b"Randomness", b"NotFirstBlock", &[]),
			Some(())
		);
		RemoveNotFirstBlock::<Test>::on_runtime_upgrade();
		assert!(get_storage_value::<()>(b"Randomness", b"NotFirstBlock", &[]).is_none());
	});
}
