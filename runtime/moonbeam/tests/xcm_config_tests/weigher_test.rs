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

//! Tests for XcmWeigher configuration.
//!
//! The weigher calculates the weight of XCM messages based on the instructions
//! they contain. Moonbeam uses WeightInfoBounds with custom XcmWeight implementation.

use crate::xcm_common::*;
use moonbeam_runtime::{xcm_config::XcmWeigher, RuntimeCall};
use parity_scale_codec::Encode;
use sp_runtime::traits::Zero;
use sp_weights::Weight;
use xcm::latest::prelude::*;
use xcm_executor::traits::WeightBounds;

#[test]
fn weigher_calculates_weight_for_simple_message() {
	ExtBuilder::default().build().execute_with(|| {
		let message: Xcm<RuntimeCall> = Xcm(vec![ClearOrigin]);

		let weight = XcmWeigher::weight(&mut message.clone(), Weight::MAX);

		assert!(weight.is_ok(), "Should calculate weight for simple message");
		let w = weight.unwrap();
		assert!(!w.is_zero(), "Weight should be non-zero");
	});
}

#[test]
fn weigher_calculates_weight_for_transfer_message() {
	ExtBuilder::default().build().execute_with(|| {
		let message: Xcm<RuntimeCall> = Xcm(vec![
			WithdrawAsset((Location::parent(), 1_000_000_000_000u128).into()),
			BuyExecution {
				fees: (Location::parent(), 1_000_000_000_000u128).into(),
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

		let weight = XcmWeigher::weight(&mut message.clone(), Weight::MAX);

		assert!(
			weight.is_ok(),
			"Should calculate weight for transfer message"
		);
		let w = weight.unwrap();
		assert!(w.ref_time() > 0, "Weight ref_time should be positive");
	});
}

#[test]
fn weigher_weight_increases_with_more_instructions() {
	ExtBuilder::default().build().execute_with(|| {
		let simple_message: Xcm<RuntimeCall> = Xcm(vec![ClearOrigin]);

		let complex_message: Xcm<RuntimeCall> = Xcm(vec![
			ClearOrigin,
			ClearOrigin,
			ClearOrigin,
			ClearOrigin,
			ClearOrigin,
		]);

		let simple_weight = XcmWeigher::weight(&mut simple_message.clone(), Weight::MAX).unwrap();
		let complex_weight = XcmWeigher::weight(&mut complex_message.clone(), Weight::MAX).unwrap();

		assert!(
			complex_weight.ref_time() > simple_weight.ref_time(),
			"More instructions should result in higher weight"
		);
	});
}

#[test]
fn weigher_respects_max_instructions() {
	ExtBuilder::default().build().execute_with(|| {
		// MaxInstructions is 100 in xcm_config
		// Create a message with more than 100 instructions
		let instructions: Vec<Instruction<RuntimeCall>> = (0..150).map(|_| ClearOrigin).collect();
		let message: Xcm<RuntimeCall> = Xcm(instructions);

		let weight = XcmWeigher::weight(&mut message.clone(), Weight::MAX);

		// Should fail because it exceeds MaxInstructions
		assert!(
			weight.is_err(),
			"Message exceeding MaxInstructions should fail weighing"
		);
	});
}

#[test]
fn weigher_handles_transact_instruction() {
	ExtBuilder::default().build().execute_with(|| {
		// Transact instruction has variable weight based on the encoded call
		let encoded_call = RuntimeCall::System(frame_system::Call::remark {
			remark: vec![1, 2, 3],
		})
		.encode();
		let message: Xcm<RuntimeCall> = Xcm(vec![Transact {
			origin_kind: OriginKind::Xcm,
			call: encoded_call.into(),
			fallback_max_weight: Some(Weight::from_parts(100_000_000, 10_000)),
		}]);

		let weight = XcmWeigher::weight(&mut message.clone(), Weight::MAX);

		assert!(
			weight.is_ok(),
			"Should calculate weight for Transact instruction"
		);
	});
}
