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
use sp_core::H256;

/// Helps test same effects for all 4 variants of RequestType
fn build_default_request(info: RequestType<Test>) -> Request<Test> {
	Request {
		refund_address: Account::Bob.into(),
		contract_address: Account::Alice.into(),
		fee: 5,
		gas_limit: 100u64,
		salt: H256::default(),
		info,
	}
}

// REQUEST RANDOMNESS

#[test]
fn cannot_make_request_already_fulfillable() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 15)])
		.build()
		.execute_with(|| {
			let request = build_default_request(RequestType::BabeCurrentBlock(0u64));
			assert_noop!(
				Randomness::request_randomness(request),
				Error::<Test>::CannotRequestPastRandomness
			);
			let request = build_default_request(RequestType::BabeOneEpochAgo(0u64));
			assert_noop!(
				Randomness::request_randomness(request),
				Error::<Test>::CannotRequestPastRandomness
			);
			let request = build_default_request(RequestType::BabeTwoEpochsAgo(0u64));
			assert_noop!(
				Randomness::request_randomness(request),
				Error::<Test>::CannotRequestPastRandomness
			);
			let request = build_default_request(RequestType::Local(0u64));
			assert_noop!(
				Randomness::request_randomness(request),
				Error::<Test>::CannotRequestPastRandomness
			);
		});
}

#[test]
fn cannot_make_request_with_less_than_deposit() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 9)])
		.build()
		.execute_with(|| {
			let request = build_default_request(RequestType::BabeCurrentBlock(16u64));
			assert_noop!(
				Randomness::request_randomness(request),
				Error::<Test>::InsufficientDeposit
			);
			let request = build_default_request(RequestType::BabeOneEpochAgo(16u64));
			assert_noop!(
				Randomness::request_randomness(request),
				Error::<Test>::InsufficientDeposit
			);
			let request = build_default_request(RequestType::BabeTwoEpochsAgo(16u64));
			assert_noop!(
				Randomness::request_randomness(request),
				Error::<Test>::InsufficientDeposit
			);
			let request = build_default_request(RequestType::Local(16u64));
			assert_noop!(
				Randomness::request_randomness(request),
				Error::<Test>::InsufficientDeposit
			);
		});
}

#[test]
fn cannot_make_request_with_less_than_deposit_plus_fee() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 14)])
		.build()
		.execute_with(|| {
			let request = build_default_request(RequestType::BabeCurrentBlock(16u64));
			assert_noop!(
				Randomness::request_randomness(request),
				Error::<Test>::InsufficientDeposit
			);
			let request = build_default_request(RequestType::BabeOneEpochAgo(16u64));
			assert_noop!(
				Randomness::request_randomness(request),
				Error::<Test>::InsufficientDeposit
			);
			let request = build_default_request(RequestType::BabeTwoEpochsAgo(16u64));
			assert_noop!(
				Randomness::request_randomness(request),
				Error::<Test>::InsufficientDeposit
			);
			let request = build_default_request(RequestType::Local(16u64));
			assert_noop!(
				Randomness::request_randomness(request),
				Error::<Test>::InsufficientDeposit
			);
		});
}

#[test]
fn request_reserves_deposit_and_fee() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 60)])
		.build()
		.execute_with(|| {
			assert_eq!(Balances::free_balance(&Account::Precompile), 0);
			assert_eq!(Balances::free_balance(&Account::Alice), 60);
			let request = build_default_request(RequestType::BabeCurrentBlock(16u64));
			assert_ok!(Randomness::request_randomness(request));
			assert_eq!(Balances::free_balance(&Account::Precompile), 15);
			assert_eq!(Balances::free_balance(&Account::Alice), 45);
			let request = build_default_request(RequestType::BabeOneEpochAgo(16u64));
			assert_ok!(Randomness::request_randomness(request));
			assert_eq!(Balances::free_balance(&Account::Precompile), 30);
			assert_eq!(Balances::free_balance(&Account::Alice), 30);
			let request = build_default_request(RequestType::BabeTwoEpochsAgo(16u64));
			assert_ok!(Randomness::request_randomness(request));
			assert_eq!(Balances::free_balance(&Account::Precompile), 45);
			assert_eq!(Balances::free_balance(&Account::Alice), 15);
			let request = build_default_request(RequestType::Local(16u64));
			assert_ok!(Randomness::request_randomness(request));
			assert_eq!(Balances::free_balance(&Account::Precompile), 60);
			assert_eq!(Balances::free_balance(&Account::Alice), 0);
		});
}

#[test]
fn request_babe_current_block_randomness_increments_request_counter() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 60)])
		.build()
		.execute_with(|| {
			let request = build_default_request(RequestType::BabeCurrentBlock(16u64));
			assert_eq!(Randomness::request_count(), 0);
			assert_ok!(Randomness::request_randomness(request));
			assert_eq!(Randomness::request_count(), 1);
			let request = build_default_request(RequestType::BabeOneEpochAgo(16u64));
			assert_ok!(Randomness::request_randomness(request));
			assert_eq!(Randomness::request_count(), 2);
			let request = build_default_request(RequestType::BabeTwoEpochsAgo(16u64));
			assert_ok!(Randomness::request_randomness(request));
			assert_eq!(Randomness::request_count(), 3);
			let request = build_default_request(RequestType::Local(16u64));
			assert_ok!(Randomness::request_randomness(request));
			assert_eq!(Randomness::request_count(), 4);
		});
}

#[test]
fn request_babe_current_block_randomness_inserts_request_state() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 60)])
		.build()
		.execute_with(|| {
			let request = build_default_request(RequestType::BabeCurrentBlock(16u64));
			assert_eq!(Randomness::requests(0), None);
			assert_ok!(Randomness::request_randomness(request.clone()));
			assert_eq!(
				Randomness::requests(0),
				Some(RequestState {
					request,
					deposit: 10,
					expires: 6,
				})
			);
			let request = build_default_request(RequestType::BabeOneEpochAgo(16u64));
			assert_eq!(Randomness::requests(1), None);
			assert_ok!(Randomness::request_randomness(request.clone()));
			assert_eq!(
				Randomness::requests(1),
				Some(RequestState {
					request,
					deposit: 10,
					expires: 6,
				})
			);
			let request = build_default_request(RequestType::BabeTwoEpochsAgo(16u64));
			assert_eq!(Randomness::requests(2), None);
			assert_ok!(Randomness::request_randomness(request.clone()));
			assert_eq!(
				Randomness::requests(2),
				Some(RequestState {
					request,
					deposit: 10,
					expires: 6,
				})
			);
			let request = build_default_request(RequestType::Local(16u64));
			assert_eq!(Randomness::requests(3), None);
			assert_ok!(Randomness::request_randomness(request.clone()));
			assert_eq!(
				Randomness::requests(3),
				Some(RequestState {
					request,
					deposit: 10,
					expires: 6,
				})
			);
		});
}

// REQUEST RANDOMNESS EVENTS EMIT BASED ON REQUESTED TYPE OF RANDOMNESS

#[test]
fn request_babe_current_block_randomness_emits_event() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 15)])
		.build()
		.execute_with(|| {
			let request = Request {
				refund_address: Account::Bob.into(),
				contract_address: Account::Alice.into(),
				fee: 5,
				gas_limit: 100u64,
				salt: H256::default(),
				info: RequestType::BabeCurrentBlock(16u64),
			};
			assert_ok!(Randomness::request_randomness(request));
			assert_event_emitted!(crate::Event::RandomnessRequestedCurrentBlock {
				id: 0,
				refund_address: Account::Bob.into(),
				contract_address: Account::Alice.into(),
				fee: 5,
				gas_limit: 100u64,
				salt: H256::default(),
				earliest_block: 16u64,
			});
		});
}

#[test]
fn request_babe_one_epoch_ago_randomness_emits_event() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 15)])
		.build()
		.execute_with(|| {
			let request = Request {
				refund_address: Account::Bob.into(),
				contract_address: Account::Alice.into(),
				fee: 5,
				gas_limit: 100u64,
				salt: H256::default(),
				info: RequestType::BabeOneEpochAgo(16u64),
			};
			assert_ok!(Randomness::request_randomness(request));
			assert_event_emitted!(crate::Event::RandomnessRequestedBabeOneEpochAgo {
				id: 0,
				refund_address: Account::Bob.into(),
				contract_address: Account::Alice.into(),
				fee: 5,
				gas_limit: 100u64,
				salt: H256::default(),
				earliest_epoch: 16u64,
			});
		});
}

#[test]
fn request_babe_two_epochs_ago_randomness_emits_event() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 15)])
		.build()
		.execute_with(|| {
			let request = Request {
				refund_address: Account::Bob.into(),
				contract_address: Account::Alice.into(),
				fee: 5,
				gas_limit: 100u64,
				salt: H256::default(),
				info: RequestType::BabeTwoEpochsAgo(16u64),
			};
			assert_ok!(Randomness::request_randomness(request));
			assert_event_emitted!(crate::Event::RandomnessRequestedBabeTwoEpochsAgo {
				id: 0,
				refund_address: Account::Bob.into(),
				contract_address: Account::Alice.into(),
				fee: 5,
				gas_limit: 100u64,
				salt: H256::default(),
				earliest_epoch: 16u64,
			});
		});
}

#[test]
fn request_local_randomness_emits_event() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 15)])
		.build()
		.execute_with(|| {
			let request = Request {
				refund_address: Account::Bob.into(),
				contract_address: Account::Alice.into(),
				fee: 5,
				gas_limit: 100u64,
				salt: H256::default(),
				info: RequestType::Local(16u64),
			};
			assert_ok!(Randomness::request_randomness(request));
			assert_event_emitted!(crate::Event::RandomnessRequestedLocal {
				id: 0,
				refund_address: Account::Bob.into(),
				contract_address: Account::Alice.into(),
				fee: 5,
				gas_limit: 100u64,
				salt: H256::default(),
				earliest_block: 16u64,
			});
		});
}

#[test]
fn request_randomness_adds_new_randomness_result() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 15)])
		.build()
		.execute_with(|| {
			let request = Request {
				refund_address: Account::Bob.into(),
				contract_address: Account::Alice.into(),
				fee: 5,
				gas_limit: 100u64,
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
		.with_balances(vec![(Account::Alice, 30)])
		.build()
		.execute_with(|| {
			let request = Request {
				refund_address: Account::Bob.into(),
				contract_address: Account::Alice.into(),
				fee: 5,
				gas_limit: 100u64,
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

// #[test]
// fn prepare_fulfillment_fails_before_can_be_fulfilled() {

// }

// #[test]
// fn prepare_fulfillment_uses_randomness_result_without_changing_count() {

// }

// FINISH FULFILLMENT

// finish fulfillment decrements randomness result and will remove it if last
// test both cases separately

// INCREASE REQUEST FEE

// increase request fee updates the request fee

// EXECUTE REQUEST EXPIRATION

// execute request expiration fails before expired

// execute request expiration succeeds

// ON INITIALIZE LOGIC AND HOOKS
