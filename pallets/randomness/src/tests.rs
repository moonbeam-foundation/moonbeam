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
use crate::*;
use frame_support::{assert_noop, assert_ok};
use sp_core::H256;

#[test]
fn pallet_account_id() {
	assert_eq!(
		Randomness::account_id(),
		core::str::FromStr::from_str("0x6d6f646c6d6f6f6e72616e640000000000000000").unwrap(),
	);
}

#[test]
fn set_babe_randomness_results_is_mandatory() {
	use frame_support::dispatch::{DispatchClass, GetDispatchInfo};

	let info = crate::Call::<Test>::set_babe_randomness_results {}.get_dispatch_info();
	assert_eq!(info.class, DispatchClass::Mandatory);
}

// REQUEST RANDOMNESS

#[test]
fn cannot_make_local_request_already_fulfillable() {
	ExtBuilder::default()
		.with_balances(vec![(ALICE, 15)])
		.build()
		.execute_with(|| {
			let request = build_default_request(RequestType::Local(0));
			assert_noop!(
				Randomness::request_randomness(request),
				Error::<Test>::CannotRequestRandomnessBeforeMinDelay
			);
		});
}

#[test]
fn cannot_make_request_fulfillable_past_expiry() {
	ExtBuilder::default()
		.with_balances(vec![(ALICE, 15)])
		.build()
		.execute_with(|| {
			let request = build_default_request(RequestType::Local(22));
			assert_noop!(
				Randomness::request_randomness(request),
				Error::<Test>::CannotRequestRandomnessAfterMaxDelay
			);
		});
}

#[test]
fn cannot_make_request_with_less_than_deposit() {
	ExtBuilder::default()
		.with_balances(vec![(ALICE, 9)])
		.build()
		.execute_with(|| {
			let request = build_default_request(RequestType::BabeEpoch(16));
			assert_noop!(
				Randomness::request_randomness(request),
				sp_runtime::DispatchError::Module(sp_runtime::ModuleError {
					index: 1,
					error: [2, 0, 0, 0],
					message: Some("InsufficientBalance")
				})
			);
			let request = build_default_request(RequestType::Local(16));
			assert_noop!(
				Randomness::request_randomness(request),
				sp_runtime::DispatchError::Module(sp_runtime::ModuleError {
					index: 1,
					error: [2, 0, 0, 0],
					message: Some("InsufficientBalance")
				})
			);
		});
}

#[test]
fn cannot_make_request_with_less_than_deposit_plus_fee() {
	ExtBuilder::default()
		.with_balances(vec![(ALICE, 14)])
		.build()
		.execute_with(|| {
			let request = build_default_request(RequestType::BabeEpoch(16));
			assert_noop!(
				Randomness::request_randomness(request),
				sp_runtime::DispatchError::Module(sp_runtime::ModuleError {
					index: 1,
					error: [2, 0, 0, 0],
					message: Some("InsufficientBalance")
				})
			);
			let request = build_default_request(RequestType::Local(16));
			assert_noop!(
				Randomness::request_randomness(request),
				sp_runtime::DispatchError::Module(sp_runtime::ModuleError {
					index: 1,
					error: [2, 0, 0, 0],
					message: Some("InsufficientBalance")
				})
			);
		});
}

#[test]
fn request_reserves_deposit_and_fee() {
	ExtBuilder::default()
		.with_balances(vec![(ALICE, 30)])
		.build()
		.execute_with(|| {
			assert_eq!(Randomness::total_locked(), 0);
			assert_eq!(Balances::free_balance(&ALICE), 30);
			let request = build_default_request(RequestType::BabeEpoch(16));
			assert_ok!(Randomness::request_randomness(request));
			assert_eq!(Randomness::total_locked(), 15);
			assert_eq!(Balances::free_balance(&ALICE), 15);
			let request = build_default_request(RequestType::Local(16));
			assert_ok!(Randomness::request_randomness(request));
			assert_eq!(Randomness::total_locked(), 30);
			assert_eq!(Balances::free_balance(&ALICE), 0);
		});
}

#[test]
fn request_babe_current_block_randomness_increments_request_counter() {
	ExtBuilder::default()
		.with_balances(vec![(ALICE, 60)])
		.build()
		.execute_with(|| {
			assert_eq!(Randomness::request_count(), 0);
			let request = build_default_request(RequestType::BabeEpoch(16));
			assert_ok!(Randomness::request_randomness(request));
			assert_eq!(Randomness::request_count(), 1);
			let request = build_default_request(RequestType::Local(16));
			assert_ok!(Randomness::request_randomness(request));
			assert_eq!(Randomness::request_count(), 2);
		});
}

#[test]
fn request_babe_current_block_randomness_inserts_request_state() {
	ExtBuilder::default()
		.with_balances(vec![(ALICE, 60)])
		.build()
		.execute_with(|| {
			let request = build_default_request(RequestType::BabeEpoch(16));
			assert_eq!(Randomness::requests(0), None);
			assert_ok!(Randomness::request_randomness(request.clone()));
			assert_eq!(
				Randomness::requests(0),
				Some(RequestState {
					request: request.into(),
					deposit: 10,
				})
			);
			let request = build_default_request(RequestType::Local(16));
			assert_eq!(Randomness::requests(1), None);
			assert_ok!(Randomness::request_randomness(request.clone()));
			assert_eq!(
				Randomness::requests(1),
				Some(RequestState {
					request: request.into(),
					deposit: 10,
				})
			);
		});
}

// REQUEST RANDOMNESS EVENTS EMIT BASED ON REQUESTED TYPE OF RANDOMNESS

#[test]
fn request_babe_one_epoch_ago_randomness_emits_event() {
	ExtBuilder::default()
		.with_balances(vec![(ALICE, 15)])
		.build()
		.execute_with(|| {
			let request = Request {
				refund_address: BOB.into(),
				contract_address: ALICE.into(),
				fee: 5,
				gas_limit: 100u64,
				num_words: 1u8,
				salt: H256::default(),
				info: RequestType::BabeEpoch(16),
			};
			assert_ok!(Randomness::request_randomness(request));
			assert_event_emitted!(crate::Event::RandomnessRequestedBabeEpoch {
				id: 0,
				refund_address: BOB.into(),
				contract_address: ALICE.into(),
				fee: 5,
				gas_limit: 100u64,
				num_words: 1u8,
				salt: H256::default(),
				earliest_epoch: 16,
			});
		});
}

#[test]
fn request_local_randomness_emits_event() {
	ExtBuilder::default()
		.with_balances(vec![(ALICE, 15)])
		.build()
		.execute_with(|| {
			let request = Request {
				refund_address: BOB.into(),
				contract_address: ALICE.into(),
				fee: 5,
				gas_limit: 100u64,
				num_words: 1u8,
				salt: H256::default(),
				info: RequestType::Local(16),
			};
			assert_ok!(Randomness::request_randomness(request));
			assert_event_emitted!(crate::Event::RandomnessRequestedLocal {
				id: 0,
				refund_address: BOB.into(),
				contract_address: ALICE.into(),
				fee: 5,
				gas_limit: 100u64,
				num_words: 1u8,
				salt: H256::default(),
				earliest_block: 16,
			});
		});
}

#[test]
fn request_randomness_adds_new_randomness_result() {
	ExtBuilder::default()
		.with_balances(vec![(ALICE, 15)])
		.build()
		.execute_with(|| {
			let request = Request {
				refund_address: BOB.into(),
				contract_address: ALICE.into(),
				fee: 5,
				gas_limit: 100u64,
				num_words: 1u8,
				salt: H256::default(),
				info: RequestType::Local(16),
			};
			assert_ok!(Randomness::request_randomness(request));
			let result = Randomness::randomness_results(RequestType::Local(16)).unwrap();
			assert_eq!(result.request_count, 1u64);
			assert!(result.randomness.is_none());
		});
}

#[test]
fn request_randomness_increments_randomness_result() {
	ExtBuilder::default()
		.with_balances(vec![(ALICE, 30)])
		.build()
		.execute_with(|| {
			let request = Request {
				refund_address: BOB.into(),
				contract_address: ALICE.into(),
				fee: 5,
				gas_limit: 100u64,
				num_words: 1u8,
				salt: H256::default(),
				info: RequestType::Local(16),
			};
			assert_ok!(Randomness::request_randomness(request.clone()));
			assert_ok!(Randomness::request_randomness(request));
			let result = Randomness::randomness_results(RequestType::Local(16)).unwrap();
			assert_eq!(result.request_count, 2u64);
			assert!(result.randomness.is_none());
		});
}

// PREPARE FULFILLMENT

#[test]
fn prepare_fulfillment_for_local_works() {
	ExtBuilder::default()
		.with_balances(vec![(ALICE, 30)])
		.build()
		.execute_with(|| {
			let request = Request {
				refund_address: BOB.into(),
				contract_address: ALICE.into(),
				fee: 5,
				gas_limit: 100u64,
				num_words: 1u8,
				salt: H256::default(),
				info: RequestType::Local(16),
			};
			assert_ok!(Randomness::request_randomness(request));
			System::set_block_number(16);
			let mut result =
				crate::pallet::RandomnessResults::<Test>::get(RequestType::Local(16)).unwrap();
			result.randomness = Some(H256::default());
			crate::pallet::RandomnessResults::<Test>::insert(RequestType::Local(16), result);
			assert_ok!(Randomness::prepare_fulfillment(0u64));
		});
}

#[test]
fn prepare_fulfillment_fails_before_can_be_fulfilled() {
	ExtBuilder::default()
		.with_balances(vec![(ALICE, 30)])
		.build()
		.execute_with(|| {
			let request = Request {
				refund_address: BOB.into(),
				contract_address: ALICE.into(),
				fee: 5,
				gas_limit: 100u64,
				num_words: 1u8,
				salt: H256::default(),
				info: RequestType::Local(16),
			};
			assert_ok!(Randomness::request_randomness(request.clone()));
			assert_ok!(Randomness::request_randomness(request));
			assert_noop!(
				Randomness::prepare_fulfillment(0u64),
				Error::<Test>::RequestCannotYetBeFulfilled
			);
		});
}

#[test]
fn prepare_fulfillment_fails_if_request_dne() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			Randomness::prepare_fulfillment(0u64),
			Error::<Test>::RequestDNE
		);
	});
}

#[test]
fn prepare_fulfillment_uses_randomness_result_without_updating_count() {
	ExtBuilder::default()
		.with_balances(vec![(ALICE, 30)])
		.build()
		.execute_with(|| {
			let request = Request {
				refund_address: BOB.into(),
				contract_address: ALICE.into(),
				fee: 5,
				gas_limit: 100u64,
				num_words: 1u8,
				salt: H256::default(),
				info: RequestType::Local(16),
			};
			assert_ok!(Randomness::request_randomness(request));
			System::set_block_number(16);
			let mut pre_result =
				crate::pallet::RandomnessResults::<Test>::get(RequestType::Local(16)).unwrap();
			pre_result.randomness = Some(H256::default());
			crate::pallet::RandomnessResults::<Test>::insert(RequestType::Local(16), pre_result);
			assert_ok!(Randomness::prepare_fulfillment(0u64));
			let post_result =
				crate::pallet::RandomnessResults::<Test>::get(RequestType::Local(16)).unwrap();
			assert_eq!(post_result.request_count, 1);
		});
}

// FINISH FULFILLMENT

#[test]
fn finish_fulfillment_removes_request_from_storage() {
	ExtBuilder::default()
		.with_balances(vec![(ALICE, 30)])
		.build()
		.execute_with(|| {
			let request = Request {
				refund_address: BOB.into(),
				contract_address: ALICE.into(),
				fee: 5,
				gas_limit: 100u64,
				num_words: 1u8,
				salt: H256::default(),
				info: RequestType::Local(16),
			};
			assert_ok!(Randomness::request_randomness(request.clone()));
			assert_ok!(Randomness::request_randomness(request));
			System::set_block_number(16);
			let mut pre_result =
				crate::pallet::RandomnessResults::<Test>::get(RequestType::Local(16)).unwrap();
			pre_result.randomness = Some(H256::default());
			crate::pallet::RandomnessResults::<Test>::insert(RequestType::Local(16), pre_result);
			let fulfill_args = Randomness::prepare_fulfillment(0u64).unwrap();
			Randomness::finish_fulfillment(
				1u64,
				fulfill_args.request,
				fulfill_args.deposit,
				&ALICE,
				5,
			);
			assert!(Randomness::requests(1u64).is_none());
		});
}

#[test]
fn finish_fulfillment_refunds_refund_address_with_excess_and_caller_with_cost_of_execution() {
	ExtBuilder::default()
		.with_balances(vec![(ALICE, 30)])
		.build()
		.execute_with(|| {
			let request = Request {
				refund_address: BOB.into(),
				contract_address: ALICE.into(),
				fee: 5,
				gas_limit: 100u64,
				num_words: 1u8,
				salt: H256::default(),
				info: RequestType::Local(16),
			};
			assert_ok!(Randomness::request_randomness(request));
			System::set_block_number(16);
			let mut pre_result =
				crate::pallet::RandomnessResults::<Test>::get(RequestType::Local(16)).unwrap();
			pre_result.randomness = Some(H256::default());
			crate::pallet::RandomnessResults::<Test>::insert(RequestType::Local(16), pre_result);
			let fulfill_args = Randomness::prepare_fulfillment(0u64).unwrap();
			Randomness::finish_fulfillment(
				1u64,
				fulfill_args.request,
				fulfill_args.deposit,
				&ALICE,
				3,
			);
			// 30 - ( deposit = 10 + fee = 5) + cost_of_execution_refund_for_caller = 3 == 18
			assert_eq!(Balances::free_balance(&ALICE), 18);
			// 0 + deposit = 10 + fee = 5 - cost_of_execution = 3 == 12
			assert_eq!(Balances::free_balance(&BOB), 12);
		});
}

#[test]
fn finish_fulfillment_decrements_randomness_result_and_keeps_in_storage_if_not_last() {
	ExtBuilder::default()
		.with_balances(vec![(ALICE, 30)])
		.build()
		.execute_with(|| {
			let request = Request {
				refund_address: BOB.into(),
				contract_address: ALICE.into(),
				fee: 5,
				gas_limit: 100u64,
				num_words: 1u8,
				salt: H256::default(),
				info: RequestType::Local(16),
			};
			assert_ok!(Randomness::request_randomness(request.clone()));
			assert_ok!(Randomness::request_randomness(request));
			System::set_block_number(16);
			let mut pre_result =
				crate::pallet::RandomnessResults::<Test>::get(RequestType::Local(16)).unwrap();
			pre_result.randomness = Some(H256::default());
			crate::pallet::RandomnessResults::<Test>::insert(RequestType::Local(16), pre_result);
			let fulfill_args = Randomness::prepare_fulfillment(0u64).unwrap();
			assert_eq!(
				Randomness::randomness_results(RequestType::Local(16))
					.unwrap()
					.request_count,
				2
			);
			Randomness::finish_fulfillment(
				1u64,
				fulfill_args.request,
				fulfill_args.deposit,
				&ALICE,
				5,
			);
			assert_eq!(
				Randomness::randomness_results(RequestType::Local(16))
					.unwrap()
					.request_count,
				1
			);
		});
}

#[test]
fn finish_fulfillment_decrements_randomness_result_and_removes_from_storage_if_last() {
	ExtBuilder::default()
		.with_balances(vec![(ALICE, 30)])
		.build()
		.execute_with(|| {
			let request = Request {
				refund_address: BOB.into(),
				contract_address: ALICE.into(),
				fee: 5,
				gas_limit: 100u64,
				num_words: 1u8,
				salt: H256::default(),
				info: RequestType::Local(16),
			};
			assert_ok!(Randomness::request_randomness(request));
			System::set_block_number(16);
			let mut pre_result =
				crate::pallet::RandomnessResults::<Test>::get(RequestType::Local(16)).unwrap();
			pre_result.randomness = Some(H256::default());
			crate::pallet::RandomnessResults::<Test>::insert(RequestType::Local(16), pre_result);
			let fulfill_args = Randomness::prepare_fulfillment(0).unwrap();
			assert_eq!(
				Randomness::randomness_results(RequestType::Local(16))
					.unwrap()
					.request_count,
				1
			);
			Randomness::finish_fulfillment(
				1u64,
				fulfill_args.request,
				fulfill_args.deposit,
				&ALICE,
				5,
			);
			assert!(Randomness::randomness_results(RequestType::Local(16)).is_none());
		});
}

// INCREASE REQUEST FEE

#[test]
fn increase_request_fee_fails_if_request_dne() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			Randomness::increase_request_fee(&ALICE, 1u64, 10),
			Error::<Test>::RequestDNE
		);
	});
}

#[test]
fn non_requester_cannot_increase_fee() {
	ExtBuilder::default()
		.with_balances(vec![(ALICE, 30)])
		.build()
		.execute_with(|| {
			let request = Request {
				refund_address: BOB.into(),
				contract_address: ALICE.into(),
				fee: 5,
				gas_limit: 100u64,
				num_words: 1u8,
				salt: H256::default(),
				info: RequestType::Local(16),
			};
			assert_ok!(Randomness::request_randomness(request));
			assert_noop!(
				Randomness::increase_request_fee(&BOB, 0u64, 6),
				Error::<Test>::OnlyRequesterCanIncreaseFee
			);
		});
}

#[test]
fn increase_request_fee_transfers_from_caller_and_updates_request_state_fee() {
	ExtBuilder::default()
		.with_balances(vec![(ALICE, 30)])
		.build()
		.execute_with(|| {
			let request = Request {
				refund_address: BOB.into(),
				contract_address: ALICE.into(),
				fee: 5,
				gas_limit: 100u64,
				num_words: 1u8,
				salt: H256::default(),
				info: RequestType::Local(16),
			};
			assert_ok!(Randomness::request_randomness(request));
			assert_ok!(Randomness::increase_request_fee(&ALICE, 0u64, 6));
			// initial_fee = 5 + fee_increase = 6 == 11
			assert_eq!(Randomness::requests(0u64).unwrap().request.fee, 11);
			// initial_balance = 30 - deposit = 10 - initial_fee = 5 - fee_increase = 6 == 9
			assert_eq!(Balances::free_balance(&ALICE), 9);
		});
}

#[test]
fn increase_request_fee_fails_if_insufficient_balance() {
	ExtBuilder::default()
		.with_balances(vec![(ALICE, 20)])
		.build()
		.execute_with(|| {
			let request = Request {
				refund_address: BOB.into(),
				contract_address: ALICE.into(),
				fee: 5,
				gas_limit: 100u64,
				num_words: 1u8,
				salt: H256::default(),
				info: RequestType::Local(16),
			};
			assert_ok!(Randomness::request_randomness(request));
			assert_noop!(
				Randomness::increase_request_fee(&ALICE, 0u64, 6),
				sp_runtime::DispatchError::Module(sp_runtime::ModuleError {
					index: 1,
					error: [2, 0, 0, 0],
					message: Some("InsufficientBalance")
				})
			);
		});
}

// EXECUTE REQUEST EXPIRATION

#[test]
fn execute_request_expiration_fails_if_request_dne() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			Randomness::execute_request_expiration(&ALICE, 1u64),
			Error::<Test>::RequestDNE
		);
	});
}

#[test]
fn execute_request_expiration_fails_before_request_expiration() {
	ExtBuilder::default()
		.with_balances(vec![(ALICE, 30)])
		.build()
		.execute_with(|| {
			assert_ok!(Randomness::request_randomness(build_default_request(
				RequestType::BabeEpoch(16)
			)));
			assert_noop!(
				Randomness::execute_request_expiration(&ALICE, 0u64),
				Error::<Test>::RequestHasNotExpired
			);
		});
}

#[test]
fn execute_request_expiration_removes_request() {
	ExtBuilder::default()
		.with_balances(vec![(ALICE, 30)])
		.build()
		.execute_with(|| {
			assert_ok!(Randomness::request_randomness(build_default_request(
				RequestType::BabeEpoch(16)
			)));
			// increase epoch to expiry
			crate::pallet::RelayEpoch::<Test>::put(20u64);
			assert!(Randomness::requests(0u64).is_some());
			// execute expiry
			assert_ok!(Randomness::execute_request_expiration(&BOB, 0u64));
			assert!(Randomness::requests(0u64).is_none());
		});
}

#[test]
fn execute_request_expiration_removes_result() {
	ExtBuilder::default()
		.with_balances(vec![(ALICE, 30)])
		.build()
		.execute_with(|| {
			assert_ok!(Randomness::request_randomness(build_default_request(
				RequestType::BabeEpoch(16)
			)));
			// increase epoch to expiry
			crate::pallet::RelayEpoch::<Test>::put(20u64);
			assert!(Randomness::randomness_results(RequestType::BabeEpoch(16)).is_some());
			// execute expiry
			assert_ok!(Randomness::execute_request_expiration(&BOB, 0u64));
			assert!(Randomness::randomness_results(RequestType::BabeEpoch(16)).is_none());
		});
}

#[test]
fn execute_request_expiration_returns_deposit_to_contract_address_and_fees_to_caller() {
	ExtBuilder::default()
		.with_balances(vec![(ALICE, 30)])
		.build()
		.execute_with(|| {
			assert_ok!(Randomness::request_randomness(build_default_request(
				RequestType::BabeEpoch(16)
			)));
			crate::pallet::RelayEpoch::<Test>::put(20u64);
			assert_ok!(Randomness::execute_request_expiration(&BOB, 0u64));
			// fee returned to BOB (caller)
			assert_eq!(Balances::free_balance(&BOB), 5);
			// deposit returned to ALICE (contract_address)
			assert_eq!(Balances::free_balance(&ALICE), 25);
		});
}
