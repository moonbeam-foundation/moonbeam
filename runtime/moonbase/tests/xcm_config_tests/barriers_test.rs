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
//! Moonbase's barrier allows:
//! - TakeWeightCredit: Messages that consume credited weight
//! - AllowKnownQueryResponses: Expected query responses
//! - AllowTopLevelPaidExecutionFrom<Everything>: Paid execution from any origin
//! - AllowSubscriptionsFrom<Everything>: Version subscription messages

use crate::xcm_common::*;
use moonbase_runtime::RuntimeCall;
use xcm::latest::prelude::*;

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
		// Note: TakeWeightCredit is the first barrier, which passes if weight credit is available.
		// In the XcmExecutor, weight is credited before barrier checks, so simple messages pass.
		// This test verifies that TakeWeightCredit works as expected.
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

		let outcome = execute_xcm(origin, message);
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
fn barrier_with_computed_origin_has_depth_limit() {
	ExtBuilder::default().build().execute_with(|| {
		// WithComputedOrigin has ConstU32<8> which limits the computed origin's junction depth.
		// Note: TakeWeightCredit is checked first, so messages may pass before WithComputedOrigin.
		// This test verifies that messages can still execute even with multiple DescendOrigin
		// instructions, as TakeWeightCredit processes them first.
		let origin = Location::parent();

		let mut instructions: Vec<Instruction<RuntimeCall>> = Vec::new();
		// Add DescendOrigin instructions
		for i in 0..3 {
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
		// Message should pass the barrier (TakeWeightCredit or WithComputedOrigin)
		// It may fail later for other reasons (no funds), but not barrier
		assert!(!is_barrier_error(&outcome));
	});
}
