#![cfg(feature = "runtime-benchmarks")]

use frame_benchmarking::{benchmarks, impl_benchmark_test_suite};
use frame_system::RawOrigin;
use crate::{Pallet, Call, Config};

benchmarks! {
	register_asset {
        // does not really matter what we register
		let asset_type = T::AssetType::default();
        let metadata = T::AssetRegistrarMetadata::default();
        let amount = 0u32.into();
        let asset_id: T::AssetId = asset_type.clone().into();

	}: _(RawOrigin::Root, asset_type.clone(), metadata, amount)
	verify {
		assert_eq!(Pallet::<T>::asset_id_type(asset_id), Some(asset_type));
	}

    set_asset_units_per_second {
        // does not really matter what we register
		let asset_type = T::AssetType::default();
        let metadata = T::AssetRegistrarMetadata::default();
        let amount = 0u32.into();
        let asset_id: T::AssetId = asset_type.clone().into();
        Pallet::<T>::register_asset(RawOrigin::Root.into(), asset_type.clone(), metadata, amount)?;

	}: _(RawOrigin::Root, asset_id, 1)
	verify {
		assert_eq!(Pallet::<T>::asset_id_units_per_second(asset_id), Some(1));
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