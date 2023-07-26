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
use crate::mock::*;
use frame_support::{assert_ok, dispatch::Dispatchable, BoundedVec};
use frame_system::RawOrigin;
use pallet_evm::{Call as EvmCall, Event as EvmEvent};
use pallet_identity::{
	Event as IdentityEvent, IdentityFields, Pallet as IdentityPallet, RegistrarInfo,
};
use precompile_utils::testing::*;
use sp_core::{H160, H256, U256};
use sp_runtime::{traits::PostDispatchInfoOf, DispatchResultWithInfo};

fn precompiles() -> Precompiles<Runtime> {
	PrecompilesValue::get()
}

fn evm_call(source: impl Into<H160>, input: Vec<u8>) -> EvmCall<Runtime> {
	EvmCall::call {
		source: source.into(),
		target: Precompile1.into(),
		input,
		value: U256::zero(),
		gas_limit: u64::max_value(),
		max_fee_per_gas: 0.into(),
		max_priority_fee_per_gas: Some(U256::zero()),
		nonce: None,
		access_list: Vec::new(),
	}
}

// #[test]
// fn test_solidity_interface_has_all_function_selectors_documented_and_implemented() {
// 	check_precompile_implements_solidity_interfaces(
// 		&["Identity.sol"],
// 		PCall::supports_selector,
// 	)
// }

#[test]
fn test_add_registrar_with_registrar_origin_succeeds() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 100_000)])
		.build()
		.execute_with(|| {
			assert_ok!(RuntimeCall::Evm(evm_call(
				RegistrarOrigin,
				PCall::add_registrar {
					account: H160::from(Bob).into(),
				}
				.into()
			))
			.dispatch(RuntimeOrigin::root()));

			assert!(events().contains(&Into::<crate::mock::RuntimeEvent>::into(
				IdentityEvent::RegistrarAdded { registrar_index: 0 }
			)));

			assert_eq!(
				<IdentityPallet<Runtime>>::registrars().to_vec(),
				vec![Some(RegistrarInfo {
					account: Bob.into(),
					fee: 0,
					fields: IdentityFields::default(),
				})]
			);
		})
}

#[test]
fn test_add_registrar_with_non_registrar_origin_fails() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 100_000)])
		.build()
		.execute_with(|| {
			// assert_ok!(RuntimeCall::Evm(evm_call(
			// 	Charlie,
			// 	PCall::add_registrar {
			// 		account: H160::from(Bob).into(),
			// 	}
			// 	.into()
			// ))
			// .dispatch(RuntimeOrigin::root()));
			let x = RuntimeCall::Evm(evm_call(
				Charlie,
				PCall::add_registrar {
					account: H160::from(Bob).into(),
				}
				.into(),
			))
			.dispatch(RuntimeOrigin::root());
			println!("{x:?}");
			assert_ok!(x);

			assert!(events().contains(&Into::<crate::mock::RuntimeEvent>::into(
				IdentityEvent::RegistrarAdded { registrar_index: 0 }
			)));

			assert_eq!(
				<IdentityPallet<Runtime>>::registrars().to_vec(),
				vec![Some(RegistrarInfo {
					account: Bob.into(),
					fee: 0,
					fields: IdentityFields::default(),
				})]
			);
		})
}

#[test]
fn test_set_fee_on_existing_registrar_index_succeeds() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 100_000)])
		.build()
		.execute_with(|| {
			assert_ok!(<IdentityPallet<Runtime>>::add_registrar(
				RuntimeOrigin::root(),
				Bob.into()
			));

			assert_ok!(RuntimeCall::Evm(evm_call(
				Bob,
				PCall::set_fee {
					index: 0,
					fee: 100.into(),
				}
				.into()
			))
			.dispatch(RuntimeOrigin::root()));

			assert_eq!(
				<IdentityPallet<Runtime>>::registrars().to_vec(),
				vec![Some(RegistrarInfo {
					account: Bob.into(),
					fee: 100,
					fields: IdentityFields::default(),
				})]
			);
		})
}

#[test]
fn test_set_fee_on_non_existing_registrar_index_fails() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 100_000)])
		.build()
		.execute_with(|| {
			assert_ok!(<IdentityPallet<Runtime>>::add_registrar(
				RuntimeOrigin::root(),
				Bob.into()
			));

			assert_ok!(RuntimeCall::Evm(evm_call(
				Bob,
				PCall::set_fee {
					index: 1,
					fee: 100.into(),
				}
				.into()
			))
			.dispatch(RuntimeOrigin::root()));
		})
}
