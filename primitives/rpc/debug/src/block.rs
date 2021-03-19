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

//! Types for the tracing of a all Ethereum transactions of a block.

#[cfg(feature = "std")]
use crate::serialization::*;
#[cfg(feature = "std")]
use serde::{ser::SerializeSeq, Serialize, Serializer};

use codec::{Decode, Encode};
use ethereum_types::{H160, H256, U256};
use sp_std::{collections::btree_map::BTreeMap, vec::Vec};

#[derive(Clone, Eq, PartialEq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Serialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct TransactionTrace {
	#[cfg_attr(feature = "std", serde(flatten))]
	pub action: TransactionTraceAction,
	#[cfg_attr(feature = "std", serde(serialize_with = "h256_serialize"))]
	pub block_hash: H256,
	pub block_number: u32,
	pub result: TransactionTraceResult,
	pub subtraces: u32,
	pub trace_address: Vec<u32>,
	#[cfg_attr(feature = "std", serde(serialize_with = "h256_serialize"))]
	pub transaction_hash: H256,
	pub transaction_position: u32,
}

#[derive(Clone, Eq, PartialEq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Serialize))]
#[cfg_attr(
	feature = "std",
	serde(rename_all = "camelCase", tag = "type", content = "action")
)]
pub enum TransactionTraceAction {
	Call {
		call_type: super::CallType,
		from: H160,
		gas: U256,
		#[cfg_attr(feature = "std", serde(serialize_with = "bytes_0x_serialize"))]
		input: Vec<u8>,
		to: H160,
		value: U256,
	},
	// TODO : Create and SelfDestruct
}

#[derive(Clone, Eq, PartialEq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Serialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct TransactionTraceResult {
	pub gas_used: U256,
	#[cfg_attr(feature = "std", serde(flatten))]
	pub res: crate::CallResult,
}
