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

#![cfg(feature = "runtime-benchmarks")]

use crate::{Call, Config, Currency, CurrencyPayment, HrmpOperation, Pallet, TransactWeights};
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite};
use frame_support::weights::Weight;
use frame_system::RawOrigin;
use sp_std::boxed::Box;
use sp_std::vec;
use xcm::latest::prelude::*;

benchmarks! {
	where_clause { where T::Transactor: Default, T::CurrencyId: From<MultiLocation>}
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
		Pallet::<T>::register(RawOrigin::Root.into(), user, index).expect("must succeed");
	}: _(RawOrigin::Root, index)
	verify {
		assert!(Pallet::<T>::index_to_account(index).is_none());
	}

	set_transact_info {
		let extra_weight: Weight = Weight::from_parts(300000000u64, 0);
		let fee_per_second = 1;
		let max_weight: Weight = Weight::from_parts(20000000000u64, 0);
		let location = MultiLocation::parent();
	}: _(
		RawOrigin::Root,
		Box::new(xcm::VersionedMultiLocation::V3(location.clone())),
		extra_weight,
		max_weight,
		None
	)
	verify {
		assert_eq!(Pallet::<T>::transact_info(&location), Some(crate::RemoteTransactInfoWithMaxWeight {
			transact_extra_weight: extra_weight.into(),
			max_weight: max_weight.into(),
			transact_extra_weight_signed: None
		}));
	}

	remove_transact_info {
		let extra_weight: Weight = Weight::from_parts(300000000u64, 0);
		let max_weight: Weight = Weight::from_parts(20000000000u64, u64::MAX);
		let location = MultiLocation::parent();
		Pallet::<T>::set_transact_info(
			RawOrigin::Root.into(),
			Box::new(xcm::VersionedMultiLocation::V3(location.clone())),
			extra_weight,
			max_weight,
			None
		).expect("must succeed");
	}: _(RawOrigin::Root, Box::new(xcm::VersionedMultiLocation::V3(location.clone())))
	verify {
		assert!(Pallet::<T>::transact_info(&location).is_none());
	}

	set_fee_per_second {
		let fee_per_second = 1;
		let location = MultiLocation::parent();
	}: _(
		RawOrigin::Root,
		Box::new(xcm::VersionedMultiLocation::V3(location.clone())),
		fee_per_second
	)
	verify {
		assert_eq!(Pallet::<T>::dest_asset_fee_per_second(&location), Some(fee_per_second));
	}

	// Worst Case: AsCurrencyId, as the translation could involve db reads
	// Worst Case: transacInfo db reads
	transact_through_derivative {
		let fee_per_second = 1;
		let extra_weight: Weight = Weight::from_parts(300000000u64, 0);
		let max_weight: Weight = Weight::from_parts(20000000000u64, u64::MAX);
		let location = MultiLocation::parent();
		let call = vec![1u8];
		let dest_weight: Weight = Weight::from_parts(100u64, 0);
		let currency: T::CurrencyId = location.clone().into();
		let user: T::AccountId  = account("account id", 0u32, 0u32);
		Pallet::<T>::set_transact_info(
			RawOrigin::Root.into(),
			Box::new(xcm::VersionedMultiLocation::V3(location.clone())),
			extra_weight,
			max_weight,
			Some(extra_weight)
		).expect("must succeed");
		Pallet::<T>::set_fee_per_second(
			RawOrigin::Root.into(),
			Box::new(xcm::VersionedMultiLocation::V3(location.clone())),
			fee_per_second
		).expect("must succeed");
		Pallet::<T>::register(
			RawOrigin::Root.into(),
			user.clone(),
			0
		).expect("must succeed");
	}: {

		let result = Pallet::<T>::transact_through_derivative(
			RawOrigin::Signed(user.clone()).into(),
			T::Transactor::default(),
			0,
			CurrencyPayment {
				// This might involve a db Read when translating, therefore worst case
				currency: Currency::AsCurrencyId(currency),
				// This involves a db Read, hence the None is worst case
				fee_amount: None
			},
			call,
			TransactWeights {
				transact_required_weight_at_most: dest_weight,
				// This involves a db Read, hence the None is worst case
				overall_weight: None
			},
			false
		);

		// It's expected that the error comes from the fact that the asset is not known
		// The weight coming withdraw asset + send is accounted by charging for the instruction per se
		if result.is_ok() {
			assert_eq!(result, Ok(()))
		}
		else {
			assert_eq!(result, Err(crate::Error::<T>::UnableToWithdrawAsset.into()))
		}
	}

	transact_through_sovereign {
		let fee_per_second = 1;
		let extra_weight: Weight = Weight::from_parts(300000000u64, 0);
		let max_weight: Weight = Weight::from_parts(20000000000u64, u64::MAX);
		let location = MultiLocation::parent();
		let currency: T::CurrencyId = location.clone().into();
		let call = vec![1u8];
		let dest_weight: Weight = Weight::from_parts(100u64, 0);
		let user: T::AccountId  = account("account id", 0u32, 0u32);
		Pallet::<T>::set_transact_info(
			RawOrigin::Root.into(),
			Box::new(xcm::VersionedMultiLocation::V3(location.clone())),
			extra_weight,
			max_weight,
			Some(extra_weight)
		).expect("must succeed");
		Pallet::<T>::set_fee_per_second(
			RawOrigin::Root.into(),
			Box::new(xcm::VersionedMultiLocation::V3(location.clone())),
			fee_per_second
		).expect("must succeed");
	}: {

		let result = Pallet::<T>::transact_through_sovereign(
			RawOrigin::Root.into(),
			Box::new(xcm::VersionedMultiLocation::V3(location.clone())),
			user.clone(),
			CurrencyPayment {
				// This might involve a db Read when translating, therefore worst case
				currency: Currency::AsCurrencyId(currency),
				// This involves a db Read, hence the None is worst case
				fee_amount: None
			},
			call,
			OriginKind::SovereignAccount,
			TransactWeights {
				transact_required_weight_at_most: dest_weight,
				// This involves a db Read, hence the None is worst case
				overall_weight: None
			},
			false
		);

		// It's expected that the error comes from the fact that the asset is not known
		// The weight coming withdraw asset + send is accounted by charging for the instruction per se
		if result.is_ok() {
			assert_eq!(result, Ok(()))
		}
		else {
			assert_eq!(result, Err(crate::Error::<T>::UnableToWithdrawAsset.into()))
		}
	}

	transact_through_signed {
		let fee_per_second = 1;
		let extra_weight: Weight = Weight::from_parts(300000000u64, 0);
		let max_weight: Weight = Weight::from_parts(20000000000u64, u64::MAX);
		let location = MultiLocation::parent();
		let currency: T::CurrencyId = location.clone().into();
		let call = vec![1u8];
		let dest_weight: Weight = Weight::from_parts(100u64, 0);
		let user: T::AccountId  = account("account id", 0u32, 0u32);
		Pallet::<T>::set_transact_info(
			RawOrigin::Root.into(),
			Box::new(xcm::VersionedMultiLocation::V3(location.clone())),
			extra_weight,
			max_weight,
			Some(extra_weight)
		).expect("must succeed");
		Pallet::<T>::set_fee_per_second(
			RawOrigin::Root.into(),
			Box::new(xcm::VersionedMultiLocation::V3(location.clone())),
			fee_per_second
		).expect("must succeed");
	}: _(
		RawOrigin::Signed(user.clone()),
		Box::new(xcm::VersionedMultiLocation::V3(location.clone())),
		CurrencyPayment {
			// This might involve a db Read when translating, therefore worst case
			currency: Currency::AsCurrencyId(currency),
			// This involves a db Read, hence the None is worst case
			fee_amount: None
		},
		call,
		TransactWeights {
			transact_required_weight_at_most: dest_weight,
			// This involves a db Read, hence the None is worst case
			overall_weight: None
		},
		false
	)

	hrmp_manage {
		let fee_per_second = 1;
		let extra_weight: Weight = Weight::from_parts(300000000u64, 0);
		let max_weight: Weight = Weight::from_parts(20000000000u64, u64::MAX);
		let location = MultiLocation::parent();
		let currency: T::CurrencyId = location.clone().into();
		let call = vec![1u8];
		let dest_weight: Weight = Weight::from_parts(100u64, 0);
		let user: T::AccountId  = account("account id", 0u32, 0u32);
		Pallet::<T>::set_transact_info(
			RawOrigin::Root.into(),
			Box::new(xcm::VersionedMultiLocation::V3(location.clone())),
			extra_weight,
			max_weight,
			Some(extra_weight)
		).expect("must succeed");
		Pallet::<T>::set_fee_per_second(
			RawOrigin::Root.into(),
			Box::new(xcm::VersionedMultiLocation::V3(location.clone())),
			fee_per_second
		).expect("must succeed");
	}: _(
		RawOrigin::Root,
		HrmpOperation::Accept{ para_id: 1000u32.into() },
		CurrencyPayment {
			// This might involve a db Read when translating, therefore worst case
			currency: Currency::AsCurrencyId(currency),
			// This involves a db Read, hence the None is worst case
			fee_amount: None
		},
		TransactWeights {
			transact_required_weight_at_most: dest_weight,
			// This involves a db Read, hence the None is worst case
			overall_weight: None
		}
	)
}

#[cfg(test)]
mod tests {
	use crate::mock::Test;
	use sp_io::TestExternalities;
	use sp_runtime::BuildStorage;

	pub fn new_test_ext() -> TestExternalities {
		let t = frame_system::GenesisConfig::<Test>::default()
			.build_storage()
			.unwrap();
		TestExternalities::new(t)
	}
}

impl_benchmark_test_suite!(
	Pallet,
	crate::benchmarks::tests::new_test_ext(),
	crate::mock::Test
);
