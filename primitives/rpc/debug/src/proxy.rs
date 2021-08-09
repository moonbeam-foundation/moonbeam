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

//! A Proxy in this context is an environmental trait implementor meant to be used for capturing
//! EVM trace results sent to a Host function from the Runtime. Works like:
//! - Runtime Api call `using` environmental.
//! - Runtime calls a Host function with some scale-encoded data.
//! - Host function emits an event.
//! - Proxy listens for the event and stores the decoded data.
//!
//! There are two proxy types: `Raw` and `CallList`.
//! - `Raw` - used for opcode-level traces.
//! - `CallList` - used for block tracing (stack of call stacks) and custom tracing outputs.
extern crate alloc;
environmental::environmental!(listener: dyn Listener + 'static);

use crate::block::{
	TransactionTrace as BlockTrace, TransactionTraceAction, TransactionTraceOutput,
	TransactionTraceResult,
};
use crate::single::{Call, CallInner, GethCallInner, RawStepLog, TransactionTrace as SingleTrace};
use crate::{CallResult, CreateResult, CreateType, TracerInput};
use alloc::string::{String, ToString};
use codec::{Decode, Encode};
use ethereum_types::{H256, U256};
use sp_std::{cmp::Ordering, collections::btree_map::BTreeMap, vec::Vec};
/// Main trait to proxy emitted messages.
pub trait Listener {
	fn event(&mut self, event: Event);
}

#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode)]
pub enum Event {
	/// Opcode-level trace event.
	RawStep(RawStepLog),
	/// Final gas used event.
	RawGas(U256),
	/// EVM execution return value.
	RawReturnValue(Vec<u8>),
	/// An internal EVM Call for a single call-stack.
	CallListEntry((u32, Call)),
	/// A new call-stack.
	CallListNew(),
}

impl Event {
	/// Access the global reference and call it's `event` method, passing the `Event` itself as
	/// argument.
	///
	/// This only works if we are `using` a global reference to a `Listener` implementor.
	pub fn emit(self) {
		listener::with(|listener| listener.event(self));
	}
}

/// DebugRuntimeApi V1 result. Trace response is stored in runtime memory and returned as part of
/// the runtime api call.
#[derive(Debug)]
pub enum ResultV1 {
	Single(SingleTrace),
	Block(Vec<BlockTrace>),
}

/// DebugRuntimeApi V2 result. Trace response is stored in client and runtime api call response is
/// empty.
#[derive(Debug)]
pub enum ResultV2 {
	Single,
	Block,
}

/// Runtime api closure result.
#[derive(Debug)]
pub enum Result {
	V1(ResultV1),
	V2(ResultV2),
}

#[derive(Debug)]
pub struct RawProxy {
	gas: U256,
	return_value: Vec<u8>,
	step_logs: Vec<RawStepLog>,
}

impl RawProxy {
	pub fn new() -> Self {
		Self {
			gas: U256::zero(),
			return_value: Vec::new(),
			step_logs: Vec::new(),
		}
	}
	/// In the RPC handler context, `F` wraps a Runtime Api call.
	///
	/// With `using`, the Runtime Api is called with thread safe/local access to the mutable
	/// reference of `self`.
	pub fn using<R, F: FnOnce() -> R>(&mut self, f: F) -> R {
		listener::using(self, f)
	}

	/// Format the RPC output.
	pub fn into_tx_trace(self) -> SingleTrace {
		SingleTrace::Raw {
			step_logs: self.step_logs,
			gas: self.gas,
			return_value: self.return_value,
		}
	}
}

impl Listener for RawProxy {
	fn event(&mut self, event: Event) {
		match event {
			Event::RawStep(step) => self.step_logs.push(step),
			Event::RawGas(gas) => self.gas = gas,
			Event::RawReturnValue(value) => self.return_value = value,
			_ => {}
		};
	}
}

// List
#[derive(Debug)]
pub struct CallListProxy {
	entries: Vec<BTreeMap<u32, Call>>,
}

impl CallListProxy {
	pub fn new() -> Self {
		Self {
			entries: Vec::new(),
		}
	}
	/// In the RPC handler context, `F` wraps a Runtime Api call.
	///
	/// With `using`, the Runtime Api is called with thread safe/local access to the mutable
	/// reference of `self`.
	pub fn using<R, F: FnOnce() -> R>(&mut self, f: F) -> R {
		listener::using(self, f)
	}

	/// Format the RPC output of a single call-stack.
	pub fn into_tx_trace(self, tracer: TracerInput) -> Option<SingleTrace> {
		if let Some(entry) = self.entries.last() {
			let mut result: Vec<Call> = entry
				.into_iter()
				.filter_map(|(_, value)| match (value, tracer) {
					(Call::Blockscout { .. }, TracerInput::Blockscout) => Some(value.clone()),
					(
						Call::Blockscout {
							from,
							trace_address,
							value,
							gas,
							gas_used,
							inner,
							..
						},
						TracerInput::GethCallTrace,
					) => Some(Call::GethCallTrace {
						from: *from,
						gas: *gas,
						gas_used: *gas_used,
						trace_address: Some(trace_address.clone()),
						inner: match inner.clone() {
							CallInner::Call {
								input,
								to,
								res,
								call_type,
							} => GethCallInner::Call {
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
							CallInner::Create { res, .. } => GethCallInner::Create {
								res,
								value: *value,
								call_type: "CREATE".as_bytes().to_vec(),
							},
							CallInner::SelfDestruct {
								balance,
								refund_address,
							} => GethCallInner::SelfDestruct {
								value: balance,
								to: refund_address,
								call_type: "SELFDESTRUCT".as_bytes().to_vec(),
							},
						},
						calls: Vec::new(),
					}),
					_ => None,
				})
				.map(|x| x)
				.collect();
			if tracer == TracerInput::GethCallTrace {
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
							Call::GethCallTrace {
								trace_address: Some(a),
								..
							},
							Call::GethCallTrace {
								trace_address: Some(b),
								..
							},
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
										Call::GethCallTrace {
											trace_address: Some(a),
											..
										},
										Call::GethCallTrace {
											trace_address: Some(b),
											..
										},
									) => &b[..] == &a[0..a.len() - 1],
									_ => unreachable!(),
								}) {
							// Remove `trace_address` from result.
							if let Call::GethCallTrace {
								ref mut trace_address,
								..
							} = last
							{
								*trace_address = None;
							}
							// Push the children to parent.
							if let Some(Call::GethCallTrace { calls, .. }) = result.get_mut(index) {
								calls.push(last);
							}
						}
					}
				}
				// Remove `trace_address` from result.
				if let Some(Call::GethCallTrace { trace_address, .. }) = result.get_mut(0) {
					*trace_address = None;
				}
				if result.len() == 1 {
					return Some(SingleTrace::CallListNested(result.pop().unwrap()));
				}
				return None;
			}
			return Some(SingleTrace::CallList(result));
		}
		None
	}

	/// Format the RPC output for multiple transactions. Each call-stack represents a single
	/// transaction/EVM execution.
	pub fn into_tx_traces(self) -> Vec<BlockTrace> {
		let mut traces = Vec::new();
		for (eth_tx_index, entry) in self.entries.iter().enumerate() {
			let mut tx_traces: Vec<_> = entry
				.into_iter()
				.filter_map(|(_, trace)| {
					match trace {
						Call::Blockscout {
							from,
							trace_address,
							subtraces,
							value,
							gas,
							gas_used,
							inner,
						} => match inner.clone() {
							CallInner::Call {
								input,
								to,
								res,
								call_type,
							} => Some(BlockTrace {
								action: TransactionTraceAction::Call {
									call_type,
									from: *from,
									gas: *gas,
									input,
									to,
									value: *value,
								},
								// Can't be known here, must be inserted upstream.
								block_hash: H256::default(),
								// Can't be known here, must be inserted upstream.
								block_number: 0,
								output: match res {
									CallResult::Output(output) => TransactionTraceOutput::Result(
										TransactionTraceResult::Call {
											gas_used: *gas_used,
											output,
										},
									),
									crate::CallResult::Error(error) => {
										TransactionTraceOutput::Error(error)
									}
								},
								subtraces: *subtraces,
								trace_address: trace_address.clone(),
								// Can't be known here, must be inserted upstream.
								transaction_hash: H256::default(),
								transaction_position: eth_tx_index as u32,
							}),
							CallInner::Create { init, res } => {
								Some(BlockTrace {
									action: TransactionTraceAction::Create {
										creation_method: CreateType::Create,
										from: *from,
										gas: *gas,
										init,
										value: *value,
									},
									// Can't be known here, must be inserted upstream.
									block_hash: H256::default(),
									// Can't be known here, must be inserted upstream.
									block_number: 0,
									output: match res {
										CreateResult::Success {
											created_contract_address_hash,
											created_contract_code,
										} => TransactionTraceOutput::Result(
											TransactionTraceResult::Create {
												gas_used: *gas_used,
												code: created_contract_code,
												address: created_contract_address_hash,
											},
										),
										crate::CreateResult::Error { error } => {
											TransactionTraceOutput::Error(error)
										}
									},
									subtraces: *subtraces,
									trace_address: trace_address.clone(),
									// Can't be known here, must be inserted upstream.
									transaction_hash: H256::default(),
									transaction_position: eth_tx_index as u32,
								})
							}
							CallInner::SelfDestruct {
								balance,
								refund_address,
							} => Some(BlockTrace {
								action: TransactionTraceAction::Suicide {
									address: *from,
									balance,
									refund_address,
								},
								// Can't be known here, must be inserted upstream.
								block_hash: H256::default(),
								// Can't be known here, must be inserted upstream.
								block_number: 0,
								output: TransactionTraceOutput::Result(
									TransactionTraceResult::Suicide,
								),
								subtraces: *subtraces,
								trace_address: trace_address.clone(),
								// Can't be known here, must be inserted upstream.
								transaction_hash: H256::default(),
								transaction_position: eth_tx_index as u32,
							}),
						},
						_ => None,
					}
				})
				.map(|x| x)
				.collect();

			traces.append(&mut tx_traces);
		}
		traces
	}
}

impl Listener for CallListProxy {
	fn event(&mut self, event: Event) {
		match event {
			Event::CallListNew() => {
				self.entries.push(BTreeMap::new());
			}
			Event::CallListEntry((index, value)) => {
				if self.entries.is_empty() {
					self.entries.push(BTreeMap::new());
				}
				self.entries.last_mut().unwrap().insert(index, value);
			}
			_ => {}
		};
	}
}
