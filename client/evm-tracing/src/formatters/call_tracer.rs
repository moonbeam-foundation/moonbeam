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

use super::blockscout::BlockscoutCallInner;
use crate::types::{
	single::{Call, TransactionTrace},
	CallResult, CallType, CreateResult,
};

use crate::listeners::call_list::Listener;

use crate::types::serialization::*;
use serde::Serialize;

use ethereum_types::{H160, U256};
use parity_scale_codec::{Decode, Encode};
use sp_std::{cmp::Ordering, vec::Vec};

pub struct Formatter;

impl super::ResponseFormatter for Formatter {
	type Listener = Listener;
	type Response = Vec<TransactionTrace>;

	fn format(mut listener: Listener) -> Option<Vec<TransactionTrace>> {
		// Remove empty BTreeMaps pushed to `entries`.
		// I.e. InvalidNonce or other pallet_evm::runner exits
		listener.entries.retain(|x| !x.is_empty());
		let mut traces = Vec::new();
		for entry in listener.entries.iter() {
			let mut result: Vec<Call> = entry
				.into_iter()
				.map(|(_, it)| {
					let from = it.from;
					let trace_address = it.trace_address.clone();
					let value = it.value;
					let gas = it.gas;
					let gas_used = it.gas_used;
					let inner = it.inner.clone();
					Call::CallTracer(CallTracerCall {
						from: from,
						gas: gas,
						gas_used: gas_used,
						trace_address: Some(trace_address.clone()),
						inner: match inner.clone() {
							BlockscoutCallInner::Call {
								input,
								to,
								res,
								call_type,
							} => CallTracerInner::Call {
								call_type: match call_type {
									CallType::Call => "CALL".as_bytes().to_vec(),
									CallType::CallCode => "CALLCODE".as_bytes().to_vec(),
									CallType::DelegateCall => "DELEGATECALL".as_bytes().to_vec(),
									CallType::StaticCall => "STATICCALL".as_bytes().to_vec(),
								},
								to,
								input,
								res,
								value: Some(value),
							},
							BlockscoutCallInner::Create { init, res } => CallTracerInner::Create {
								input: init,
								error: match res {
									CreateResult::Success { .. } => None,
									CreateResult::Error { ref error } => Some(error.clone()),
								},
								to: match res {
									CreateResult::Success {
										created_contract_address_hash,
										..
									} => Some(created_contract_address_hash),
									CreateResult::Error { .. } => None,
								},
								output: match res {
									CreateResult::Success {
										created_contract_code,
										..
									} => Some(created_contract_code),
									CreateResult::Error { .. } => None,
								},
								value: value,
								call_type: "CREATE".as_bytes().to_vec(),
							},
							BlockscoutCallInner::SelfDestruct { balance, to } => {
								CallTracerInner::SelfDestruct {
									value: balance,
									to,
									call_type: "SELFDESTRUCT".as_bytes().to_vec(),
								}
							}
						},
						calls: Vec::new(),
					})
				})
				.collect();
			// Geth's `callTracer` expects a tree of nested calls and we have a stack.
			//
			// We iterate over the sorted stack, and push each children to it's
			// parent (the item which's `trace_address` matches &T[0..T.len()-1]) until there
			// is a single item on the list.
			//
			// The last remaining item is the context call with all it's descendants. I.e.
			//
			// 		# Input
			// 		[]
			// 		[0]
			// 		[0,0]
			// 		[0,0,0]
			// 		[0,1]
			// 		[0,1,0]
			// 		[0,1,1]
			// 		[0,1,2]
			// 		[1]
			// 		[1,0]
			//
			// 		# Sorted
			// 		[0,0,0] -> pop 0 and push to [0,0]
			// 		[0,1,0] -> pop 0 and push to [0,1]
			// 		[0,1,1] -> pop 1 and push to [0,1]
			// 		[0,1,2] -> pop 2 and push to [0,1]
			// 		[0,0] -> pop 0 and push to [0]
			// 		[0,1] -> pop 1 and push to [0]
			// 		[1,0] -> pop 0 and push to [1]
			// 		[0] -> pop 0 and push to root
			// 		[1] -> pop 1 and push to root
			// 		[]
			//
			// 		# Result
			// 		root {
			// 			calls: {
			// 				0 { 0 { 0 }, 1 { 0, 1, 2 }},
			// 				1 { 0 },
			// 			}
			// 		}
			if result.len() > 1 {
				// Sort the stack. Assume there is no `Ordering::Equal`, as we are
				// sorting by index.
				//
				// We consider an item to be `Ordering::Less` when:
				// 	- Is closer to the root or
				//	- Is greater than its sibling.
				result.sort_by(|a, b| match (a, b) {
					(
						Call::CallTracer(CallTracerCall {
							trace_address: Some(a),
							..
						}),
						Call::CallTracer(CallTracerCall {
							trace_address: Some(b),
							..
						}),
					) => {
						let a_len = a.len();
						let b_len = b.len();
						let sibling_greater_than = |a: &Vec<u32>, b: &Vec<u32>| -> bool {
							for (i, a_value) in a.iter().enumerate() {
								if a_value > &b[i] {
									return true;
								} else if a_value < &b[i] {
									return false;
								} else {
									continue;
								}
							}
							return false;
						};
						if b_len > a_len || (a_len == b_len && sibling_greater_than(&a, &b)) {
							Ordering::Less
						} else {
							Ordering::Greater
						}
					}
					_ => unreachable!(),
				});
				// Stack pop-and-push.
				while result.len() > 1 {
					let mut last = result
						.pop()
						.expect("result.len() > 1, so pop() necessarily returns an element");
					// Find the parent index.
					if let Some(index) =
						result
							.iter()
							.position(|current| match (last.clone(), current) {
								(
									Call::CallTracer(CallTracerCall {
										trace_address: Some(a),
										..
									}),
									Call::CallTracer(CallTracerCall {
										trace_address: Some(b),
										..
									}),
								) => {
									&b[..]
										== a.get(0..a.len() - 1).expect(
											"non-root element while traversing trace result",
										)
								}
								_ => unreachable!(),
							}) {
						// Remove `trace_address` from result.
						if let Call::CallTracer(CallTracerCall {
							ref mut trace_address,
							..
						}) = last
						{
							*trace_address = None;
						}
						// Push the children to parent.
						if let Some(Call::CallTracer(CallTracerCall { calls, .. })) =
							result.get_mut(index)
						{
							calls.push(last);
						}
					}
				}
			}
			// Remove `trace_address` from result.
			if let Some(Call::CallTracer(CallTracerCall { trace_address, .. })) = result.get_mut(0)
			{
				*trace_address = None;
			}
			if result.len() == 1 {
				traces.push(TransactionTrace::CallListNested(result.pop().expect(
					"result.len() == 1, so pop() necessarily returns this element",
				)));
			}
		}
		if traces.is_empty() {
			return None;
		}
		return Some(traces);
	}
}

#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CallTracerCall {
	pub from: H160,

	/// Indices of parent calls. Used to build the Etherscan nested response.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub trace_address: Option<Vec<u32>>,

	/// Remaining gas in the runtime.
	pub gas: U256,
	/// Gas used by this context.
	pub gas_used: U256,

	#[serde(flatten)]
	pub inner: CallTracerInner,

	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub calls: Vec<Call>,
}

#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode, Serialize)]
#[serde(untagged)]
pub enum CallTracerInner {
	Call {
		#[serde(rename = "type", serialize_with = "opcode_serialize")]
		call_type: Vec<u8>,
		to: H160,
		#[serde(serialize_with = "bytes_0x_serialize")]
		input: Vec<u8>,
		/// "output" or "error" field
		#[serde(flatten)]
		res: CallResult,

		#[serde(skip_serializing_if = "Option::is_none")]
		value: Option<U256>,
	},
	Create {
		#[serde(rename = "type", serialize_with = "opcode_serialize")]
		call_type: Vec<u8>,
		#[serde(serialize_with = "bytes_0x_serialize")]
		input: Vec<u8>,
		#[serde(skip_serializing_if = "Option::is_none")]
		to: Option<H160>,
		#[serde(
			skip_serializing_if = "Option::is_none",
			serialize_with = "option_bytes_0x_serialize"
		)]
		output: Option<Vec<u8>>,
		#[serde(
			skip_serializing_if = "Option::is_none",
			serialize_with = "option_string_serialize"
		)]
		error: Option<Vec<u8>>,
		value: U256,
	},
	SelfDestruct {
		#[serde(rename = "type", serialize_with = "opcode_serialize")]
		call_type: Vec<u8>,
		to: H160,
		value: U256,
	},
}
