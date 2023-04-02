// Copyright 2019-2022 PureStake Inc.
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

use super::serialization::*;
use serde::Serialize;

use ethereum_types::{H256, U256};
use parity_scale_codec::{Decode, Encode};
use sp_std::{collections::btree_map::BTreeMap, vec::Vec};

#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode, Serialize)]
#[serde(rename_all = "camelCase", untagged)]
pub enum Call {
	Blockscout(crate::formatters::blockscout::BlockscoutCall),
	CallTracer(crate::formatters::call_tracer::CallTracerCall),
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, Encode, Decode)]
pub enum TraceType {
	/// Classic geth with no javascript based tracing.
	Raw {
		disable_storage: bool,
		disable_memory: bool,
		disable_stack: bool,
	},
	/// List of calls and subcalls formatted with an input tracer (i.e. callTracer or Blockscout).
	CallList,
	/// A single block trace. Use in `debug_traceTransactionByNumber` / `traceTransactionByHash`.
	Block,
}

/// Single transaction trace.
#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode, Serialize)]
#[serde(rename_all = "camelCase", untagged)]
pub enum TransactionTrace {
	/// Classical output of `debug_trace`.
	#[serde(rename_all = "camelCase")]
	Raw {
		gas: U256,
		#[serde(with = "hex")]
		return_value: Vec<u8>,
		struct_logs: Vec<RawStepLog>,
	},
	/// Matches the formatter used by Blockscout.
	/// Is also used to built output of OpenEthereum's `trace_filter`.
	CallList(Vec<Call>),
	/// Used by Geth's callTracer.
	CallListNested(Call),
}

#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RawStepLog {
	#[serde(serialize_with = "u256_serialize")]
	pub depth: U256,

	//error: TODO
	#[serde(serialize_with = "u256_serialize")]
	pub gas: U256,

	#[serde(serialize_with = "u256_serialize")]
	pub gas_cost: U256,

	#[serde(
		serialize_with = "seq_h256_serialize",
		skip_serializing_if = "Option::is_none"
	)]
	pub memory: Option<Vec<H256>>,

	#[serde(serialize_with = "opcode_serialize")]
	pub op: Vec<u8>,

	#[serde(serialize_with = "u256_serialize")]
	pub pc: U256,

	#[serde(
		serialize_with = "seq_h256_serialize",
		skip_serializing_if = "Option::is_none"
	)]
	pub stack: Option<Vec<H256>>,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub storage: Option<BTreeMap<H256, H256>>,
}
