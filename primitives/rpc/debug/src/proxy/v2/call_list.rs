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

extern crate alloc;
use super::{
	Capture, ContextType, Event, EvmEvent, ExitError, ExitReason, ExitSucceed, GasometerEvent,
	Listener as ListenerT, RuntimeEvent, H160, H256, U256,
};
use crate::{
	block::{
		TransactionTrace as BlockTrace, TransactionTraceAction, TransactionTraceOutput,
		TransactionTraceResult,
	},
	single::{Call, CallInner, TransactionTrace as SingleTrace},
	CallResult, CallType, CreateResult, CreateType,
};
use alloc::{collections::btree_map::BTreeMap, vec, vec::Vec};

pub struct Listener {
	// Transaction cost that must be added to the first context cost.
	transaction_cost: u64,

	// Final logs.
	entries: Vec<BTreeMap<u32, Call>>,
	// Next index to use.
	entries_next_index: u32,
	// Stack of contexts with data to keep between events.
	context_stack: Vec<Context>,

	// Type of the next call.
	// By default is None and corresponds to the root call, which
	// can be determined using the `is_static` field of the `Call` event.
	// Then by looking at call traps events we can set this value to the correct
	// call type, to be used when the following `Call` event is received.
	call_type: Option<CallType>,

	/// true = we are before the first Evm::Call/Create event a transaction.
	/// Allow to handle early errors before these events.
	early_in_tx: bool,
}

struct Context {
	entries_index: u32,

	context_type: ContextType,

	from: H160,
	trace_address: Vec<u32>,
	subtraces: u32,
	value: U256,

	gas: u64,
	start_gas: Option<u64>,

	// input / data
	data: Vec<u8>,
	// to / create address
	to: H160,
}

impl Default for Listener {
	fn default() -> Self {
		Self {
			transaction_cost: 0,

			entries: vec![],
			entries_next_index: 0,

			context_stack: vec![],

			call_type: None,
			early_in_tx: true,
		}
	}
}

impl Listener {
	pub fn using<R, F: FnOnce() -> R>(&mut self, f: F) -> R {
		super::listener::using(self, f)
	}

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
	pub fn into_tx_traces(self) -> Vec<BlockTrace> {
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
							crate::CallResult::Error(error) => TransactionTraceOutput::Error(error),
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
								crate::CreateResult::Error { error } => {
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

	pub fn gasometer_event(&mut self, event: GasometerEvent) {
		match event {
			GasometerEvent::RecordCost { snapshot, .. }
			| GasometerEvent::RecordDynamicCost { snapshot, .. }
			| GasometerEvent::RecordStipend { snapshot, .. } => {
				if let Some(context) = self.context_stack.last_mut() {
					if context.start_gas.is_none() {
						context.start_gas = Some(snapshot.gas());
					}
					context.gas = snapshot.gas();
				}
			}
			GasometerEvent::RecordTransaction { cost, .. } => self.transaction_cost = cost,
			// We ignore other kinds of message if any (new ones may be added in the future).
			#[allow(unreachable_patterns)]
			_ => (),
		}
	}

	pub fn runtime_event(&mut self, event: RuntimeEvent) {
		match event {
			RuntimeEvent::StepResult {
				result: Err(Capture::Trap(opcode)),
				..
			} => {
				if let Some(ContextType::Call(call_type)) = ContextType::from(opcode.clone()) {
					self.call_type = Some(call_type)
				}
			}
			// RuntimeEvent::StepResult {
			// 	result: Err(Capture::Exit(reason)),
			// 	return_value,
			// } => {
			// 	if let Some(context) = self.context_stack.pop() {
			// 		let mut gas_used = context.start_gas.unwrap() - context.gas;
			// 		if context.entries_index == 0 {
			// 			gas_used += self.transaction_cost;
			// 		}

			// 		if self.entries.is_empty() {
			// 			self.entries.push(BTreeMap::new());
			// 		}
			// 		self.entries.last_mut().unwrap().insert(
			// 			context.entries_index,
			// 			match context.context_type {
			// 				ContextType::Call(call_type) => {
			// 					let res = match &reason {
			// 						ExitReason::Succeed(ExitSucceed::Returned) => {
			// 							CallResult::Output(return_value.to_vec())
			// 						}
			// 						ExitReason::Succeed(_) => CallResult::Output(vec![]),
			// 						ExitReason::Error(error) => {
			// 							CallResult::Error(error_message(error))
			// 						}

			// 						ExitReason::Revert(_) => {
			// 							CallResult::Error(b"execution reverted".to_vec())
			// 						}
			// 						ExitReason::Fatal(_) => CallResult::Error(vec![]),
			// 					};

			// 					Call {
			// 						from: context.from,
			// 						trace_address: context.trace_address,
			// 						subtraces: context.subtraces,
			// 						value: context.value,
			// 						gas: context.gas.into(),
			// 						gas_used: gas_used.into(),
			// 						inner: CallInner::Call {
			// 							call_type,
			// 							to: context.to,
			// 							input: context.data,
			// 							res,
			// 						},
			// 					}
			// 				}
			// 				ContextType::Create => {
			// 					let res = match &reason {
			// 						ExitReason::Succeed(_) => CreateResult::Success {
			// 							created_contract_address_hash: context.to,
			// 							created_contract_code: return_value.to_vec(),
			// 						},
			// 						ExitReason::Error(error) => CreateResult::Error {
			// 							error: error_message(error),
			// 						},
			// 						ExitReason::Revert(_) => CreateResult::Error {
			// 							error: b"execution reverted".to_vec(),
			// 						},
			// 						ExitReason::Fatal(_) => CreateResult::Error { error: vec![] },
			// 					};

			// 					Call {
			// 						value: context.value,
			// 						trace_address: context.trace_address,
			// 						subtraces: context.subtraces,
			// 						gas: context.gas.into(),
			// 						gas_used: gas_used.into(),
			// 						from: context.from,
			// 						inner: CallInner::Create {
			// 							init: context.data,
			// 							res,
			// 						},
			// 					}
			// 				}
			// 			},
			// 		);
			// 	}
			// }
			// We ignore other kinds of message if any (new ones may be added in the future).
			#[allow(unreachable_patterns)]
			_ => (),
		}
	}

	pub fn evm_event(&mut self, event: EvmEvent) {
		debug(&event);

		// let trace_address = if let Some(context) = self.context_stack.last_mut() {
		// 	let mut trace_address = context.trace_address.clone();
		// 	trace_address.push(context.subtraces);
		// 	context.subtraces += 1;
		// 	trace_address
		// } else {
		// 	vec![]
		// };

		match event {
			EvmEvent::TransactCall {
				caller,
				address,
				value,
				data,
				gas_limit,
			} => {
				self.context_stack.push(Context {
					entries_index: self.entries_next_index,

					context_type: ContextType::Call(CallType::Call),

					from: caller,
					trace_address: vec![],
					subtraces: 0,
					value,

					gas: 0,
					start_gas: None,

					data,
					to: address,
				});

				self.entries_next_index += 1;
			}

			EvmEvent::TransactCreate {
				caller,
				value,
				init_code,
				gas_limit,
				address,
			} => {
				self.context_stack.push(Context {
					entries_index: self.entries_next_index,

					context_type: ContextType::Create,

					from: caller,
					trace_address: vec![],
					subtraces: 0,
					value,

					gas: 0,
					start_gas: None,

					data: init_code,
					to: address,
				});

				self.entries_next_index += 1;
			}

			EvmEvent::TransactCreate2 {
				caller,
				value,
				init_code,
				gas_limit,
				address,
				..
			} => {
				self.context_stack.push(Context {
					entries_index: self.entries_next_index,

					context_type: ContextType::Create,

					from: caller,
					trace_address: vec![],
					subtraces: 0,
					value,

					gas: 0,
					start_gas: None,

					data: init_code,
					to: address,
				});

				self.entries_next_index += 1;
			}

			EvmEvent::Call {
				// code_address,
				// transfer,
				input,
				target_gas,
				is_static,
				context,
				..
			} => {
				let call_type = match (self.call_type, is_static) {
					(None, true) => CallType::StaticCall,
					(None, false) => CallType::Call,
					(Some(call_type), _) => call_type,
				};

				if !self.early_in_tx {
					let trace_address = if let Some(context) = self.context_stack.last_mut() {
						let mut trace_address = context.trace_address.clone();
						trace_address.push(context.subtraces);
						context.subtraces += 1;
						trace_address
					} else {
						vec![]
					};

					self.context_stack.push(Context {
						entries_index: self.entries_next_index,

						context_type: ContextType::Call(call_type),

						from: context.caller,
						trace_address,
						subtraces: 0,
						value: context.apparent_value,

						gas: 0,
						start_gas: None,

						data: input.to_vec(),
						to: context.address,
					});

					self.entries_next_index += 1;
				} else {
					self.early_in_tx = false;
				}
			}

			EvmEvent::Create {
				caller,
				address,
				// scheme,
				value,
				init_code,
				target_gas,
				..
			} => {
				if !self.early_in_tx {
					let trace_address = if let Some(context) = self.context_stack.last_mut() {
						let mut trace_address = context.trace_address.clone();
						trace_address.push(context.subtraces);
						context.subtraces += 1;
						trace_address
					} else {
						vec![]
					};

					self.context_stack.push(Context {
						entries_index: self.entries_next_index,

						context_type: ContextType::Create,

						from: caller,
						trace_address,
						subtraces: 0,
						value,

						gas: 0,
						start_gas: None,

						data: init_code.to_vec(),
						to: address,
					});
				} else {
					self.early_in_tx = false;
				}

				self.entries_next_index += 1;
			}
			EvmEvent::Suicide {
				address,
				target,
				balance,
			} => {
				let trace_address = if let Some(context) = self.context_stack.last_mut() {
					let mut trace_address = context.trace_address.clone();
					trace_address.push(context.subtraces);
					context.subtraces += 1;
					trace_address
				} else {
					vec![]
				};

				if self.entries.is_empty() {
					self.entries.push(BTreeMap::new());
				}
				self.entries.last_mut().unwrap().insert(
					self.entries_next_index,
					Call {
						from: address, // this contract is self destructing
						trace_address,
						subtraces: 0,
						value: 0.into(),
						gas: 0.into(),
						gas_used: 0.into(),
						inner: CallInner::SelfDestruct {
							refund_address: target,
							balance,
						},
					},
				);
				self.entries_next_index += 1;
			}
			EvmEvent::Exit {
				reason,
				return_value,
			} => {
				if let Some(context) = self.context_stack.pop() {
					let mut gas_used = context.start_gas.unwrap() - context.gas;
					if context.entries_index == 0 {
						gas_used += self.transaction_cost;
					}

					if self.entries.is_empty() {
						self.entries.push(BTreeMap::new());
					}
					self.entries.last_mut().unwrap().insert(
						context.entries_index,
						match context.context_type {
							ContextType::Call(call_type) => {
								let res = match &reason {
									ExitReason::Succeed(ExitSucceed::Returned) => {
										CallResult::Output(return_value.to_vec())
									}
									ExitReason::Succeed(_) => CallResult::Output(vec![]),
									ExitReason::Error(error) => {
										CallResult::Error(error_message(error))
									}

									ExitReason::Revert(_) => {
										CallResult::Error(b"execution reverted".to_vec())
									}
									ExitReason::Fatal(_) => CallResult::Error(vec![]),
								};

								Call {
									from: context.from,
									trace_address: context.trace_address,
									subtraces: context.subtraces,
									value: context.value,
									gas: context.gas.into(),
									gas_used: gas_used.into(),
									inner: CallInner::Call {
										call_type,
										to: context.to,
										input: context.data,
										res,
									},
								}
							}
							ContextType::Create => {
								let res = match &reason {
									ExitReason::Succeed(_) => CreateResult::Success {
										created_contract_address_hash: context.to,
										created_contract_code: return_value.to_vec(),
									},
									ExitReason::Error(error) => CreateResult::Error {
										error: error_message(error),
									},
									ExitReason::Revert(_) => CreateResult::Error {
										error: b"execution reverted".to_vec(),
									},
									ExitReason::Fatal(_) => CreateResult::Error { error: vec![] },
								};

								Call {
									value: context.value,
									trace_address: context.trace_address,
									subtraces: context.subtraces,
									gas: context.gas.into(),
									gas_used: gas_used.into(),
									from: context.from,
									inner: CallInner::Create {
										init: context.data,
										res,
									},
								}
							}
						},
					);
				}
			}
			// We ignore other kinds of message if any (new ones may be added in the future).
			#[allow(unreachable_patterns)]
			_ => (),
		}
	}
}

#[cfg(feature = "std")]
fn debug<T: core::fmt::Debug>(v: T) {
	println!("EvmEvent::Exit : {:?}", v);
}

#[cfg(not(feature = "std"))]
fn debug<T: core::fmt::Debug>(v: T) {}

fn error_message(error: &ExitError) -> Vec<u8> {
	match error {
		ExitError::StackUnderflow => "stack underflow",
		ExitError::StackOverflow => "stack overflow",
		ExitError::InvalidJump => "invalid jump",
		ExitError::InvalidRange => "invalid range",
		ExitError::DesignatedInvalid => "designated invalid",
		ExitError::CallTooDeep => "call too deep",
		ExitError::CreateCollision => "create collision",
		ExitError::CreateContractLimit => "create contract limit",
		ExitError::OutOfOffset => "out of offset",
		ExitError::OutOfGas => "out of gas",
		ExitError::OutOfFund => "out of funds",
		ExitError::Other(err) => err,
		_ => "unexpected error",
	}
	.as_bytes()
	.to_vec()
}

impl ListenerT for Listener {
	fn event(&mut self, event: Event) {
		match event {
			Event::Gasometer(gasometer_event) => self.gasometer_event(gasometer_event),
			Event::Runtime(runtime_event) => self.runtime_event(runtime_event),
			Event::Evm(evm_event) => self.evm_event(evm_event),
			Event::CallListNew() => {
				self.early_in_tx = true;
				self.entries.push(BTreeMap::new());
			}
		};
	}
}
