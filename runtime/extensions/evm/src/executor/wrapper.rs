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
use moonbeam_rpc_primitives_debug::StepLog;
use sp_std::{collections::btree_map::BTreeMap, convert::Infallible, rc::Rc, vec, vec::Vec};

pub struct TraceExecutorWrapper<'config, S> {
	pub inner: &'config mut StackExecutor<'config, S>,
	is_tracing: bool,
	pub step_logs: Vec<StepLog>,
}

impl<'config, S: StackStateT<'config>> TraceExecutorWrapper<'config, S> {
	pub fn new(
		inner: &'config mut StackExecutor<'config, S>,
		is_tracing: bool,
	) -> TraceExecutorWrapper<'config, S> {
		TraceExecutorWrapper {
			inner,
			is_tracing,
			step_logs: Vec::new(),
		}
	}
	fn trace(&mut self, runtime: &mut Runtime) -> ExitReason {
		loop {
			if let Some((opcode, stack)) = runtime.machine().inspect() {
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

				self.step_logs.push(StepLog {
					depth: U256::from(self.inner.state().metadata().depth().unwrap_or_default()),
					gas: U256::from(self.inner.gas()),
					gas_cost: U256::from(gas_cost),
					memory: {
						// Vec<u8> to Vec<H256> conversion.
						let memory = &runtime.machine().memory().data().clone()[..];
						let size: usize = 32;
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
					storage: BTreeMap::new(), // TODO support this
				});
			}

			match runtime.step(self) {
				Ok(_) => continue,
				Err(Capture::Exit(s)) => {
					break s;
				}
				Err(Capture::Trap(_)) => {
					break ExitReason::Fatal(ExitFatal::UnhandledInterrupt);
				}
			}
		}
	}

	pub fn trace_call(
		&mut self,
		caller: H160,
		address: H160,
		transfer: Option<Transfer>,
		value: U256,
		data: Vec<u8>,
		gas_limit: u64,
	) -> Capture<(ExitReason, Vec<u8>), Infallible> {
		macro_rules! try_or_fail {
			( $e:expr ) => {
				match $e {
					Ok(v) => v,
					Err(e) => return Capture::Exit((e.into(), Vec::new())),
					}
			};
		}

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

		let mut runtime = Runtime::new(Rc::new(code), Rc::new(data), context, self.inner.config());

		match self.trace(&mut runtime) {
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
		gas_limit: u64,
	) -> Capture<(ExitReason, Option<H160>, Vec<u8>), Infallible> {
		macro_rules! try_or_fail {
			( $e:expr ) => {
				match $e {
					Ok(v) => v,
					Err(e) => return Capture::Exit((e.into(), None, Vec::new())),
					}
			};
		}

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
			Rc::new(code),
			Rc::new(Vec::new()),
			context,
			self.inner.config(),
		);

		match self.trace(&mut runtime) {
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
			let gas_limit = if let Some(gas) = target_gas {
				gas
			} else {
				u64::MAX
			};
			return self.trace_create(caller, value, init_code, gas_limit);
		} else {
			unreachable!("TODO StackExecutorWrapper only available on tracing enabled.");
		}
		// self.inner
		// 	.create_inner(caller, scheme, value, init_code, target_gas, true)
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
			let gas_limit = if let Some(gas) = target_gas {
				gas
			} else {
				u64::MAX
			};
			return self.trace_call(
				caller,
				code_address,
				transfer.clone(),
				value,
				input,
				gas_limit,
			);
		} else {
			unreachable!("TODO StackExecutorWrapper only available on tracing enabled.");
		}
		// self.inner.call_inner(
		// 	code_address,
		// 	transfer,
		// 	input,
		// 	target_gas,
		// 	is_static,
		// 	true,
		// 	true,
		// 	context,
		// )
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
