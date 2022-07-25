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

//! # Randomness Pallet Unit Tests
use crate::mock::*;
use crate::*;
use frame_support::{assert_noop, assert_ok};
use sp_core::{H160, H256};

pub const ALICE: H160 = H160::repeat_byte(0xAA);
pub const BOB: H160 = H160::repeat_byte(0xBB);

/// Helps test same effects for all 4 variants of RequestType
fn build_default_request(info: RequestType<Test>) -> Request<BalanceOf<Test>, RequestType<Test>> {
	Request {
		refund_address: BOB,
		contract_address: ALICE,
		fee: 5,
		gas_limit: 100u64,
		num_words: 1u8,
		salt: H256::default(),
		info,
	}
}

// REQUEST RANDOMNESS

#[test]
fn cannot_make_local_request_already_fulfillable() {
	ExtBuilder::default()
		.with_balances(vec![(ALICE, 15)])
		.build()
		.execute_with(|| {
			let request = build_default_request(RequestType::Local(0u64));
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
			let request = build_default_request(RequestType::Local(22u64));
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
			let request = build_default_request(RequestType::BabeEpoch(16u64));
			assert_noop!(
				Randomness::request_randomness(request),
				sp_runtime::DispatchError::Module(sp_runtime::ModuleError {
					index: 1,
					error: [2, 0, 0, 0],
					message: Some("InsufficientBalance")
				})
			);
			let request = build_default_request(RequestType::Local(16u64));
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
			let request = build_default_request(RequestType::BabeEpoch(16u64));
			assert_noop!(
				Randomness::request_randomness(request),
				sp_runtime::DispatchError::Module(sp_runtime::ModuleError {
					index: 1,
					error: [2, 0, 0, 0],
					message: Some("InsufficientBalance")
				})
			);
			let request = build_default_request(RequestType::Local(16u64));
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
			let request = build_default_request(RequestType::BabeEpoch(16u64));
			assert_ok!(Randomness::request_randomness(request));
			assert_eq!(Randomness::total_locked(), 15);
			assert_eq!(Balances::free_balance(&ALICE), 15);
			let request = build_default_request(RequestType::Local(16u64));
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
			let request = build_default_request(RequestType::BabeEpoch(16u64));
			assert_ok!(Randomness::request_randomness(request));
			assert_eq!(Randomness::request_count(), 1);
			let request = build_default_request(RequestType::Local(16u64));
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
			let request = build_default_request(RequestType::BabeEpoch(16u64));
			assert_eq!(Randomness::requests(0), None);
			assert_ok!(Randomness::request_randomness(request.clone()));
			assert_eq!(
				Randomness::requests(0),
				Some(RequestState {
					request: request.into(),
					deposit: 10,
				})
			);
			let request = build_default_request(RequestType::Local(16u64));
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
				info: RequestType::BabeEpoch(16u64),
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
				earliest_epoch: 16u64,
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
				info: RequestType::Local(16u64),
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
				earliest_block: 16u64,
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
				info: RequestType::Local(16u64),
			};
			assert_ok!(Randomness::request_randomness(request));
			let result = Randomness::randomness_results(RequestType::Local(16u64)).unwrap();
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
				info: RequestType::Local(16u64),
			};
			assert_ok!(Randomness::request_randomness(request.clone()));
			assert_ok!(Randomness::request_randomness(request));
			let result = Randomness::randomness_results(RequestType::Local(16u64)).unwrap();
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
				info: RequestType::Local(16u64),
			};
			assert_ok!(Randomness::request_randomness(request));
			System::set_block_number(16u64);
			let mut result =
				crate::pallet::RandomnessResults::<Test>::get(RequestType::Local(16u64)).unwrap();
			result.randomness = Some(H256::default());
			crate::pallet::RandomnessResults::<Test>::insert(RequestType::Local(16u64), result);
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
				info: RequestType::Local(16u64),
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
				info: RequestType::Local(16u64),
			};
			assert_ok!(Randomness::request_randomness(request));
			System::set_block_number(16u64);
			let mut pre_result =
				crate::pallet::RandomnessResults::<Test>::get(RequestType::Local(16u64)).unwrap();
			pre_result.randomness = Some(H256::default());
			crate::pallet::RandomnessResults::<Test>::insert(RequestType::Local(16u64), pre_result);
			assert_ok!(Randomness::prepare_fulfillment(0u64));
			let post_result =
				crate::pallet::RandomnessResults::<Test>::get(RequestType::Local(16u64)).unwrap();
			assert_eq!(post_result.request_count, 1);
		});
}

#[test]
fn set_babe_randomness_results_is_mandatory() {
	use frame_support::weights::{DispatchClass, GetDispatchInfo};

	let info = crate::Call::<Test>::set_babe_randomness_results {}.get_dispatch_info();
	assert_eq!(info.class, DispatchClass::Mandatory);
}

// FINISH FULFILLMENT

// finish fulfillment decrements randomness result and will remove it if last
// test both cases separately

// INCREASE REQUEST FEE

// increase request fee updates the request fee

// EXECUTE REQUEST EXPIRATION

// execute request expiration fails before expired

// execute request expiration succeeds

// ON INITIALIZE LOGIC AND HOOKS
