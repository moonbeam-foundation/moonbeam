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

//! Blockscout specific responses.

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
	/// Indices of parent calls.
	pub trace_address: Vec<u32>,
	/// Number of children calls.
	/// Not needed for Blockscout, but needed for `crate::block`
	/// types that are build from this type.
	#[cfg_attr(feature = "std", serde(skip))]
	pub subtraces: u32,
	/// Sends funds to the (payable) function
	pub value: U256,
	/// Remaining gas in the runtime.
	pub gas: U256,
	/// Gas used by this context.
	pub gas_used: U256,
	#[cfg_attr(feature = "std", serde(flatten))]
	pub inner: CallInner,
}

#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Serialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase", tag = "type"))]
pub enum CallInner {
	#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
	Call {
		/// Type of call.
		call_type: crate::CallType,
		to: H160,
		#[cfg_attr(feature = "std", serde(serialize_with = "bytes_0x_serialize"))]
		input: Vec<u8>,
		/// "output" or "error" field
		#[cfg_attr(feature = "std", serde(flatten))]
		res: crate::CallResult,
	},

	#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
	Create {
		#[cfg_attr(feature = "std", serde(serialize_with = "bytes_0x_serialize"))]
		init: Vec<u8>,
		#[cfg_attr(feature = "std", serde(flatten))]
		res: crate::CreateResult,
	},
	// Revert,
	SelfDestruct {
		#[cfg_attr(feature = "std", serde(skip))]
		balance: U256,
		#[cfg_attr(feature = "std", serde(skip))]
		refund_address: H160,
	},
}
