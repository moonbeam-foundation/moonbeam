// Copyright 2019-2025 Moonbeam Foundation.
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

//! Tests for XcmBarrier configuration.
//!
//! The barrier determines which XCM messages are allowed to execute.
//! Moonriver's barrier allows:
//! - TakeWeightCredit: Messages that consume credited weight
//! - AllowKnownQueryResponses: Expected query responses
//! - AllowTopLevelPaidExecutionFrom<Everything>: Paid execution from any origin
//! - AllowSubscriptionsFrom<Everything>: Version subscription messages

use crate::xcm_common::*;
use moonriver_runtime::{Runtime, RuntimeCall};
use parity_scale_codec::Encode;
use xcm::latest::prelude::*;
use xcm_executor::traits::QueryHandler;

const ONE_DOT: u128 = 10_000_000_000; // DOT has 10 decimals

#[test]
fn barrier_allows_paid_execution_from_relay() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Location::parent();
		let message: Xcm<RuntimeCall> = Xcm(vec![
			WithdrawAsset((Location::parent(), ONE_DOT).into()),
			BuyExecution {
				fees: (Location::parent(), ONE_DOT).into(),
				weight_limit: WeightLimit::Unlimited,
			},
			DepositAsset {
				assets: Wild(All),
				beneficiary: Location::new(
					0,
					[AccountKey20 {
						network: None,
						key: ALICE,
					}],
				),
			},
		]);

		let outcome = execute_xcm(origin, message);
		// Should not be blocked by barrier (may fail later due to no funds, but not barrier)
		assert!(!is_barrier_error(&outcome));
	});
}

#[test]
fn barrier_allows_paid_execution_from_sibling() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Location::new(1, [Parachain(2000)]);
		let message: Xcm<RuntimeCall> = Xcm(vec![
			WithdrawAsset((Location::parent(), ONE_DOT).into()),
			BuyExecution {
				fees: (Location::parent(), ONE_DOT).into(),
				weight_limit: WeightLimit::Unlimited,
			},
			DepositAsset {
				assets: Wild(All),
				beneficiary: Location::new(
					0,
					[AccountKey20 {
						network: None,
						key: BOB,
					}],
				),
			},
		]);

		let outcome = execute_xcm(origin, message);
		assert!(!is_barrier_error(&outcome));
	});
}

#[test]
fn barrier_passes_unpaid_with_weight_credit() {
	ExtBuilder::default().build().execute_with(|| {
		// TakeWeightCredit is the first barrier and passes when the message
		// weight is within the pre-credited amount.  `execute_xcm` passes
		// Weight::zero() as credit, so unpaid messages are rejected there.
		// Use `execute_xcm_with_credit` with a generous credit instead.
		let origin = Location::parent();
		let message: Xcm<RuntimeCall> = Xcm(vec![DepositAsset {
			assets: Wild(All),
			beneficiary: Location::new(
				0,
				[AccountKey20 {
					network: None,
					key: ALICE,
				}],
			),
		}]);

		let outcome = execute_xcm_with_credit(origin, message, Weight::MAX);
		// TakeWeightCredit allows this to pass the barrier (may fail later for other reasons)
		assert!(
			!is_barrier_error(&outcome),
			"TakeWeightCredit should allow messages with credited weight"
		);
	});
}

#[test]
fn barrier_allows_subscription_messages() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Location::parent();
		// SubscribeVersion is allowed by AllowSubscriptionsFrom
		let message: Xcm<RuntimeCall> = Xcm(vec![SubscribeVersion {
			query_id: 0,
			max_response_weight: Weight::from_parts(1_000_000, 64 * 1024),
		}]);

		let outcome = execute_xcm(origin, message);
		// Should not be a barrier error - subscriptions are allowed
		assert!(!is_barrier_error(&outcome));
	});
}

#[test]
fn barrier_allows_unsubscribe_messages() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Location::parent();
		let message: Xcm<RuntimeCall> = Xcm(vec![UnsubscribeVersion]);

		let outcome = execute_xcm(origin, message);
		assert!(!is_barrier_error(&outcome));
	});
}

#[test]
fn barrier_allows_paid_execution_with_descend_origin() {
	ExtBuilder::default().build().execute_with(|| {
		// Test that WithComputedOrigin allows descending origin
		let origin = Location::parent();
		let message: Xcm<RuntimeCall> = Xcm(vec![
			DescendOrigin(
				[AccountId32 {
					network: None,
					id: [1u8; 32],
				}]
				.into(),
			),
			WithdrawAsset((Location::parent(), ONE_DOT).into()),
			BuyExecution {
				fees: (Location::parent(), ONE_DOT).into(),
				weight_limit: WeightLimit::Unlimited,
			},
			DepositAsset {
				assets: Wild(All),
				beneficiary: Location::new(
					0,
					[AccountKey20 {
						network: None,
						key: ALICE,
					}],
				),
			},
		]);

		let outcome = execute_xcm(origin, message);
		assert!(!is_barrier_error(&outcome));
	});
}

#[test]
fn barrier_allows_set_topic() {
	ExtBuilder::default().build().execute_with(|| {
		// SetTopic is wrapped by TrailingSetTopicAsId so should be handled
		let origin = Location::parent();
		let message: Xcm<RuntimeCall> = Xcm(vec![
			WithdrawAsset((Location::parent(), ONE_DOT).into()),
			BuyExecution {
				fees: (Location::parent(), ONE_DOT).into(),
				weight_limit: WeightLimit::Unlimited,
			},
			SetTopic([0u8; 32]),
		]);

		let outcome = execute_xcm(origin, message);
		assert!(!is_barrier_error(&outcome));
	});
}

#[test]
fn barrier_with_computed_origin_rejects_when_depth_limit_exceeded() {
	ExtBuilder::default().build().execute_with(|| {
		// WithComputedOrigin is configured with ConstU32<8>, meaning it will skip at most 8
		// DescendOrigin (or similar) instructions when computing the origin. If the message
		// contains more than 8 such instructions, WithComputedOrigin cannot reach the inner
		// barriers (AllowTopLevelPaidExecutionFrom, AllowSubscriptionsFrom) and the message
		// is rejected.
		let origin = Location::parent();

		// Build a message with 9 DescendOrigin instructions, exceeding the limit of 8
		let mut instructions: Vec<Instruction<RuntimeCall>> = Vec::new();
		for i in 0..9 {
			instructions.push(DescendOrigin(
				[AccountId32 {
					network: None,
					id: [i as u8; 32],
				}]
				.into(),
			));
		}
		instructions.push(WithdrawAsset((Location::parent(), ONE_DOT).into()));
		instructions.push(BuyExecution {
			fees: (Location::parent(), ONE_DOT).into(),
			weight_limit: WeightLimit::Unlimited,
		});

		let message: Xcm<RuntimeCall> = Xcm(instructions);
		let outcome = execute_xcm(origin, message);
		assert!(
			is_barrier_error(&outcome),
			"Message exceeding WithComputedOrigin depth limit of 8 should be rejected"
		);
	});
}

#[test]
fn barrier_with_computed_origin_allows_at_depth_limit() {
	ExtBuilder::default().build().execute_with(|| {
		// WithComputedOrigin is configured with ConstU32<8>. A message with exactly 8
		// DescendOrigin instructions should still be processed by the inner barriers.
		let origin = Location::parent();

		let mut instructions: Vec<Instruction<RuntimeCall>> = Vec::new();
		for i in 0..8 {
			instructions.push(DescendOrigin(
				[AccountId32 {
					network: None,
					id: [i as u8; 32],
				}]
				.into(),
			));
		}
		instructions.push(WithdrawAsset((Location::parent(), ONE_DOT).into()));
		instructions.push(BuyExecution {
			fees: (Location::parent(), ONE_DOT).into(),
			weight_limit: WeightLimit::Unlimited,
		});

		let message: Xcm<RuntimeCall> = Xcm(instructions);
		let outcome = execute_xcm(origin, message);
		// Should pass the barrier (may fail later for other reasons like no funds)
		assert!(
			!is_barrier_error(&outcome),
			"Message within WithComputedOrigin depth limit of 8 should pass the barrier"
		);
	});
}

#[test]
fn barrier_allows_paid_execution_from_account_key20() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Location::new(
			0,
			[AccountKey20 {
				network: Some(NetworkId::Polkadot),
				key: ALICE,
			}],
		);
		let message: Xcm<RuntimeCall> = Xcm(vec![
			WithdrawAsset((Location::parent(), ONE_DOT).into()),
			BuyExecution {
				fees: (Location::parent(), ONE_DOT).into(),
				weight_limit: WeightLimit::Unlimited,
			},
			DepositAsset {
				assets: Wild(All),
				beneficiary: Location::new(
					0,
					[AccountKey20 {
						network: None,
						key: BOB,
					}],
				),
			},
		]);

		let outcome = execute_xcm(origin, message);
		assert!(
			!is_barrier_error(&outcome),
			"Paid execution from AccountKey20 should pass the barrier"
		);
	});
}

#[test]
fn barrier_rejects_unpaid_execution_from_sibling() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Location::new(1, [Parachain(2000)]);
		let message: Xcm<RuntimeCall> = Xcm(vec![
			UnpaidExecution {
				weight_limit: Unlimited,
				check_origin: None,
			},
			DepositAsset {
				assets: Wild(All),
				beneficiary: Location::new(
					0,
					[AccountKey20 {
						network: None,
						key: ALICE,
					}],
				),
			},
		]);

		let outcome = execute_xcm(origin, message);
		assert!(
			is_barrier_error(&outcome),
			"UnpaidExecution from sibling should be rejected by barrier"
		);
	});
}

#[test]
fn barrier_rejects_unpaid_transact_from_sibling() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Location::new(1, [Parachain(2000)]);
		let encoded_call = RuntimeCall::System(frame_system::Call::remark_with_event {
			remark: vec![1, 2, 3],
		})
		.encode();

		let message: Xcm<RuntimeCall> = Xcm(vec![
			UnpaidExecution {
				weight_limit: Unlimited,
				check_origin: None,
			},
			Transact {
				origin_kind: OriginKind::SovereignAccount,
				call: encoded_call.into(),
				fallback_max_weight: Some(Weight::from_parts(100_000_000, 10_000)),
			},
		]);

		let outcome = execute_xcm(origin, message);
		assert!(
			is_barrier_error(&outcome),
			"UnpaidExecution + Transact from sibling should be rejected"
		);
	});
}

#[test]
fn barrier_allows_known_query_response() {
	ExtBuilder::default().build().execute_with(|| {
		let relay_origin = Location::parent();

		let query_id = pallet_xcm::Pallet::<Runtime>::new_query(
			relay_origin.clone(),
			100u32.into(),
			Location::here(),
		);

		let message: Xcm<RuntimeCall> = Xcm(vec![QueryResponse {
			query_id,
			response: Response::Null,
			max_weight: Weight::from_parts(1_000_000, 64 * 1024),
			querier: Some(Location::here()),
		}]);

		let outcome = execute_xcm(relay_origin, message);
		assert!(
			!is_barrier_error(&outcome),
			"Known query response should pass AllowKnownQueryResponses barrier"
		);
	});
}

#[test]
fn barrier_rejects_unknown_query_response() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Location::parent();
		let unknown_query_id = 999_999u64;

		let message: Xcm<RuntimeCall> = Xcm(vec![QueryResponse {
			query_id: unknown_query_id,
			response: Response::Null,
			max_weight: Weight::from_parts(1_000_000, 64 * 1024),
			querier: Some(Location::here()),
		}]);

		let outcome = execute_xcm(origin, message);
		assert!(
			is_barrier_error(&outcome),
			"Unknown query response should be rejected by barrier"
		);
	});
}
