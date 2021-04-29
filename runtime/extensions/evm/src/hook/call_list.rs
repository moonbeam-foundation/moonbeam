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

use super::*;

use ethereum_types::{H160, U256};
use evm::{ExitError, ExitSucceed, Handler, Opcode};
use moonbeam_rpc_primitives_debug::{
	single::{Call, CallInner},
	CallResult, CallType, CreateResult,
};
use sp_std::{collections::btree_map::BTreeMap, vec, vec::Vec};

pub struct State {
	entries: BTreeMap<u32, Call>,
	entries_next_index: u32,
	next_context_type: Option<ContextType>,
	trace_address: Vec<u32>,
	context_stack: Vec<Context>,
}

struct Context {
	entries_index: u32,
	context_type: ContextType,
	from: H160,
	to: H160,
	value: U256,
	gas_at_start: u64,

	input: Vec<u8>,
	subcall_step: bool,
	suicide_send: Option<(H160, U256)>,
}

enum ContextType {
	Call(CallType),
	Create,
}

impl State {
	pub fn new() -> Self {
		Self {
			entries: BTreeMap::new(),
			entries_next_index: 0,
			next_context_type: None,
			trace_address: vec![],
			context_stack: vec![],
		}
	}

	/// Called before the execution of a context.
	pub fn before_loop<'config, S: StackState<'config>, H: Hook>(
		&mut self,
		executor: &StackExecutor<'config, S, H>,
		runtime: &Runtime,
	) {
		let context_type = self.next_context_type.take().unwrap_or_else(|| {
			// This will be reached only in the first context entry/root context.
			// We can know if we're in a Call or a Create by looking at the
			// "to" codesize. Inside a Create, this size will be 0.
			if executor.code_size(runtime.context().address) == 0.into() {
				ContextType::Create
			} else {
				ContextType::Call(CallType::Call)
			}
		});

		self.trace_address.push(0);

		self.context_stack.push(Context {
			entries_index: self.entries_next_index,
			context_type,
			from: runtime.context().caller,
			to: runtime.context().address,
			value: runtime.context().apparent_value,
			gas_at_start: executor.gas(),

			input: runtime.machine().data().to_vec(),
			subcall_step: false,
			suicide_send: None,
		});

		self.entries_next_index += 1;
	}

	/// Called before each step.
	pub fn before_step<'config, S: StackState<'config>, H: Hook>(
		&mut self,
		executor: &StackExecutor<'config, S, H>,
		runtime: &Runtime,
	) {
		let context = self
			.context_stack
			.last_mut()
			.expect("before_step called after before_loop");

		if let Some((opcode, _)) = runtime.machine().inspect() {
			self.next_context_type = match opcode {
				Opcode(0xf0) => Some(ContextType::Create),
				Opcode(0xf1) => Some(ContextType::Call(CallType::Call)),
				Opcode(0xf2) => Some(ContextType::Call(CallType::CallCode)),
				Opcode(0xf4) => Some(ContextType::Call(CallType::DelegateCall)),
				// 0xf5 : create2 not supported
				Opcode(0xfa) => Some(ContextType::Call(CallType::StaticCall)),
				_ => None,
			};

			context.subcall_step = self.next_context_type.is_some();

			// SELFDESTRUCT
			if opcode == Opcode(0xff) {
				let stack = runtime.machine().stack().data();

				context.suicide_send = stack
					.get(stack.len() - 1)
					.cloned()
					.map(|v| (H160::from(v), executor.balance(runtime.context().address)));
			}
		}
	}

	/// Called after each step. Will not be called if runtime exited
	/// from the loop.
	pub fn after_step<'config, S: StackState<'config>, H: Hook>(
		&mut self,
		_executor: &StackExecutor<'config, S, H>,
		_runtime: &Runtime,
	) {
		let context = self
			.context_stack
			.last_mut()
			.expect("after_step called after before_step");

		if context.subcall_step {
			*self.trace_address.last_mut().unwrap() += 1;
		}
	}

	/// Called after the execution of a context.
	pub fn after_loop<'config, S: StackState<'config>, H: Hook>(
		&mut self,
		executor: &StackExecutor<'config, S, H>,
		runtime: &Runtime,
		reason: &ExitReason,
	) {
		let context = self
			.context_stack
			.pop()
			.expect("after_loop called after after_step");

		// Compute used gas.
		let gas_at_end = executor.gas();
		let gas_used = context.gas_at_start - gas_at_end;

		// Handle suicide additional entry.
		if let Some((refund_address, balance)) = context.suicide_send {
			let entries_index = self.entries_next_index;
			self.entries_next_index += 1;

			self.entries.insert(
				entries_index,
				Call {
					from: context.to, // this contyract is self destructing
					trace_address: self.trace_address.clone(),
					subtraces: 0,
					value: context.value,
					gas: gas_at_end.into(),
					gas_used: gas_used.into(),
					inner: CallInner::SelfDestruct {
						refund_address,
						balance,
					},
				},
			);
		}

		// We pop the children item, giving back this context trace_address.
		let subtraces = self.trace_address.pop().unwrap();

		self.entries.insert(
			context.entries_index,
			match context.context_type {
				ContextType::Call(call_type) => {
					let res = match &reason {
						ExitReason::Succeed(ExitSucceed::Returned) => {
							CallResult::Output(runtime.machine().return_value())
						}
						ExitReason::Succeed(_) => CallResult::Output(vec![]),
						ExitReason::Error(error) => CallResult::Error(error_message(error)),

						ExitReason::Revert(_) => CallResult::Error(b"execution reverted".to_vec()),
						ExitReason::Fatal(_) => CallResult::Error(vec![]),
					};

					Call {
						from: context.from,
						trace_address: self.trace_address.clone(),
						subtraces,
						value: context.value,
						gas: gas_at_end.into(),
						gas_used: gas_used.into(),
						inner: CallInner::Call {
							call_type,
							to: context.to,
							input: context.input,
							res,
						},
					}
				}
				ContextType::Create => {
					// let offset = runtine.machine().stack().data();
					let contract_code = executor.code(context.to);

					let res = match &reason {
						ExitReason::Succeed(_) => CreateResult::Success {
							created_contract_address_hash: context.to,
							created_contract_code: contract_code,
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
						trace_address: self.trace_address.clone(),
						subtraces,
						gas: U256::from(gas_at_end),
						gas_used: U256::from(gas_used),
						from: context.from,
						inner: CallInner::Create {
							init: context.input,
							res,
						},
					}
				}
			},
		);
	}

	pub fn finish(self) -> TransactionTrace {
		TransactionTrace::CallList(self.entries.into_iter().map(|(_, value)| value).collect())
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
