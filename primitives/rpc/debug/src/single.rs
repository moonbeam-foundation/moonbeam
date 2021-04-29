// Copyright 2019-2020 PureStake Inc.
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

//! Types for the tracing of a single Ethereum transaction.
//! Structure from "raw" debug_trace and a "call list" matching
//! Blockscout formatter. This "call list" is also used to build
//! the whole block tracing output.

#[cfg(feature = "std")]
use crate::serialization::*;
#[cfg(feature = "std")]
use serde::Serialize;

use codec::{Decode, Encode};
use ethereum_types::{H160, H256, U256};
use sp_std::{collections::btree_map::BTreeMap, vec::Vec};

#[derive(Clone, Copy, Eq, PartialEq, Debug, Encode, Decode)]
pub enum TraceType {
	/// Classic geth with no javascript based tracing.
	Raw {
		disable_storage: bool,
		disable_memory: bool,
		disable_stack: bool,
	},
	/// List of calls and subcalls (output Blockscout expects).
	CallList,
}

/// Single transaction trace.
#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Serialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase", untagged))]
pub enum TransactionTrace {
	/// Classical output of `debug_trace`.
	#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
	Raw {
		gas: U256,
		#[cfg_attr(feature = "std", serde(with = "hex"))]
		return_value: Vec<u8>,
		step_logs: Vec<RawStepLog>,
	},
	/// Matches the formatter used by Blockscout.
	/// Is also used to built output of OpenEthereum's `trace_filter`.
	CallList(Vec<Call>),
}

#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Serialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct RawStepLog {
	#[cfg_attr(feature = "std", serde(serialize_with = "u256_serialize"))]
	pub depth: U256,

	//error: TODO
	#[cfg_attr(feature = "std", serde(serialize_with = "u256_serialize"))]
	pub gas: U256,

	#[cfg_attr(feature = "std", serde(serialize_with = "u256_serialize"))]
	pub gas_cost: U256,

	#[cfg_attr(
		feature = "std",
		serde(
			serialize_with = "seq_h256_serialize",
			skip_serializing_if = "Option::is_none"
		)
	)]
	pub memory: Option<Vec<H256>>,

	#[cfg_attr(feature = "std", serde(serialize_with = "opcode_serialize"))]
	pub op: Vec<u8>,

	#[cfg_attr(feature = "std", serde(serialize_with = "u256_serialize"))]
	pub pc: U256,

	#[cfg_attr(
		feature = "std",
		serde(
			serialize_with = "seq_h256_serialize",
			skip_serializing_if = "Option::is_none"
		)
	)]
	pub stack: Option<Vec<H256>>,

	#[cfg_attr(feature = "std", serde(skip_serializing_if = "Option::is_none"))]
	pub storage: Option<BTreeMap<H256, H256>>,
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
