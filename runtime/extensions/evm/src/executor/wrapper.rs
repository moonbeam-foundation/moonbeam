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

use crate::executor::util::opcodes;
use moonbeam_rpc_primitives_debug::{
	single::{Call, CallInner, RawStepLog, TraceType},
	CallResult, CallType, CreateResult,
};

use ethereum_types::{H160, H256, U256};
pub use evm::{
	backend::{Apply, Backend as BackendT, Log},
	executor::{StackExecutor, StackExitKind, StackState as StackStateT},
	gasometer::{self as gasometer},
	Capture, Config, Context, CreateScheme, ExitError, ExitFatal, ExitReason, ExitSucceed,
	Handler as HandlerT, Opcode, Runtime, Stack, Transfer,
};
use sp_std::{
	cmp::min, collections::btree_map::BTreeMap, convert::Infallible, rc::Rc, vec, vec::Vec,
};

pub struct TraceExecutorWrapper<'config, S> {
	// Common parts.
	pub inner: &'config mut StackExecutor<'config, S>,
	is_tracing: bool,
	trace_type: TraceType,

	// Raw state.
	pub step_logs: Vec<RawStepLog>,

	// Blockscout state.
	pub entries: BTreeMap<u32, Call>,
	entries_next_index: u32,
	call_type: Option<CallType>,
	trace_address: Vec<u32>,
}

enum ContextType {
	Call,
	Create,
}

impl<'config, S: StackStateT<'config>> TraceExecutorWrapper<'config, S> {
	pub fn new(
		inner: &'config mut StackExecutor<'config, S>,
		is_tracing: bool,
		trace_type: TraceType,
	) -> TraceExecutorWrapper<'config, S> {
		TraceExecutorWrapper {
			inner,
			is_tracing,
			trace_type,
			step_logs: vec![],
			entries: BTreeMap::new(),
			entries_next_index: 0,
			call_type: None,
			trace_address: vec![],
		}
	}

	fn trace(
		&mut self,
		runtime: &mut Runtime,
		context_type: ContextType,
		code: Vec<u8>,
	) -> ExitReason {
		match self.trace_type {
			TraceType::Raw => self.trace_raw(runtime),
			TraceType::CallList => self.trace_call_list(runtime, context_type, code),
		}
	}

	fn trace_raw(&mut self, runtime: &mut Runtime) -> ExitReason {
		// TODO : If subcalls on a same contract access more storage, does it cache it here too ?
		// (not done yet)
		let mut storage_cache: BTreeMap<H256, H256> = BTreeMap::new();
		let address = runtime.context().address;

		loop {
			let mut storage_complete_scan = false;
			let mut storage_key_scan: Option<H256> = None;
			let mut steplog = None;

			if let Some((opcode, stack)) = runtime.machine().inspect() {
				// Will opcode modify storage
				if matches!(
					opcode,
					Opcode(0x54) | // sload
					Opcode(0x55) // sstore
				) {
					if let Ok(key) = stack.peek(0) {
						storage_key_scan = Some(key);
					}
				}

				// Any call might modify the storage values outside if this instance of the loop,
				// rendering the cache obsolete. In this case we'll refresh the cache after the next
				// step.
				storage_complete_scan = matches!(
					opcode,
					Opcode(240) | // create
					Opcode(241) | // call
					Opcode(242) | // call code
					Opcode(244) | // delegate call
					Opcode(245) | // create 2
					Opcode(250) // static call
				);

				let gas = self.inner.state().metadata().gasometer().gas();

				let gas_cost = match gasometer::static_opcode_cost(opcode) {
					Some(cost) => cost,
					_ => {
						match gasometer::dynamic_opcode_cost(
							runtime.context().address,
							opcode,
							stack,
							self.inner.state().metadata().is_static(),
							self.inner.config(),
							self,
						) {
							Ok((opcode_cost, _)) => match self
								.inner
								.state()
								.metadata()
								.gasometer()
								.gas_cost(opcode_cost, gas)
							{
								Ok(cost) => cost,
								Err(e) => return ExitReason::Error(e),
							},
							Err(e) => break ExitReason::Error(e),
						}
					}
				};
				let position = match runtime.machine().position() {
					Ok(p) => p,
					Err(reason) => break reason.clone(),
				};

				steplog = Some(RawStepLog {
					// EVM's returned depth is depth output format - 1.
					depth: U256::from(
						self.inner.state().metadata().depth().unwrap_or_default() + 1,
					),
					gas: U256::from(self.inner.gas()),
					gas_cost: U256::from(gas_cost),
					memory: {
						// Vec<u8> to Vec<H256> conversion.
						let memory = &runtime.machine().memory().data()[..];
						let size = 32;
						memory
							.chunks(size)
							.map(|c| {
								let mut msg = [0u8; 32];
								let chunk = c.len();
								if chunk < size {
									let left = size - chunk;
									let remainder = vec![0; left];
									msg[0..left].copy_from_slice(&remainder[..]);
									msg[left..size].copy_from_slice(c);
								} else {
									msg[0..size].copy_from_slice(c)
								}
								H256::from_slice(&msg[..])
							})
							.collect()
					},
					op: opcodes(opcode),
					pc: U256::from(*position),
					stack: runtime.machine().stack().data().clone(),
					storage: BTreeMap::new(),
				});
			}

			let step_result = runtime.step(self);

			// Update cache if needed.
			if let Some(key) = storage_key_scan {
				let _ = storage_cache.insert(key, self.storage(address, key));
			}

			if storage_complete_scan {
				for (key, value) in storage_cache.iter_mut() {
					*value = self.storage(address, *key);
				}
			}

			// Push log into vec here instead here (for SLOAD/STORE "early" update).
			if let Some(mut steplog) = steplog {
				steplog.storage = storage_cache.clone();
				self.step_logs.push(steplog);
			}

			// Do we continue ?
			match step_result {
				Ok(_) => {}
				Err(Capture::Exit(s)) => {
					break s;
				}
				Err(Capture::Trap(_)) => {
					break ExitReason::Fatal(ExitFatal::UnhandledInterrupt);
				}
			}
		}
	}

	fn trace_call_list(
		&mut self,
		runtime: &mut Runtime,
		context_type: ContextType,
		data: Vec<u8>,
	) -> ExitReason {
		// Starting new entry.
		//
		// traceAddress field matches this explanation :
		// https://openethereum.github.io/JSONRPC-trace-module#traceaddress-field
		//
		// We update "trace_address" for a potential subcall.
		// Will be popped at the end of this context.
		self.trace_address.push(0);

		let entries_index = self.entries_next_index;
		self.entries_next_index += 1;

		// Fetch all data we currently can for the entry.
		let call_type = self.call_type;
		let from = runtime.context().caller;
		let to = runtime.context().address;
		let value = runtime.context().apparent_value;

		let gas_at_start = self.inner.gas();
		let mut return_stack_offset = None;
		let mut return_stack_len = None;
		let mut suicide_info = None;

		// Execute the call/create.
		let exit_reason = loop {
			let mut subcall = false;

			if let Some((opcode, _stack)) = runtime.machine().inspect() {
				self.call_type = match opcode {
					Opcode(241) => Some(CallType::Call),
					Opcode(242) => Some(CallType::CallCode),
					Opcode(244) => Some(CallType::DelegateCall),
					Opcode(250) => Some(CallType::StaticCall),
					_ => None,
				};

				subcall = self.call_type.is_some();

				// RETURN
				if opcode == Opcode(0xf3) {
					let stack = runtime.machine().stack().data();

					return_stack_offset = stack.get(stack.len() - 1).cloned();
					return_stack_len = stack.get(stack.len() - 2).cloned();
				}

				// SELFDESTRUCT
				if opcode == Opcode(0xff) {
					let stack = runtime.machine().stack().data();

					suicide_info = stack
						.get(stack.len() - 1)
						.cloned()
						.map(|v| (H160::from(v), self.balance(runtime.context().address)));
				}
			}

			match runtime.step(self) {
				Ok(_) => {}
				Err(Capture::Exit(s)) => {
					break s;
				}
				Err(Capture::Trap(_)) => {
					break ExitReason::Fatal(ExitFatal::UnhandledInterrupt);
				}
			}

			if subcall {
				// We increase the last value of "trace_address" for a potential next subcall.
				*self.trace_address.last_mut().unwrap() += 1;
			}
		};

		// Compute used gas.
		let gas_at_end = self.inner.gas();
		let gas_used = gas_at_start - gas_at_end;

		// If `exit_reason` is `Suicided`, we need to add the suicide subcall to the traces.
		if exit_reason == ExitReason::Succeed(ExitSucceed::Suicided) {
			let entries_index = self.entries_next_index;
			self.entries_next_index += 1;

			let (refund_address, balance) = suicide_info.unwrap();

			self.entries.insert(
				entries_index,
				Call {
					from: to, // this contract is self destructing
					trace_address: self.trace_address.clone(),
					subtraces: 0,
					value,
					gas: U256::from(gas_at_end),
					gas_used: U256::from(gas_used),
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
			entries_index,
			match context_type {
				ContextType::Call => {
					let res = match &exit_reason {
						ExitReason::Succeed(ExitSucceed::Returned) => {
							CallResult::Output(runtime.machine().return_value())
						}
						ExitReason::Succeed(_) => CallResult::Output(vec![]),
						ExitReason::Error(error) => CallResult::Error(Self::error_message(error)),

						ExitReason::Revert(_) => CallResult::Error(b"execution reverted".to_vec()),
						ExitReason::Fatal(_) => CallResult::Error(vec![]),
					};

					Call {
						from,
						trace_address: self.trace_address.clone(),
						subtraces,
						value,
						gas: U256::from(gas_at_end),
						gas_used: U256::from(gas_used),
						inner: CallInner::Call {
							call_type: call_type.expect("should always have a call type"),
							to,
							input: data,
							res,
						},
					}
				}
				ContextType::Create => {
					// let offset = runtine.machine().stack().data();
					let contract_code = if let (Some(offset), Some(len)) =
						(return_stack_offset, return_stack_len)
					{
						let offset = offset.to_low_u64_be() as usize;
						let len = len.to_low_u64_be() as usize;

						let memory = runtime.machine().memory().data();

						if memory.len() >= offset + len {
							memory[offset..offset + len].to_vec()
						} else {
							vec![] // TODO : Should not be possible
						}
					} else {
						vec![] // TODO : Should not be possible
					};

					let res = match &exit_reason {
						ExitReason::Succeed(_) => CreateResult::Success {
							created_contract_address_hash: to,
							created_contract_code: contract_code,
						},
						ExitReason::Error(error) => CreateResult::Error {
							error: Self::error_message(error),
						},

						ExitReason::Revert(_) => CreateResult::Error {
							error: b"execution reverted".to_vec(),
						},
						ExitReason::Fatal(_) => CreateResult::Error { error: vec![] },
					};

					Call {
						value,
						trace_address: self.trace_address.clone(),
						subtraces,
						gas: U256::from(gas_at_end),
						gas_used: U256::from(gas_used),
						from,
						inner: CallInner::Create { init: data, res },
					}
				}
			},
		);

		exit_reason
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

	pub fn trace_call(
		&mut self,
		address: H160,
		transfer: Option<Transfer>,
		data: Vec<u8>,
		target_gas: Option<u64>,
		is_static: bool,
		take_l64: bool,
		take_stipend: bool,
		context: Context,
	) -> Capture<(ExitReason, Vec<u8>), Infallible> {
		macro_rules! try_or_fail {
			( $e:expr ) => {
				match $e {
					Ok(v) => v,
					Err(e) => return Capture::Exit((e.into(), Vec::new())),
				}
			};
		}

		// let after_gas = self.inner.state().metadata().gasometer().gas();
		fn l64(gas: u64) -> u64 {
			gas - gas / 64
		}

		let after_gas = if take_l64 && self.inner.config().call_l64_after_gas {
			if self.inner.config().estimate {
				let initial_after_gas = self.inner.state().metadata().gasometer().gas();
				let diff = initial_after_gas - l64(initial_after_gas);
				try_or_fail!(self
					.inner
					.state_mut()
					.metadata_mut()
					.gasometer_mut()
					.record_cost(diff));
				self.inner.state().metadata().gasometer().gas()
			} else {
				l64(self.inner.state().metadata().gasometer().gas())
			}
		} else {
			self.inner.state().metadata().gasometer().gas()
		};
		let target_gas = target_gas.unwrap_or(after_gas);
		let mut gas_limit = min(target_gas, after_gas);

		try_or_fail!(self
			.inner
			.state_mut()
			.metadata_mut()
			.gasometer_mut()
			.record_cost(gas_limit));

		if let Some(transfer) = transfer.as_ref() {
			if take_stipend && transfer.value != U256::zero() {
				gas_limit = gas_limit.saturating_add(self.inner.config().call_stipend);
			}
		}

		let code = self.inner.code(address);
		self.inner.enter_substate(gas_limit, is_static);
		self.inner.state_mut().touch(context.address);

		if let Some(depth) = self.inner.state().metadata().depth() {
			if depth > self.inner.config().call_stack_limit {
				let _ = self.inner.exit_substate(StackExitKind::Reverted);
				return Capture::Exit((ExitError::CallTooDeep.into(), Vec::new()));
			}
		}

		if let Some(transfer) = transfer {
			match self.inner.state_mut().transfer(transfer) {
				Ok(()) => (),
				Err(e) => {
					let _ = self.inner.exit_substate(StackExitKind::Reverted);
					return Capture::Exit((ExitReason::Error(e), Vec::new()));
				}
			}
		}

		let mut runtime = Runtime::new(
			Rc::new(code),
			Rc::new(data.clone()),
			context,
			self.inner.config(),
		);

		self.call_type = Some(CallType::Call);
		match self.trace(&mut runtime, ContextType::Call, data) {
			ExitReason::Succeed(s) => {
				let _ = self.inner.exit_substate(StackExitKind::Succeeded);
				Capture::Exit((ExitReason::Succeed(s), runtime.machine().return_value()))
			}
			ExitReason::Error(e) => {
				let _ = self.inner.exit_substate(StackExitKind::Failed);
				Capture::Exit((ExitReason::Error(e), Vec::new()))
			}
			ExitReason::Revert(e) => {
				let _ = self.inner.exit_substate(StackExitKind::Reverted);
				Capture::Exit((ExitReason::Revert(e), runtime.machine().return_value()))
			}
			ExitReason::Fatal(e) => {
				self.inner.state_mut().metadata_mut().gasometer_mut().fail();
				let _ = self.inner.exit_substate(StackExitKind::Failed);
				Capture::Exit((ExitReason::Fatal(e), Vec::new()))
			}
		}
	}

	pub fn trace_create(
		&mut self,
		caller: H160,
		scheme: CreateScheme,
		value: U256,
		code: Vec<u8>,
		target_gas: Option<u64>,
	) -> Capture<(ExitReason, Option<H160>, Vec<u8>), Infallible> {
		macro_rules! try_or_fail {
			( $e:expr ) => {
				match $e {
					Ok(v) => v,
					Err(e) => return Capture::Exit((e.into(), None, Vec::new())),
				}
			};
		}

		if let Some(depth) = self.inner.state().metadata().depth() {
			if depth > self.inner.config().call_stack_limit {
				return Capture::Exit((ExitError::CallTooDeep.into(), None, Vec::new()));
			}
		}

		let after_gas = self.inner.state().metadata().gasometer().gas();
		let target_gas = target_gas.unwrap_or(after_gas);
		let gas_limit = min(target_gas, after_gas);

		try_or_fail!(self
			.inner
			.state_mut()
			.metadata_mut()
			.gasometer_mut()
			.record_cost(gas_limit));
		let address = self.inner.create_address(scheme);
		self.inner.state_mut().inc_nonce(caller);
		self.inner.enter_substate(gas_limit, false);

		let context = Context {
			address,
			caller,
			apparent_value: value,
		};

		let transfer = Transfer {
			source: caller,
			target: address,
			value,
		};

		match self.inner.state_mut().transfer(transfer) {
			Ok(()) => (),
			Err(e) => {
				let _ = self.inner.exit_substate(StackExitKind::Reverted);
				return Capture::Exit((ExitReason::Error(e), None, Vec::new()));
			}
		}

		let mut runtime = Runtime::new(
			Rc::new(code.clone()),
			Rc::new(Vec::new()),
			context,
			self.inner.config(),
		);

		match self.trace(&mut runtime, ContextType::Create, code) {
			ExitReason::Succeed(s) => {
				let out = runtime.machine().return_value();

				if let Some(limit) = self.inner.config().create_contract_limit {
					if out.len() > limit {
						self.inner.state_mut().metadata_mut().gasometer_mut().fail();
						let _ = self.inner.exit_substate(StackExitKind::Failed);
						return Capture::Exit((
							ExitError::CreateContractLimit.into(),
							None,
							Vec::new(),
						));
					}
				}

				match self
					.inner
					.state_mut()
					.metadata_mut()
					.gasometer_mut()
					.record_deposit(out.len())
				{
					Ok(()) => {
						let e = self.inner.exit_substate(StackExitKind::Succeeded);
						self.inner.state_mut().set_code(address, out);
						try_or_fail!(e);
						Capture::Exit((ExitReason::Succeed(s), Some(address), Vec::new()))
					}
					Err(e) => {
						let _ = self.inner.exit_substate(StackExitKind::Failed);
						Capture::Exit((ExitReason::Error(e), None, Vec::new()))
					}
				}
			}
			ExitReason::Error(e) => Capture::Exit((ExitReason::Error(e), None, Vec::new())),
			ExitReason::Revert(e) => Capture::Exit((
				ExitReason::Revert(e),
				None,
				runtime.machine().return_value(),
			)),
			ExitReason::Fatal(e) => Capture::Exit((ExitReason::Fatal(e), None, Vec::new())),
		}
	}
}

impl<'config, S: StackStateT<'config>> HandlerT for TraceExecutorWrapper<'config, S> {
	type CreateInterrupt = Infallible;
	type CreateFeedback = Infallible;
	type CallInterrupt = Infallible;
	type CallFeedback = Infallible;

	fn balance(&self, address: H160) -> U256 {
		self.inner.balance(address)
	}

	fn code_size(&self, address: H160) -> U256 {
		self.inner.code_size(address)
	}

	fn code_hash(&self, address: H160) -> H256 {
		self.inner.code_hash(address)
	}

	fn code(&self, address: H160) -> Vec<u8> {
		self.inner.code(address)
	}

	fn storage(&self, address: H160, index: H256) -> H256 {
		self.inner.storage(address, index)
	}

	fn original_storage(&self, address: H160, index: H256) -> H256 {
		self.inner.original_storage(address, index)
	}

	fn exists(&self, address: H160) -> bool {
		self.inner.exists(address)
	}

	fn gas_left(&self) -> U256 {
		self.inner.gas_left()
	}

	fn gas_price(&self) -> U256 {
		self.inner.state().gas_price()
	}
	fn origin(&self) -> H160 {
		self.inner.state().origin()
	}
	fn block_hash(&self, number: U256) -> H256 {
		self.inner.state().block_hash(number)
	}
	fn block_number(&self) -> U256 {
		self.inner.state().block_number()
	}
	fn block_coinbase(&self) -> H160 {
		self.inner.state().block_coinbase()
	}
	fn block_timestamp(&self) -> U256 {
		self.inner.state().block_timestamp()
	}
	fn block_difficulty(&self) -> U256 {
		self.inner.state().block_difficulty()
	}
	fn block_gas_limit(&self) -> U256 {
		self.inner.state().block_gas_limit()
	}
	fn chain_id(&self) -> U256 {
		self.inner.state().chain_id()
	}

	fn deleted(&self, address: H160) -> bool {
		self.inner.deleted(address)
	}

	fn set_storage(&mut self, address: H160, index: H256, value: H256) -> Result<(), ExitError> {
		self.inner.set_storage(address, index, value)
	}

	fn log(&mut self, address: H160, topics: Vec<H256>, data: Vec<u8>) -> Result<(), ExitError> {
		self.inner.log(address, topics, data)
	}

	fn mark_delete(&mut self, address: H160, target: H160) -> Result<(), ExitError> {
		self.inner.mark_delete(address, target)
	}

	fn create(
		&mut self,
		caller: H160,
		scheme: CreateScheme,
		value: U256,
		init_code: Vec<u8>,
		target_gas: Option<u64>,
	) -> Capture<(ExitReason, Option<H160>, Vec<u8>), Self::CreateInterrupt> {
		if self.is_tracing {
			self.trace_create(caller, scheme, value, init_code, target_gas)
		} else {
			unreachable!("TODO StackExecutorWrapper only available on tracing enabled.");
		}
	}

	fn call(
		&mut self,
		code_address: H160,
		transfer: Option<Transfer>,
		input: Vec<u8>,
		target_gas: Option<u64>,
		is_static: bool,
		context: Context,
	) -> Capture<(ExitReason, Vec<u8>), Self::CallInterrupt> {
		if self.is_tracing {
			self.trace_call(
				code_address,
				transfer,
				input,
				target_gas,
				is_static,
				true,
				true,
				context,
			)
		} else {
			unreachable!("TODO StackExecutorWrapper only available on tracing enabled.");
		}
	}

	fn pre_validate(
		&mut self,
		context: &Context,
		opcode: Opcode,
		stack: &Stack,
	) -> Result<(), ExitError> {
		self.inner.pre_validate(context, opcode, stack)
	}
}
