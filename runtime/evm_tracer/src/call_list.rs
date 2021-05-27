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

use crate::util::*;
use ethereum_types::{H160, U256};
use evm::{Capture, ExitError, ExitReason, ExitSucceed};
use moonbeam_rpc_primitives_debug::{
	single::{Call, CallInner, TransactionTrace},
	CallResult, CallType, CreateResult,
};
use sp_std::collections::btree_map::BTreeMap;

#[derive(Debug)]
pub struct CallListTracer {
	// Transaction cost that must be added to the first context cost.
	transaction_cost: u64,

	// Final logs.
	entries: BTreeMap<u32, Call>,
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
}

#[derive(Debug)]
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

impl CallListTracer {
	pub fn new() -> Self {
		Self {
			transaction_cost: 0,

			entries: BTreeMap::new(),
			entries_next_index: 0,

			context_stack: vec![],

			call_type: None,
		}
	}

	pub fn trace<R, F: FnOnce() -> R>(self, f: F) -> (Self, R) {
		let wrapped = Rc::new(RefCell::new(self));

		let result = {
			let mut gasometer = ListenerProxy(Rc::clone(&wrapped));
			let mut runtime = ListenerProxy(Rc::clone(&wrapped));
			let mut evm = ListenerProxy(Rc::clone(&wrapped));

			// Each line wraps the previous `f` into a `using` call.
			// Listening to new events results in adding one new line.
			// Order is irrelevant when registering listeners.
			let f = || runtime_using(&mut runtime, f);
			let f = || gasometer_using(&mut gasometer, f);
			let f = || evm_using(&mut evm, f);
			f()
		};

		(Rc::try_unwrap(wrapped).unwrap().into_inner(), result)
	}

	pub fn into_tx_trace(self) -> TransactionTrace {
		TransactionTrace::CallList(self.entries.into_iter().map(|(_, value)| value).collect())
	}
}

impl GasometerListener for CallListTracer {
	fn event(&mut self, event: GasometerEvent) {
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
}

impl RuntimeListener for CallListTracer {
	fn event(&mut self, event: RuntimeEvent) {
		match event {
			RuntimeEvent::StepResult {
				result: Err(Capture::Trap(opcode)),
				..
			} => {
				if let Some(ContextType::Call(call_type)) = ContextType::from(*opcode) {
					self.call_type = Some(call_type)
				}
			}
			RuntimeEvent::StepResult {
				result: Err(Capture::Exit(reason)),
				return_value,
			} => {
				if let Some(context) = self.context_stack.pop() {
					let mut gas_used = context.start_gas.unwrap() - context.gas;
					if context.entries_index == 0 {
						gas_used += self.transaction_cost;
					}

					self.entries.insert(
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

impl EvmListener for CallListTracer {
	fn event(&mut self, event: EvmEvent) {
		let trace_address = if let Some(context) = self.context_stack.last_mut() {
			let mut trace_address = context.trace_address.clone();
			trace_address.push(context.subtraces);
			context.subtraces += 1;
			trace_address
		} else {
			vec![]
		};

		match event {
			EvmEvent::Call {
				// code_address,
				// transfer,
				input,
				// target_gas,
				is_static,
				context,
				..
			} => {
				let call_type = match (self.call_type, is_static) {
					(None, true) => CallType::StaticCall,
					(None, false) => CallType::Call,
					(Some(call_type), _) => call_type,
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
			}
			EvmEvent::Create {
				caller,
				address,
				// scheme,
				value,
				init_code,
				// target_gas,
				..
			} => {
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

				self.entries_next_index += 1;
			}
			EvmEvent::Suicide {
				address,
				target,
				balance,
			} => {
				self.entries.insert(
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
			// We ignore other kinds of message if any (new ones may be added in the future).
			#[allow(unreachable_patterns)]
			_ => (),
		}
	}
}

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
