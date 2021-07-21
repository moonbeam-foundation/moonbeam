// Copyright 2019-2021 PureStake Inc.
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

use evm::ExitError;
use sp_core::H160;

/// Parses an H160 account address from a 256 bit (32 byte) buffer. Only the last 20 bytes are used.
pub fn parse_account(input: &[u8]) -> Result<H160, ExitError> {
	const PADDING_SIZE_BYTES: usize = 12;
	const ACCOUNT_SIZE_BYTES: usize = 20;
	const TOTAL_SIZE_BYTES: usize = PADDING_SIZE_BYTES + ACCOUNT_SIZE_BYTES;

	if input.len() != TOTAL_SIZE_BYTES {
		return Err(ExitError::Other(
			"Incorrect input length for account parsing".into(),
		));
	}

	Ok(H160::from_slice(
		&input[PADDING_SIZE_BYTES..TOTAL_SIZE_BYTES],
	))
}
