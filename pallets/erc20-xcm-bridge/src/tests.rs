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

//! Unit testing
use sp_runtime::BoundedVec;
use xcm::latest::Junction;

use crate::mock::{Erc20XcmBridge, Erc20XcmBridgeTransferGasLimit};

#[test]
fn general_key_data_size_32() {
	let junction: Junction = (BoundedVec::new()).into();

	// Assert that GeneralKey data length is 32 bytes
	match junction {
		Junction::GeneralKey { length: _, data } => {
			let _: [u8; 32] = data;
		}
		_ => assert!(false),
	}

	assert_eq!(
		Erc20XcmBridge::gas_limit_of_erc20_transfer(&junction.into()),
		Erc20XcmBridgeTransferGasLimit::get()
	)
}

#[test]
fn gas_limit_override() {
	let text = "gas_limit:".as_bytes();
	let limit = 300_000u64;
	let data = [text, &limit.to_le_bytes()].concat();
	let vec = BoundedVec::try_from(data).expect("vec should convert");
	let junction: Junction = (vec).into();
	assert_eq!(
		Erc20XcmBridge::gas_limit_of_erc20_transfer(&junction.into()),
		limit
	)
}

#[test]
fn gas_limit_override_typo() {
	let text = "gaslimit:".as_bytes();
	let limit = 300_000u64;
	let data = [text, &limit.to_le_bytes()].concat();
	let vec = BoundedVec::try_from(data).expect("vec should convert");
	let junction: Junction = (vec).into();
	assert_eq!(
		Erc20XcmBridge::gas_limit_of_erc20_transfer(&junction.into()),
		Erc20XcmBridgeTransferGasLimit::get()
	)
}
