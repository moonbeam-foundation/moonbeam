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

//! Common test utilities for XCM configuration tests.

#![allow(dead_code)]

pub use crate::common::*;

use moonriver_runtime::{xcm_config::XcmExecutorConfig, RuntimeCall};
use parity_scale_codec::Encode;
use sp_weights::Weight;
use xcm::latest::prelude::*;
use xcm_executor::XcmExecutor;

/// Execute an XCM message and return the outcome.
///
/// This uses the real Moonriver XcmExecutorConfig to test XCM behavior.
pub fn execute_xcm(origin: Location, message: Xcm<RuntimeCall>) -> Outcome {
	let hash = message.using_encoded(sp_io::hashing::blake2_256);
	XcmExecutor::<XcmExecutorConfig>::prepare_and_execute(
		origin,
		message,
		&mut hash.clone(),
		Weight::MAX,
		Weight::zero(),
	)
}

/// Execute an XCM message with a weight limit and return the outcome.
pub fn execute_xcm_with_weight(
	origin: Location,
	message: Xcm<RuntimeCall>,
	weight_limit: Weight,
) -> Outcome {
	let hash = message.using_encoded(sp_io::hashing::blake2_256);
	XcmExecutor::<XcmExecutorConfig>::prepare_and_execute(
		origin,
		message,
		&mut hash.clone(),
		weight_limit,
		Weight::zero(),
	)
}

/// Execute an XCM message with pre-credited weight.
///
/// `TakeWeightCredit` barrier passes when the message weight is within the
/// credited amount; use this to test that barrier path.
pub fn execute_xcm_with_credit(
	origin: Location,
	message: Xcm<RuntimeCall>,
	weight_credit: Weight,
) -> Outcome {
	let hash = message.using_encoded(sp_io::hashing::blake2_256);
	XcmExecutor::<XcmExecutorConfig>::prepare_and_execute(
		origin,
		message,
		&mut hash.clone(),
		Weight::MAX,
		weight_credit,
	)
}

/// Helper to check if an outcome is a barrier error.
///
/// Barrier rejections can surface as either `Outcome::Error` or
/// `Outcome::Incomplete` (when the executor begins processing before the
/// barrier rejects at instruction index 0), so both variants are matched.
pub fn is_barrier_error(outcome: &Outcome) -> bool {
	matches!(
		outcome,
		Outcome::Error(ref e) if e.error == XcmError::Barrier
	) || matches!(
		outcome,
		Outcome::Incomplete { ref error, .. } if error.error == XcmError::Barrier
	)
}

/// Helper to check if execution completed successfully
pub fn is_complete(outcome: &Outcome) -> bool {
	matches!(outcome, Outcome::Complete { .. })
}

/// Helper to check if execution is incomplete (partially executed)
pub fn is_incomplete(outcome: &Outcome) -> bool {
	matches!(outcome, Outcome::Incomplete { .. })
}

/// Create a simple asset from location and amount
pub fn asset(location: Location, amount: u128) -> Asset {
	Asset {
		id: AssetId(location),
		fun: Fungible(amount),
	}
}

/// Relay chain native asset (DOT)
pub fn relay_asset(amount: u128) -> Asset {
	asset(Location::parent(), amount)
}

/// Asset from a sibling parachain's native token
pub fn sibling_asset(para_id: u32, pallet_index: u8, amount: u128) -> Asset {
	asset(
		Location::new(1, [Parachain(para_id), PalletInstance(pallet_index)]),
		amount,
	)
}

/// Build a message with paid execution
pub fn paid_message(fees: Asset, instructions: Vec<Instruction<RuntimeCall>>) -> Xcm<RuntimeCall> {
	let mut all_instructions = vec![
		WithdrawAsset(fees.clone().into()),
		BuyExecution {
			fees,
			weight_limit: WeightLimit::Unlimited,
		},
	];
	all_instructions.extend(instructions);
	Xcm(all_instructions)
}

/// Build a deposit asset instruction to an account
pub fn deposit_to_account(account: [u8; 20]) -> Instruction<RuntimeCall> {
	DepositAsset {
		assets: Wild(All),
		beneficiary: Location::new(
			0,
			[AccountKey20 {
				network: None,
				key: account,
			}],
		),
	}
}
