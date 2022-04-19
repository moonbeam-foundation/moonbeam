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

//! Migrations

#[cfg(feature = "try-runtime")]
use frame_support::traits::OnRuntimeUpgradeHelpersExt;

use crate::{BalanceOf, Config};
use frame_support::{
	pallet_prelude::PhantomData,
	traits::{Get, OnRuntimeUpgrade},
	weights::Weight,
};

/// Initialize the storage value `MinOrbiterDeposit`.
/// This migration is only necessary if you add this pallet to your runtime after the genesis block,
/// if this is your case, you must integrate this migration only at the same time as adding this
/// pallet.
pub struct InitMinOrbiterDeposit<T, Balance, MinOrbiterDeposit: Get<Balance>>(
	PhantomData<(T, Balance, MinOrbiterDeposit)>,
);
impl<T: Config, MinOrbiterDeposit: Get<BalanceOf<T>>> OnRuntimeUpgrade
	for InitMinOrbiterDeposit<T, BalanceOf<T>, MinOrbiterDeposit>
{
	fn on_runtime_upgrade() -> Weight {
		log::info!(
			target: "InitMinOrbiterDeposit",
			"running migration to set minimal orbiter deposit"
		);
		crate::MinOrbiterDeposit::<T>::put(MinOrbiterDeposit::get());
		T::DbWeight::get().write
	}
	#[cfg(feature = "try-runtime")]
	fn post_upgrade() -> Result<(), &'static str> {
		assert_eq!(
			crate::MinOrbiterDeposit::<T>::get(),
			MinOrbiterDeposit::get()
		);
	}
}
