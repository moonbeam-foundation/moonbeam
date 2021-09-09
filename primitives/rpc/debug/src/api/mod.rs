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

//! Runtime API allowing to debug/trace Ethereum

use codec::{Decode, Encode};
use ethereum_types::H160;
use sp_std::vec::Vec;

#[cfg(feature = "std")]
use crate::serialization::*;
#[cfg(feature = "std")]
use serde::Serialize;

pub mod block;
pub mod single;

pub const MANUAL_BLOCK_INITIALIZATION_RUNTIME_VERSION: u32 = 159;

#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Serialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub enum CallResult {
	Output(#[cfg_attr(feature = "std", serde(serialize_with = "bytes_0x_serialize"))] Vec<u8>),
	// field "error"
	Error(#[cfg_attr(feature = "std", serde(serialize_with = "string_serialize"))] Vec<u8>),
}

#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Serialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase", untagged))]
pub enum CreateResult {
	Error {
		#[cfg_attr(feature = "std", serde(serialize_with = "string_serialize"))]
		error: Vec<u8>,
	},
	Success {
		#[cfg_attr(feature = "std", serde(rename = "createdContractAddressHash"))]
		created_contract_address_hash: H160,
		#[cfg_attr(
			feature = "std",
			serde(serialize_with = "bytes_0x_serialize", rename = "createdContractCode")
		)]
		created_contract_code: Vec<u8>,
	},
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Serialize))]
#[cfg_attr(feature = "std", serde(rename_all = "lowercase"))]
pub enum CallType {
	Call,
	CallCode,
	DelegateCall,
	StaticCall,
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Serialize))]
#[cfg_attr(feature = "std", serde(rename_all = "lowercase"))]
pub enum CreateType {
	Create,
}
