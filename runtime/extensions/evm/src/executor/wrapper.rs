extern crate alloc;
use crate::executor::util::opcodes;
use ethereum_types::{H160, H256, U256};
pub use evm::{
	backend::{Apply, Backend as BackendT, Log},
	executor::{StackExecutor, StackExitKind, StackState as StackStateT},
	gasometer::{self as gasometer},
	Capture, Config, Context, CreateScheme, ExitError, ExitFatal, ExitReason, ExitSucceed,
	Handler as HandlerT, Opcode, Runtime, Stack, Transfer,
};
use moonbeam_rpc_primitives_debug::{
	blockscout::{CallResult, CallType, Entry, EntryInner},
	StepLog, TraceType,
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
	pub step_logs: Vec<StepLog>,

	// Blockscout state.
	pub entries: BTreeMap<u32, Entry>,
	entries_next_index: u32,
	call_type: Option<CallType>,
	parent_index: Option<u32>,
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
			parent_index: None,
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
			TraceType::Blockscout => self.trace_blockscout(runtime, context_type, code),
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
				let gas_cost = match self.inner.state().metadata().gasometer().inner() {
					Ok(inner) => match gasometer::static_opcode_cost(opcode) {
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
								Ok((opcode_cost, _)) => match inner.gas_cost(opcode_cost, gas) {
									Ok(cost) => cost,
									Err(e) => return ExitReason::Error(e),
								},
								Err(e) => break ExitReason::Error(e),
							}
						}
					},
					Err(e) => return ExitReason::Error(e),
				};
				let position = match runtime.machine().position() {
					Ok(p) => p,
					Err(reason) => break reason.clone(),
				};

				steplog = Some(StepLog {
					depth: U256::from(self.inner.state().metadata().depth().unwrap_or_default()),
					gas: U256::from(self.inner.gas()),
					gas_cost: U256::from(gas_cost),
					memory: {
						// Vec<u8> to Vec<H256> conversion.
						let memory = &runtime.machine().memory().data().clone()[..];
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

			match runtime.step(self) {
				Ok(_) => {}
				Err(Capture::Exit(s)) => {
					break s;
				}
				Err(Capture::Trap(_)) => {
					break ExitReason::Fatal(ExitFatal::UnhandledInterrupt);
				}
			}

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
		}
	}

	fn trace_blockscout(
		&mut self,
		runtime: &mut Runtime,
		context_type: ContextType,
		data: Vec<u8>,
	) -> ExitReason {
		// Starting new entry.
		let parent_index = self.parent_index;

		let entries_index = self.entries_next_index;
		self.entries_next_index += 1;

		// Fetch all data we currently can for the entry.
		let call_type = self.call_type;
		let from = runtime.context().caller;
		let to = runtime.context().address;
		let value = runtime.context().apparent_value;

		let gas_at_start = self.inner.gas();

		// Execute the call/create.
		let exit_reason = loop {
			if let Some((opcode, _stack)) = runtime.machine().inspect() {
				self.call_type = match opcode {
					Opcode(241) => Some(CallType::Call),
					Opcode(242) => Some(CallType::CallCode),
					Opcode(244) => Some(CallType::DelegateCall),
					Opcode(250) => Some(CallType::StaticCall),
					_ => None,
				}
			}

			// Set parent index for possible subcall to get this context index.
			self.parent_index = Some(entries_index);

			match runtime.step(self) {
				Ok(_) => {}
				Err(Capture::Exit(s)) => {
					break s;
				}
				Err(Capture::Trap(_)) => {
					break ExitReason::Fatal(ExitFatal::UnhandledInterrupt);
				}
			}
		};

		// Compute used gas.
		let gas_at_end = self.inner.gas();
		let gas_used = gas_at_start - gas_at_end;

		// Insert entry.
		let trace_address = parent_index.map_or(vec![], |index| vec![index]);
		self.entries.insert(
			entries_index,
			match context_type {
				ContextType::Call => {
					let res = match &exit_reason {
						ExitReason::Succeed(ExitSucceed::Returned) => {
							CallResult::Output(runtime.machine().return_value())
						}
						ExitReason::Succeed(_) => CallResult::Output(vec![]),
						ExitReason::Error(please_use_me) => {
							CallResult::Error(b"insert error message here".to_vec())
						}
						ExitReason::Revert(_) => CallResult::Error(b"execution reverted".to_vec()),
						ExitReason::Fatal(_) => CallResult::Error(vec![]),
					};

					Entry {
						from,
						trace_address,
						value,
						gas: U256::from(gas_at_end),
						gas_used: U256::from(gas_used),
						inner: EntryInner::Call {
							call_type: call_type.expect("should always have a call type"),

							to,
							input: data,
							res,
						},
					}
				}
				ContextType::Create => {
					let contract_code = self.code(to);

					// TODO : Handle reverting create
					Entry {
						value,
						trace_address,
						gas: U256::from(gas_at_end),
						gas_used: U256::from(gas_used),
						from,
						inner: EntryInner::Create {
							init: data,
							created_contract_address_hash: to,
							created_contract_code: contract_code,
						},
					}
				}
			},
		);

		// If root context, add parent hierarchy
		if entries_index == 0 {
			for i in 1..self.entries_next_index {
				let mut entry = self.entries.remove(&i).unwrap();
				let parent_index = entry.trace_address[0];

				let parent_entry = self.entries.get(&parent_index).unwrap();
				entry.trace_address = parent_entry.trace_address.clone();
				entry.trace_address.push(parent_index);

				self.entries.insert(i, entry);
			}
		}

		exit_reason
	}

	pub fn trace_call(
		&mut self,
		caller: H160,
		address: H160,
		transfer: Option<Transfer>,
		value: U256,
		data: Vec<u8>,
		target_gas: Option<u64>,
	) -> Capture<(ExitReason, Vec<u8>), Infallible> {
		macro_rules! try_or_fail {
			( $e:expr ) => {
				match $e {
					Ok(v) => v,
					Err(e) => return Capture::Exit((e.into(), Vec::new())),
					}
			};
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

		let context = Context {
			caller,
			address,
			apparent_value: value,
		};

		let code = self.inner.code(address);
		self.inner.enter_substate(gas_limit, false);
		self.inner.state_mut().touch(address);

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

		match self.trace(&mut runtime, ContextType::Call, data) {
			ExitReason::Succeed(s) => {
				Capture::Exit((ExitReason::Succeed(s), runtime.machine().return_value()))
			}
			ExitReason::Error(e) => Capture::Exit((ExitReason::Error(e), Vec::new())),
			ExitReason::Revert(e) => {
				Capture::Exit((ExitReason::Revert(e), runtime.machine().return_value()))
			}
			ExitReason::Fatal(e) => Capture::Exit((ExitReason::Fatal(e), Vec::new())),
		}
	}

	pub fn trace_create(
		&mut self,
		caller: H160,
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

		let after_gas = self.inner.state().metadata().gasometer().gas();
		let target_gas = target_gas.unwrap_or(after_gas);
		let gas_limit = min(target_gas, after_gas);

		try_or_fail!(self
			.inner
			.state_mut()
			.metadata_mut()
			.gasometer_mut()
			.record_cost(gas_limit));
		let scheme = CreateScheme::Legacy { caller };
		let address = self.inner.create_address(scheme);
		self.inner.enter_substate(gas_limit, false);

		let context = Context {
			caller,
			address,
			apparent_value: value,
		};
		let mut runtime = Runtime::new(
			Rc::new(code.clone()),
			Rc::new(Vec::new()),
			context,
			self.inner.config(),
		);

		match self.trace(&mut runtime, ContextType::Create, code) {
			ExitReason::Succeed(s) => Capture::Exit((
				ExitReason::Succeed(s),
				Some(address),
				runtime.machine().return_value(),
			)),
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
		_scheme: CreateScheme,
		value: U256,
		init_code: Vec<u8>,
		target_gas: Option<u64>,
	) -> Capture<(ExitReason, Option<H160>, Vec<u8>), Self::CreateInterrupt> {
		if self.is_tracing {
			self.trace_create(caller, value, init_code, target_gas)
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
		_is_static: bool,
		_context: Context,
	) -> Capture<(ExitReason, Vec<u8>), Self::CallInterrupt> {
		if self.is_tracing {
			let (caller, value) = if let Some(transfer) = transfer.clone() {
				(transfer.source, transfer.value)
			} else {
				(code_address, U256::zero())
			};

			self.trace_call(caller, code_address, transfer, value, input, target_gas)
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
