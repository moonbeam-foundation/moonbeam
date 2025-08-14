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
// along with Moonbeam.  If not, see <http://www.gnu.org/licenses/>.

use sp_runtime::DispatchError;
use xcm::latest::Error as XcmError;

pub(crate) enum Erc20TransferError {
	ContractTransferFail,
	ContractReturnInvalidValue,
	DispatchError(DispatchError),
	EvmCallFail,
}

impl From<DispatchError> for Erc20TransferError {
	fn from(e: DispatchError) -> Self {
		Self::DispatchError(e)
	}
}

impl From<Erc20TransferError> for XcmError {
	fn from(error: Erc20TransferError) -> XcmError {
		match error {
			Erc20TransferError::ContractTransferFail => {
				XcmError::FailedToTransactAsset("Erc20 contract transfer fail")
			}
			Erc20TransferError::ContractReturnInvalidValue => {
				XcmError::FailedToTransactAsset("Erc20 contract return invalid value")
			}
			Erc20TransferError::DispatchError(err) => {
				log::debug!("dispatch error: {:?}", err);
				Self::FailedToTransactAsset("storage layer error")
			}
			Erc20TransferError::EvmCallFail => {
				XcmError::FailedToTransactAsset("Fail to call erc20 contract")
			}
		}
	}
}
