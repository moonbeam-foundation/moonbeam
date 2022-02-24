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

use crate::{Call, Config, Pallet};
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite};
use frame_system::RawOrigin;
use sp_std::boxed::Box;
use xcm::latest::prelude::*;

benchmarks! {
	register {
		let user: T::AccountId  = account("account id", 0u32, 0u32);

		let index = 1u16;
	}: _(RawOrigin::Root, user.clone(), index)
	verify {
		assert_eq!(Pallet::<T>::index_to_account(index), Some(user));
	}

	deregister {
		let user: T::AccountId  = account("account id", 0u32, 0u32);
		let index = 1u16;
		Pallet::<T>::register(RawOrigin::Root.into(), user, index).unwrap();
	}: _(RawOrigin::Root, index)
	verify {
		assert!(Pallet::<T>::index_to_account(index).is_none());
	}

	set_transact_info {
		let extra_weight = 300000000u64;
		let fee_per_second = 1;
		let max_weight = 20000000000u64;
		let location = MultiLocation::parent();
	}: _(
		RawOrigin::Root,
		Box::new(xcm::VersionedMultiLocation::V1(location.clone())),
		extra_weight,
		fee_per_second,
		max_weight
	)
	verify {
		assert_eq!(Pallet::<T>::transact_info(&location), Some(crate::RemoteTransactInfoWithMaxWeight {
			transact_extra_weight: extra_weight,
			fee_per_second,
			max_weight
		}));
	}

	remove_transact_info {
		let extra_weight = 300000000u64;
		let fee_per_second = 1;
		let max_weight = 20000000000u64;
		let location = MultiLocation::parent();
		Pallet::<T>::set_transact_info(
			RawOrigin::Root.into(),
			Box::new(xcm::VersionedMultiLocation::V1(location.clone())),
			extra_weight,
			fee_per_second,
			max_weight
		).unwrap();
	}: _(RawOrigin::Root, Box::new(xcm::VersionedMultiLocation::V1(location.clone())))
	verify {
		assert!(Pallet::<T>::transact_info(&location).is_none());
	}
}

#[cfg(test)]
mod tests {
	use crate::mock::Test;
	use sp_io::TestExternalities;

	pub fn new_test_ext() -> TestExternalities {
		let t = frame_system::GenesisConfig::default()
			.build_storage::<Test>()
			.unwrap();
		TestExternalities::new(t)
	}
}

impl_benchmark_test_suite!(
	Pallet,
	crate::benchmarks::tests::new_test_ext(),
	crate::mock::Test
);
