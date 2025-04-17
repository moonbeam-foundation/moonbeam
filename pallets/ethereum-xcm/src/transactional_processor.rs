// Copyright 2019-2025 PureStake Inc.
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
// along with Moonbeam.  If not, see <http://www.gnu.org/licenses/>.$

use frame_support::storage::{with_transaction, TransactionOutcome};
use sp_runtime::DispatchError;
use xcm::latest::prelude::*;
use xcm_executor::traits::ProcessTransaction;

environmental::environmental!(IS_EVM_REVERT: bool);

/// Transactional processor implementation used by the XCM executor
/// to execute each XCM instruction in a transactional way.
///
/// Behave like FrameTransactionalProcessor except if the XCM instruction call the EVM AND the EVM Revert has occurred.
/// In this case, the storage changes should be committed to include the eth-xcm transaction in the "ethereum block storage".
pub struct XcmEthTransactionalProcessor;

impl XcmEthTransactionalProcessor {
	pub fn signal_evm_revert() {
		IS_EVM_REVERT::with(|is_evm_revert| *is_evm_revert = true);
	}
}

impl ProcessTransaction for XcmEthTransactionalProcessor {
	const IS_TRANSACTIONAL: bool = true;

	fn process<F>(f: F) -> Result<(), XcmError>
	where
		F: FnOnce() -> Result<(), XcmError>,
	{
		IS_EVM_REVERT::using(&mut false, || {
			with_transaction(|| -> TransactionOutcome<Result<_, DispatchError>> {
				let output = f();
				match &output {
					Ok(()) => TransactionOutcome::Commit(Ok(output)),
					Err(xcm_error) => {
						// If the XCM instruction failed from an EVM revert,
						// we should not rollback storage change
						if let Some(true) = IS_EVM_REVERT::with(|is_evm_revert| *is_evm_revert) {
							TransactionOutcome::Commit(Ok(output))
						} else {
							// Otherwise, we should rollback storage changes
							// to be consistent with FrameTransactionalProcessor
							TransactionOutcome::Rollback(Ok(Err(*xcm_error)))
						}
					}
				}
			})
			.map_err(|_| XcmError::ExceedsStackLimit)?
		})
	}
}
