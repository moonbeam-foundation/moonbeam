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

use crate::{democracy_preimages::*, Call, Config, Pallet, Vec};
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::BoundedVec;
use frame_system::RawOrigin;
use pallet_preimage::RequestStatus;
use sp_runtime::traits::Hash;

benchmarks! {
	migrate_democracy_preimage {
		let x in 5..100;

		let caller: T::AccountId = whitelisted_caller();
		let mut data = Vec::with_capacity(x as usize);
		data.resize(x as usize, 1);
		let bounded_data: BoundedVec<_, _> = data.clone().try_into().expect("fits in bound");
		let len = data.len() as u32;
		let hash = <T as frame_system::Config>::Hashing::hash_of(&data);

		DeprecatedDemocracyPreimages::<T>::insert(
			hash,
			PreimageStatus::Available {
				data,
				provider: caller.clone(),
				deposit: 142u32.into(),
				since: 0u32.into(),
				expiry: None,
			},
		);
	}: _(RawOrigin::Signed(caller.clone()), hash, len)
	verify {
		assert!(DeprecatedDemocracyPreimages::<T>::get(hash).is_none());
		assert_eq!(
			StatusFor::<T>::get(hash),
			Some(RequestStatus::Unrequested {
				deposit: (caller, 142u32.into()),
				len
			})
		);
		assert_eq!(PreimageFor::<T>::get((hash, len)), Some(bounded_data));
	}
}

#[cfg(test)]
mod tests {
	use crate::mock::Runtime;
	use sp_io::TestExternalities;

	pub fn new_test_ext() -> TestExternalities {
		let t = frame_system::GenesisConfig::default()
			.build_storage::<Runtime>()
			.unwrap();
		TestExternalities::new(t)
	}
}

impl_benchmark_test_suite!(
	Pallet,
	crate::benchmarks::tests::new_test_ext(),
	crate::mock::Runtime
);
