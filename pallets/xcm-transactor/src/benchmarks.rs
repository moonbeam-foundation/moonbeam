use crate::{Call, Config, Pallet};
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite};
use frame_system::RawOrigin;
use sp_std::boxed::Box;
use xcm::latest::prelude::*;

benchmarks! {
	register {
		let account = T::AccountId::default();
		let index = 1u16;
	}: _(RawOrigin::Root, account.clone(), index)
	verify {
		assert_eq!(Pallet::<T>::index_to_account(index), Some(account));
	}

	deregister {
		let account = T::AccountId::default();
		let index = 1u16;
		Pallet::<T>::register(RawOrigin::Root.into(), account, index)?;
	}: _(RawOrigin::Root, index)
	verify {
		assert!(Pallet::<T>::index_to_account(index).is_none());
	}

	set_transact_info {
		let extra_weight = 300000000u64;
		let fee_per_second = 1;
		let max_weight = 20000000000u64;
		let location = MultiLocation::parent();
	}: _(RawOrigin::Root, Box::new(xcm::VersionedMultiLocation::V1(location.clone())), extra_weight, fee_per_second, max_weight)
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
		Pallet::<T>::set_transact_info(RawOrigin::Root.into(), Box::new(xcm::VersionedMultiLocation::V1(location.clone())), extra_weight, fee_per_second, max_weight)?;

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
