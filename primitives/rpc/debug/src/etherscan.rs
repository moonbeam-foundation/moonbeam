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

//! Etherscan specific responses.

#[cfg(feature = "std")]
use crate::serialization::*;
#[cfg(feature = "std")]
use serde::Serialize;

use codec::{Decode, Encode};
use ethereum_types::{H160, U256};
use sp_std::vec::Vec;

#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Serialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct Call {
	pub from: H160,

	/// Indices of parent calls. Used to build the Etherscan nested response.
	#[cfg_attr(feature = "std", serde(skip_serializing_if = "Option::is_none"))]
	pub trace_address: Option<Vec<u32>>,

	/// Remaining gas in the runtime.
	pub gas: U256,
	/// Gas used by this context.
	pub gas_used: U256,

	#[cfg_attr(feature = "std", serde(flatten))]
	pub inner: CallInner,

	#[cfg_attr(feature = "std", serde(skip_serializing_if = "Vec::is_empty"))]
	pub calls: Vec<crate::single::Call>,
}

#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Serialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase", untagged))]
pub enum CallInner {
	#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
	Call {
		#[cfg_attr(
			feature = "std",
			serde(rename = "type", serialize_with = "opcode_serialize")
		)]
		call_type: Vec<u8>,
		to: H160,
		#[cfg_attr(feature = "std", serde(serialize_with = "bytes_0x_serialize"))]
		input: Vec<u8>,
		/// "output" or "error" field
		#[cfg_attr(feature = "std", serde(flatten))]
		res: crate::CallResult,

		#[cfg_attr(feature = "std", serde(skip_serializing_if = "Option::is_none"))]
		value: Option<U256>,
	},

	#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
	Create {
		#[cfg_attr(
			feature = "std",
			serde(rename = "type", serialize_with = "opcode_serialize")
		)]
		call_type: Vec<u8>,
		#[cfg_attr(feature = "std", serde(serialize_with = "bytes_0x_serialize"))]
		input: Vec<u8>,
		#[cfg_attr(feature = "std", serde(skip_serializing_if = "Option::is_none"))]
		to: Option<H160>,
		#[cfg_attr(
			feature = "std",
			serde(
				skip_serializing_if = "Option::is_none",
				serialize_with = "option_bytes_0x_serialize"
			)
		)]
		output: Option<Vec<u8>>,
		#[cfg_attr(
			feature = "std",
			serde(
				skip_serializing_if = "Option::is_none",
				serialize_with = "option_string_serialize"
			)
		)]
		error: Option<Vec<u8>>,
		value: U256,
	},
	// Revert,
	SelfDestruct {
		#[cfg_attr(
			feature = "std",
			serde(rename = "type", serialize_with = "opcode_serialize")
		)]
		call_type: Vec<u8>,
		#[cfg_attr(feature = "std", serde(skip))]
		to: H160,
		value: U256,
	},
}
