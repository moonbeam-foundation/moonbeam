// Copyright 2019-2021 PureStake Inc.
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

use crate::{
	mock::{
		events, evm_test_context, Call, ExtBuilder, Origin, Precompiles, PrecompilesValue, Runtime,
		TestAccount::{Alice, Precompile},
	},
	Action,
};
use fp_evm::PrecompileFailure;
use frame_support::{assert_ok, dispatch::Dispatchable};
use nimbus_primitives::NimbusId;
use pallet_author_mapping::{Call as AuthorMappingCall, Event as AuthorMappingEvent};
use pallet_balances::Event as BalancesEvent;
use pallet_evm::{Call as EvmCall, Event as EvmEvent, PrecompileSet};
use precompile_utils::EvmDataWriter;
use sp_core::crypto::UncheckedFrom;
use sp_core::U256;
use std::assert_matches::assert_matches;

fn precompiles() -> Precompiles<Runtime> {
	PrecompilesValue::get()
}

fn evm_call(input: Vec<u8>) -> EvmCall<Runtime> {
	EvmCall::call {
		source: Alice.into(),
		target: Precompile.into(),
		input,
		value: U256::zero(), // No value sent in EVM
		gas_limit: u64::max_value(),
		max_fee_per_gas: 0.into(),
		max_priority_fee_per_gas: Some(U256::zero()),
		nonce: None, // Use the next nonce
		access_list: Vec::new(),
	}
}

#[test]
fn selector_less_than_four_bytes() {
	ExtBuilder::default().build().execute_with(|| {
		// This selector is only three bytes long when four are required.
		let bogus_selector = vec![1u8, 2u8, 3u8];

		assert_matches!(
			precompiles().execute(
				Precompile.into(),
				&bogus_selector,
				None,
				&evm_test_context(),
				false,
			),
			Some(Err(PrecompileFailure::Revert { output, ..}))
				if output == b"tried to parse selector out of bounds",
		);
	});
}

#[test]
fn no_selector_exists_but_length_is_right() {
	ExtBuilder::default().build().execute_with(|| {
		let bogus_selector = vec![1u8, 2u8, 3u8, 4u8];

		assert_matches!(
			precompiles().execute(
				Precompile.into(),
				&bogus_selector,
				None,
				&evm_test_context(),
				false,
			),
			Some(Err(PrecompileFailure::Revert { output, ..}))
				if output == b"unknown selector",
		);
	});
}

#[test]
fn selectors() {
	assert_eq!(Action::AddAssociation as u32, 0xaa5ac585);
	assert_eq!(Action::UpdateAssociation as u32, 0xd9cef879);
	assert_eq!(Action::ClearAssociation as u32, 0x7354c91d);
}

#[test]
fn add_association_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)])
		.build()
		.execute_with(|| {
			let expected_nimbus_id: NimbusId =
				sp_core::sr25519::Public::unchecked_from([1u8; 32]).into();

			let input = EvmDataWriter::new_with_selector(Action::AddAssociation)
				.write(sp_core::H256::from([1u8; 32]))
				.build();

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(evm_call(input)).dispatch(Origin::root()));

			// Assert that the events are as expected
			assert_eq!(
				events(),
				vec![
					BalancesEvent::Reserved {
						who: Alice,
						amount: 10
					}
					.into(),
					AuthorMappingEvent::AuthorRegistered(expected_nimbus_id, Alice).into(),
					EvmEvent::Executed(Precompile.into()).into(),
				]
			);
		})
}

#[test]
fn update_association_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)])
		.build()
		.execute_with(|| {
			let first_nimbus_id: NimbusId =
				sp_core::sr25519::Public::unchecked_from([1u8; 32]).into();
			let second_nimbus_id: NimbusId =
				sp_core::sr25519::Public::unchecked_from([2u8; 32]).into();

			assert_ok!(Call::AuthorMapping(AuthorMappingCall::add_association {
				author_id: first_nimbus_id.clone(),
			})
			.dispatch(Origin::signed(Alice)));

			let input = EvmDataWriter::new_with_selector(Action::UpdateAssociation)
				.write(sp_core::H256::from([1u8; 32]))
				.write(sp_core::H256::from([2u8; 32]))
				.build();

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(evm_call(input)).dispatch(Origin::root()));

			// Assert that the events are as expected
			assert_eq!(
				events(),
				vec![
					BalancesEvent::Reserved {
						who: Alice,
						amount: 10
					}
					.into(),
					AuthorMappingEvent::AuthorRegistered(first_nimbus_id, Alice).into(),
					AuthorMappingEvent::AuthorRotated(second_nimbus_id, Alice).into(),
					EvmEvent::Executed(Precompile.into()).into(),
				]
			);
		})
}

#[test]
fn clear_association_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)])
		.build()
		.execute_with(|| {
			let nimbus_id: NimbusId = sp_core::sr25519::Public::unchecked_from([1u8; 32]).into();

			assert_ok!(Call::AuthorMapping(AuthorMappingCall::add_association {
				author_id: nimbus_id.clone(),
			})
			.dispatch(Origin::signed(Alice)));

			let input = EvmDataWriter::new_with_selector(Action::ClearAssociation)
				.write(sp_core::H256::from([1u8; 32]))
				.build();

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(evm_call(input)).dispatch(Origin::root()));

			// Assert that the events are as expected
			assert_eq!(
				events(),
				vec![
					BalancesEvent::Reserved {
						who: Alice,
						amount: 10
					}
					.into(),
					AuthorMappingEvent::AuthorRegistered(nimbus_id.clone(), Alice).into(),
					BalancesEvent::Unreserved {
						who: Alice,
						amount: 10
					}
					.into(),
					AuthorMappingEvent::AuthorDeRegistered(nimbus_id).into(),
					EvmEvent::Executed(Precompile.into()).into(),
				]
			);
		})
}
