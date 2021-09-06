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

//! Legacy version of the client-side components for the tracer.
//!
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

environmental::environmental!(listener: dyn Listener + 'static);

use crate::api::{
	block::{
		TransactionTrace as BlockTrace, TransactionTraceAction, TransactionTraceOutput,
		TransactionTraceResult,
	},
	single::{Call, CallInner, RawStepLog, TransactionTrace as SingleTrace},
	CallResult, CreateResult, CreateType,
};
use codec::{Decode, Encode};
use ethereum_types::{H256, U256};
use sp_std::{collections::btree_map::BTreeMap, vec::Vec};

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
pub enum Response {
	Single(SingleTrace),
	Block(Vec<BlockTrace>),
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
	pub fn into_tx_trace(self) -> Option<SingleTrace> {
		if let Some(entry) = self.entries.last() {
			return Some(SingleTrace::CallList(
				entry.into_iter().map(|(_, value)| value.clone()).collect(),
			));
		}
		None
	}

	/// Format the RPC output for multiple transactions. Each call-stack represents a single
	/// transaction/EVM execution.
	pub fn into_tx_traces(&mut self) -> Vec<BlockTrace> {
		// Remove empty BTreeMaps pushed to `entries`.
		// I.e. InvalidNonce or other pallet_evm::runner exits
		self.entries.retain(|x| !x.is_empty());
		let mut traces = Vec::new();
		for (eth_tx_index, entry) in self.entries.iter().enumerate() {
			let mut tx_traces: Vec<_> = entry
				.into_iter()
				.map(|(_, trace)| match trace.inner.clone() {
					CallInner::Call {
						input,
						to,
						res,
						call_type,
					} => BlockTrace {
						action: TransactionTraceAction::Call {
							call_type,
							from: trace.from,
							gas: trace.gas,
							input,
							to,
							value: trace.value,
						},
						// Can't be known here, must be inserted upstream.
						block_hash: H256::default(),
						// Can't be known here, must be inserted upstream.
						block_number: 0,
						output: match res {
							CallResult::Output(output) => {
								TransactionTraceOutput::Result(TransactionTraceResult::Call {
									gas_used: trace.gas_used,
									output,
								})
							}
							crate::api::CallResult::Error(error) => {
								TransactionTraceOutput::Error(error)
							}
						},
						subtraces: trace.subtraces,
						trace_address: trace.trace_address.clone(),
						// Can't be known here, must be inserted upstream.
						transaction_hash: H256::default(),
						transaction_position: eth_tx_index as u32,
					},
					CallInner::Create { init, res } => {
						BlockTrace {
							action: TransactionTraceAction::Create {
								creation_method: CreateType::Create,
								from: trace.from,
								gas: trace.gas,
								init,
								value: trace.value,
							},
							// Can't be known here, must be inserted upstream.
							block_hash: H256::default(),
							// Can't be known here, must be inserted upstream.
							block_number: 0,
							output: match res {
								CreateResult::Success {
									created_contract_address_hash,
									created_contract_code,
								} => {
									TransactionTraceOutput::Result(TransactionTraceResult::Create {
										gas_used: trace.gas_used,
										code: created_contract_code,
										address: created_contract_address_hash,
									})
								}
								crate::api::CreateResult::Error { error } => {
									TransactionTraceOutput::Error(error)
								}
							},
							subtraces: trace.subtraces,
							trace_address: trace.trace_address.clone(),
							// Can't be known here, must be inserted upstream.
							transaction_hash: H256::default(),
							transaction_position: eth_tx_index as u32,
						}
					}
					CallInner::SelfDestruct {
						balance,
						refund_address,
					} => BlockTrace {
						action: TransactionTraceAction::Suicide {
							address: trace.from,
							balance,
							refund_address,
						},
						// Can't be known here, must be inserted upstream.
						block_hash: H256::default(),
						// Can't be known here, must be inserted upstream.
						block_number: 0,
						output: TransactionTraceOutput::Result(TransactionTraceResult::Suicide),
						subtraces: trace.subtraces,
						trace_address: trace.trace_address.clone(),
						// Can't be known here, must be inserted upstream.
						transaction_hash: H256::default(),
						transaction_position: eth_tx_index as u32,
					},
				})
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
