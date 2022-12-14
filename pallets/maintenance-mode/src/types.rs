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

// This file contains the ExecutiveHooks type which is intended to be used
// with frame_executive::Executive. This instructs which pallets execute
// hooks in each of the normal and maintenance modes.
use super::*;
use frame_support::{
	traits::{OffchainWorker, OnFinalize, OnIdle, OnInitialize, OnRuntimeUpgrade},
	weights::Weight,
};
use sp_std::marker::PhantomData;
#[cfg(feature = "try-runtime")]
use sp_std::vec::Vec;

pub struct ExecutiveHooks<T>(PhantomData<T>);
type BlockNumberOf<T> = <T as frame_system::Config>::BlockNumber;

impl<T> OnIdle<BlockNumberOf<T>> for ExecutiveHooks<T>
where
	T: Config,
{
	fn on_idle(n: BlockNumberOf<T>, remaining_weight: Weight) -> Weight {
		if Pallet::<T>::maintenance_mode() {
			T::MaintenanceExecutiveHooks::on_idle(n, remaining_weight)
		} else {
			T::NormalExecutiveHooks::on_idle(n, remaining_weight)
		}
	}
}

impl<T> OnInitialize<BlockNumberOf<T>> for ExecutiveHooks<T>
where
	T: Config,
{
	fn on_initialize(n: BlockNumberOf<T>) -> Weight {
		if Pallet::<T>::maintenance_mode() {
			T::MaintenanceExecutiveHooks::on_initialize(n)
		} else {
			T::NormalExecutiveHooks::on_initialize(n)
		}
	}
}

impl<T> OnFinalize<BlockNumberOf<T>> for ExecutiveHooks<T>
where
	T: Config,
{
	fn on_finalize(n: BlockNumberOf<T>) {
		if Pallet::<T>::maintenance_mode() {
			T::MaintenanceExecutiveHooks::on_finalize(n)
		} else {
			T::NormalExecutiveHooks::on_finalize(n)
		}
	}
}

impl<T> OffchainWorker<BlockNumberOf<T>> for ExecutiveHooks<T>
where
	T: Config,
{
	fn offchain_worker(n: BlockNumberOf<T>) {
		if Pallet::<T>::maintenance_mode() {
			T::MaintenanceExecutiveHooks::offchain_worker(n)
		} else {
			T::NormalExecutiveHooks::offchain_worker(n)
		}
	}
}

impl<T> OnRuntimeUpgrade for ExecutiveHooks<T>
where
	T: Config,
{
	fn on_runtime_upgrade() -> Weight {
		if Pallet::<T>::maintenance_mode() {
			T::MaintenanceExecutiveHooks::on_runtime_upgrade()
		} else {
			T::NormalExecutiveHooks::on_runtime_upgrade()
		}
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
		if Pallet::<T>::maintenance_mode() {
			T::MaintenanceExecutiveHooks::pre_upgrade()
		} else {
			T::NormalExecutiveHooks::pre_upgrade()
		}
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(state: Vec<u8>) -> Result<(), &'static str> {
		if Pallet::<T>::maintenance_mode() {
			T::MaintenanceExecutiveHooks::post_upgrade(state)
		} else {
			T::NormalExecutiveHooks::post_upgrade(state)
		}
	}
}

#[cfg(feature = "try-runtime")]
impl<T: frame_system::Config> frame_support::traits::TryState<BlockNumberOf<T>>
	for ExecutiveHooks<T>
{
	fn try_state(
		_: BlockNumberOf<T>,
		_: frame_support::traits::TryStateSelect,
	) -> Result<(), &'static str> {
		Ok(())
	}
}
