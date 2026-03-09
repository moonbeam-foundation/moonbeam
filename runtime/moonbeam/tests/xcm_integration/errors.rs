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

//! XCM error handling integration tests.
//!
//! Tests for error scenarios and edge cases:
//! - Unknown asset handling
//! - Insufficient fees
//! - Invalid origins

use crate::common::*;
use crate::networks::*;
use moonbeam_runtime::{xcm_config::XcmExecutorConfig, RuntimeCall};
use parity_scale_codec::Encode;
use sp_weights::Weight;
use xcm::latest::prelude::*;
use xcm_executor::XcmExecutor;

fn execute_xcm_message(origin: Location, message: Xcm<RuntimeCall>) -> Outcome {
	let hash = message.using_encoded(sp_io::hashing::blake2_256);
	XcmExecutor::<XcmExecutorConfig>::prepare_and_execute(
		origin,
		message,
		&mut hash.clone(),
		Weight::MAX,
		Weight::zero(),
	)
}

#[test]
fn error_on_unknown_asset_deposit() {
	moonbeam_execute_with(|| {
		let origin = Location::parent();

		// Try to deposit an unknown asset
		let unknown_asset = Asset {
			id: AssetId(Location::new(1, [Parachain(9999), PalletInstance(99)])),
			fun: Fungible(1_000_000),
		};

		let message: Xcm<RuntimeCall> = Xcm(vec![
			WithdrawAsset(unknown_asset.clone().into()),
			BuyExecution {
				fees: unknown_asset.clone(),
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

		let outcome = execute_xcm_message(origin, message);

		// Should fail - unknown asset
		assert!(
			!matches!(outcome, Outcome::Complete { .. }),
			"Unknown asset should cause error"
		);
	});
}

#[test]
fn error_on_barrier_rejection() {
	moonbeam_execute_with(|| {
		let origin = Location::parent();

		// Message without BuyExecution - should be blocked by barrier
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

		let outcome = execute_xcm_message(origin, message);

		// Should fail with barrier error
		// The executor reports Incomplete (not Error) because it begins processing
		// before the barrier rejects at instruction index 0.
		assert!(
			matches!(
				outcome,
				Outcome::Incomplete { ref error, .. } if error.error == XcmError::Barrier
			),
			"Unpaid execution should be blocked by barrier, got: {:?}",
			outcome
		);
	});
}

#[test]
fn error_on_too_expensive_execution() {
	moonbeam_execute_with(|| {
		let origin = Location::parent();

		// Message with tiny fee amount for large weight
		let tiny_fee = Asset {
			id: AssetId(Location::parent()),
			fun: Fungible(1), // Very small amount
		};

		let message: Xcm<RuntimeCall> = Xcm(vec![
			WithdrawAsset(tiny_fee.clone().into()),
			BuyExecution {
				fees: tiny_fee,
				weight_limit: WeightLimit::Limited(Weight::from_parts(
					1_000_000_000_000,
					1024 * 1024,
				)),
			},
		]);

		let outcome = execute_xcm_message(origin, message);

		// Execution cost exceeds available funds
		// May fail with TooExpensive or other asset-related error
		assert!(
			!matches!(outcome, Outcome::Complete { .. }),
			"Should fail when fees are insufficient"
		);
	});
}
