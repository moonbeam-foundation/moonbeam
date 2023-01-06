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

//! Benchmarking
use crate::{keys_wrapper, BalanceOf, Call, Config, Pallet};
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite};
use frame_support::{
	assert_ok,
	traits::{Currency, Get},
};
use frame_system::RawOrigin;
use nimbus_primitives::NimbusId;
use parity_scale_codec::Decode;

/// Create a funded user.
fn create_funded_user<T: Config>() -> T::AccountId {
	let user = account("account id", 0u32, 0u32);
	T::DepositCurrency::make_free_balance_be(
		&user,
		<<T as Config>::DepositAmount as Get<BalanceOf<T>>>::get(),
	);
	T::DepositCurrency::issue(<<T as Config>::DepositAmount as Get<BalanceOf<T>>>::get());
	user
}

/// Create a valid nimbus id from a simple u8 seed
pub fn nimbus_id(seed: u8) -> NimbusId {
	let id = [seed; 32];
	NimbusId::decode(&mut &id[..]).expect("valid input")
}

benchmarks! {
	add_association {
		let caller = create_funded_user::<T>();
		let id = nimbus_id(1u8);
	}: _(RawOrigin::Signed(caller.clone()), id.clone())
	verify {
		assert_eq!(Pallet::<T>::account_id_of(&id), Some(caller));
	}

	update_association {
		let caller = create_funded_user::<T>();
		let first_id = nimbus_id(1u8);
		let second_id = nimbus_id(2u8);
		assert_ok!(Pallet::<T>::add_association(
			RawOrigin::Signed(caller.clone()).into(),
			first_id.clone())
		);
	}: _(RawOrigin::Signed(caller.clone()), first_id.clone(), second_id.clone())
	verify {
		assert_eq!(Pallet::<T>::account_id_of(&first_id), None);
		assert_eq!(Pallet::<T>::account_id_of(&second_id), Some(caller));
	}

	clear_association {
		let caller = create_funded_user::<T>();
		let first_id = nimbus_id(1u8);
		assert_ok!(Pallet::<T>::add_association(
			RawOrigin::Signed(caller.clone()).into(),
			first_id.clone())
		);
	}: _(RawOrigin::Signed(caller.clone()), first_id.clone())
	verify {
		assert_eq!(Pallet::<T>::account_id_of(&first_id), None);
	}

	remove_keys {
		let caller = create_funded_user::<T>();
		let id = nimbus_id(1u8);
		let keys: T::Keys = nimbus_id(3u8).into();
		assert_ok!(Pallet::<T>::set_keys(
				RawOrigin::Signed(caller.clone()).into(),
				keys_wrapper::<T>(id.clone(), keys.clone()),
			)
		);
	}: _(RawOrigin::Signed(caller.clone()))
	verify {
		assert_eq!(Pallet::<T>::account_id_of(&id), None);
		assert_eq!(Pallet::<T>::nimbus_id_of(&caller), None);
	}

	set_keys {
		let caller = create_funded_user::<T>();
		let first_id = nimbus_id(1u8);
		let first_keys: T::Keys = nimbus_id(3u8).into();
		let second_id = nimbus_id(2u8);
		let second_keys: T::Keys = nimbus_id(3u8).into();
		// we benchmark set_keys after already calling set_keys because
		// key rotation is more common than initially setting them
		assert_ok!(Pallet::<T>::set_keys(
				RawOrigin::Signed(caller.clone()).into(),
				keys_wrapper::<T>(first_id.clone(),
				first_keys.clone()),
			)
		);
	}: _(RawOrigin::Signed(caller.clone()), keys_wrapper::<T>(second_id.clone(), second_keys.clone())
		) verify {
		assert_eq!(Pallet::<T>::account_id_of(&first_id), None);
		assert_eq!(Pallet::<T>::keys_of(&first_id), None);
		assert_eq!(Pallet::<T>::account_id_of(&second_id), Some(caller));
		assert_eq!(Pallet::<T>::keys_of(&second_id), Some(second_keys));
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::mock::Runtime;
	use frame_support::assert_ok;
	use sp_io::TestExternalities;

	pub fn new_test_ext() -> TestExternalities {
		let t = frame_system::GenesisConfig::default()
			.build_storage::<Runtime>()
			.unwrap();
		TestExternalities::new(t)
	}

	#[test]
	fn bench_add_association() {
		new_test_ext().execute_with(|| {
			assert_ok!(Pallet::<Runtime>::test_benchmark_add_association());
		});
	}

	#[test]
	fn bench_update_association() {
		new_test_ext().execute_with(|| {
			assert_ok!(Pallet::<Runtime>::test_benchmark_update_association());
		});
	}

	#[test]
	fn bench_clear_association() {
		new_test_ext().execute_with(|| {
			assert_ok!(Pallet::<Runtime>::test_benchmark_clear_association());
		});
	}
}

impl_benchmark_test_suite!(
	Pallet,
	crate::benchmarks::tests::new_test_ext(),
	crate::mock::Runtime
);
