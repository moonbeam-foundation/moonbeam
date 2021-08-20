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

use super::{Call, TransactionTrace, blockscout::{BlockscoutCall, BlockscoutInner}};
use crate::{CreateResult, proxy::v2::call_list::Listener};

#[cfg(feature = "std")]
use crate::serialization::*;
#[cfg(feature = "std")]
use serde::Serialize;

use codec::{Decode, Encode};
use ethereum_types::{H160, U256};
use sp_std::{cmp::Ordering, vec::Vec};

pub struct Response;

#[cfg(feature = "std")]
impl super::TraceResponseBuilder for Response {
	type Listener = Listener;
	type Response = TransactionTrace;

	fn build(listener: Listener) -> Option<TransactionTrace> {
		if let Some(entry) = listener.entries.last() {
			let mut result: Vec<Call> = entry
				.into_iter()
				.filter_map(|(_, value)| match value {
					Call::Blockscout(BlockscoutCall {
						from,
						trace_address,
						value,
						gas,
						gas_used,
						inner,
						..
					}) => Some(Call::CallTracer(CallTracerCall {
						from: *from,
						gas: *gas,
						gas_used: *gas_used,
						trace_address: Some(trace_address.clone()),
						inner: match inner.clone() {
							BlockscoutInner::Call {
								input,
								to,
								res,
								call_type,
							} => CallTracerInner::Call {
								call_type: match call_type {
									crate::CallType::Call => "CALL".as_bytes().to_vec(),
									crate::CallType::CallCode => "CALLCODE".as_bytes().to_vec(),
									crate::CallType::DelegateCall => {
										"DELEGATECALL".as_bytes().to_vec()
									}
									crate::CallType::StaticCall => "STATICCALL".as_bytes().to_vec(),
								},
								to,
								input,
								res,
								value: Some(*value),
							},
							BlockscoutInner::Create { init, res } => CallTracerInner::Create {
								input: init,
								error: match res {
									CreateResult::Success { .. } => None,
									crate::CreateResult::Error { ref error } => Some(error.clone()),
								},
								to: match res {
									CreateResult::Success {
										created_contract_address_hash,
										..
									} => Some(created_contract_address_hash),
									crate::CreateResult::Error { .. } => None,
								},
								output: match res {
									CreateResult::Success {
										created_contract_code,
										..
									} => Some(created_contract_code),
									crate::CreateResult::Error { .. } => None,
								},
								value: *value,
								call_type: "CREATE".as_bytes().to_vec(),
							},
							BlockscoutInner::SelfDestruct {
								balance,
								refund_address,
							} => CallTracerInner::SelfDestruct {
								value: balance,
								to: refund_address,
								call_type: "SELFDESTRUCT".as_bytes().to_vec(),
							},
						},
						calls: Vec::new(),
					})),
					_ => None,
				})
				.map(|x| x)
				.collect();

			// Geth's `callTracer` expects a tree of nested calls and we have a stack.
			//
			// We iterate over the sorted stack, and push each children to it's
			// parent (the item which's `trace_address` matches &T[0..T.len()-1]) until there
			// is a single item on the list.
			//
			// The last remaining item is the context call with all it's descendants. I.e.
			//
			// 		[0,0,0] -> pop() and added to [0,0]
			// 		[1,0,0] -> pop() and added to [1,0]
			// 		[1,0,1] -> pop() and added to [1,0]
			// 		[0,0] -> pop() and added to [0]
			// 		[0,1] -> pop() and added to [0]
			// 		[1,0] -> pop() and added to [1]
			// 		[0] -> pop() and added to []
			// 		[1] -> pop() and added to []
			// 		[] -> list length == 1, out

			if result.len() > 1 {
				// Sort the stack. Assume there is no `Ordering::Equal`, as we are
				// sorting by index.
				//
				// We consider an item to be `Ordering::Less` when:
				// 	- Is closer to the root.
				//	- The concatenated numerical representation of it's indexes is
				//	greater than it's siblings. This allows to later pop (and push) the indexes
				//	sorted ASC.
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
						// Concat a Vec to u32.
						let f = |idxs: &Vec<u32>| -> u32 {
							idxs.iter()
								.map(ToString::to_string)
								.collect::<String>()
								.parse::<u32>()
								.unwrap_or(0)
						};
						if b_len > a_len || (a_len == b_len && (f(&b) < f(&a))) {
							Ordering::Less
						} else {
							Ordering::Greater
						}
					}
					_ => unreachable!(),
				});
				// Stack pop-and-push.
				while result.len() > 1 {
					let mut last = result.pop().unwrap();
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
								) => &b[..] == &a[0..a.len() - 1],
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
				return Some(TransactionTrace::CallListNested(result.pop().unwrap()));
			}
			return None;
		}
		None
	}
}

#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Serialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct CallTracerCall {
	pub from: H160,

	/// Indices of parent calls. Used to build the Etherscan nested response.
	#[cfg_attr(feature = "std", serde(skip_serializing_if = "Option::is_none"))]
	pub trace_address: Option<Vec<u32>>,

	/// Remaining gas in the runtime.
	pub gas: U256,
	/// Gas used by this context.
	pub gas_used: U256,

	#[cfg_attr(feature = "std", serde(flatten))]
	pub inner: CallTracerInner,

	#[cfg_attr(feature = "std", serde(skip_serializing_if = "Vec::is_empty"))]
	pub calls: Vec<Call>,
}

#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Serialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase", untagged))]
pub enum CallTracerInner {
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
