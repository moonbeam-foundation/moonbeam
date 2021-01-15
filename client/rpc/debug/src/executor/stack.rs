extern crate alloc;
use alloc::{collections::BTreeMap, rc::Rc};
use core::{cmp::min, convert::Infallible};
pub use evm::{
	backend::{Backend as BackendT, Basic},
	executor::StackExecutor,
	gasometer::{self as gasometer},
	Capture, Context, CreateScheme, ExitReason, ExitSucceed, Handler as HandlerT, Opcode, Runtime,
	Transfer,
};
use frame_support::debug;
use sp_core::{H160, H256, U256};

// TODO
pub struct StepLog {
	depth: U256,
	//error:
	gas: U256,
	gas_cost: U256,
	memory: Vec<H256>,
	op: Opcode,
	//pc:
	stack: Vec<H256>,
	storage: BTreeMap<H256, H256>,
}

pub trait TraceExecutor {
	fn trace_call(
		&mut self,
		caller: H160,
		address: H160,
		value: U256,
		data: Vec<u8>,
		gas_limit: usize,
	) -> (ExitReason, Vec<u8>);

	fn trace_call_inner(
		&mut self,
		code_address: H160,
		transfer: Option<Transfer>,
		input: Vec<u8>,
		target_gas: Option<usize>,
		is_static: bool,
		take_l64: bool,
		take_stipend: bool,
		context: Context,
	) -> Capture<(ExitReason, Vec<u8>), Infallible>;

	fn trace_create(
		&mut self,
		caller: H160,
		value: U256,
		init_code: Vec<u8>,
		gas_limit: usize,
	) -> ExitReason;

	fn trace_create_inner(
		&mut self,
		caller: H160,
		scheme: CreateScheme,
		value: U256,
		init_code: Vec<u8>,
		target_gas: Option<usize>,
		take_l64: bool,
	) -> Capture<(ExitReason, Option<H160>, Vec<u8>), Infallible>;
}

impl<'backend, 'config, B: BackendT> TraceExecutor for StackExecutor<'backend, 'config, B> {
	fn trace_call(
		&mut self,
		caller: H160,
		address: H160,
		value: U256,
		data: Vec<u8>,
		gas_limit: usize,
	) -> (ExitReason, Vec<u8>) {
		debug::debug!(
			target: "evm",
			"----> Call"
		);
		let current = self
			.substates
			.last_mut()
			.expect("substate vec always have length greater than one; qed");

		let transaction_cost = gasometer::call_transaction_cost(&data);
		match current.gasometer.record_transaction(transaction_cost) {
			Ok(()) => (),
			Err(e) => {} //return (e.into(), Vec::new()),
		}

		self.account_mut(caller).basic.nonce += U256::one();

		let context = Context {
			caller,
			address,
			apparent_value: value,
		};

		// ...

		match self.trace_call_inner(
			address,
			Some(Transfer {
				source: caller,
				target: address,
				value,
			}),
			data,
			Some(gas_limit),
			false,
			false,
			false,
			context,
		) {
			Capture::Exit((s, v)) => (s, v),
			Capture::Trap(_) => unreachable!(),
		}
	}

	fn trace_call_inner(
		&mut self,
		code_address: H160,
		transfer: Option<Transfer>,
		input: Vec<u8>,
		target_gas: Option<usize>,
		is_static: bool,
		take_l64: bool,
		take_stipend: bool,
		context: Context,
	) -> Capture<(ExitReason, Vec<u8>), Infallible> {
		macro_rules! try_or_fail {
			( $e:expr ) => {
				match $e {
					Ok(v) => v,
					Err(e) => {} //return Capture::Exit((e.into(), Vec::new())),
					}
			};
		}

		debug::debug!(
			target: "evm",
			"----> Call inner"
		);

		fn l64(gas: usize) -> usize {
			gas - gas / 64
		}

		let mut after_gas = self
			.substates
			.last()
			.expect("substate vec always have length greater than one; qed")
			.gasometer
			.gas();
		if take_l64 && self.config.call_l64_after_gas {
			after_gas = l64(after_gas);
		}

		let target_gas = target_gas.unwrap_or(after_gas);
		let mut gas_limit = min(target_gas, after_gas);

		try_or_fail!(self
			.substates
			.last_mut()
			.expect("substate vec always have length greater than one; qed")
			.gasometer
			.record_cost(gas_limit));

		if let Some(transfer) = transfer.as_ref() {
			if take_stipend && transfer.value != U256::zero() {
				gas_limit = gas_limit.saturating_add(self.config.call_stipend);
			}
		}

		let code = self.code(code_address);

		self.enter_substate(gas_limit, is_static);
		self.account_mut(context.address);

		let mut runtime = Runtime::new(Rc::new(code), Rc::new(input), context, self.config);

		let step = runtime.step(self);
		// ...

		debug::debug!(
			target: "evm",
			"----> Call inner exit"
		);
		Capture::Exit((ExitReason::Succeed(ExitSucceed::Returned), Vec::new()))
	}

	fn trace_create(
		&mut self,
		caller: H160,
		value: U256,
		init_code: Vec<u8>,
		gas_limit: usize,
	) -> ExitReason {
		debug::debug!(
			target: "evm",
			"----> Create"
		);
		let current = self
			.substates
			.last_mut()
			.expect("substate vec always have length greater than one; qed");

		let transaction_cost = gasometer::create_transaction_cost(&init_code);
		match current.gasometer.record_transaction(transaction_cost) {
			Ok(()) => (),
			Err(e) => {} //return e.into(),
		}

		// ...
		match self.trace_create_inner(
			caller,
			CreateScheme::Legacy { caller },
			value,
			init_code,
			Some(gas_limit),
			false,
		) {
			Capture::Exit((s, _, _)) => s,
			Capture::Trap(_) => unreachable!(),
		}
	}

	fn trace_create_inner(
		&mut self,
		caller: H160,
		scheme: CreateScheme,
		value: U256,
		init_code: Vec<u8>,
		target_gas: Option<usize>,
		take_l64: bool,
	) -> Capture<(ExitReason, Option<H160>, Vec<u8>), Infallible> {
		macro_rules! try_or_fail {
			( $e:expr ) => {
				match $e {
					Ok(v) => v,
					Err(e) => {} //return Capture::Exit((e.into(), None, Vec::new())),
					}
			};
		}

		debug::debug!(
			target: "evm",
			"----> Create inner"
		);

		fn l64(gas: usize) -> usize {
			gas - gas / 64
		}

		let after_gas = if take_l64 && self.config.call_l64_after_gas {
			if self.config.estimate {
				let last_substate = self
					.substates
					.last_mut()
					.expect("substate vec always have length greater than one; qed");
				let initial_after_gas = last_substate.gasometer.gas();
				let diff = initial_after_gas - l64(initial_after_gas);
				try_or_fail!(last_substate.gasometer.record_cost(diff));
				last_substate.gasometer.gas()
			} else {
				l64(self
					.substates
					.last()
					.expect("substate vec always have length greater than one; qed")
					.gasometer
					.gas())
			}
		} else {
			self.substates
				.last()
				.expect("substate vec always have length greater than one; qed")
				.gasometer
				.gas()
		};

		let target_gas = target_gas.unwrap_or(after_gas);

		let gas_limit = min(after_gas, target_gas);
		try_or_fail!(self
			.substates
			.last_mut()
			.expect("substate vec always have length greater than one; qed")
			.gasometer
			.record_cost(gas_limit));

		let address = self.create_address(scheme);
		self.account_mut(caller).basic.nonce += U256::one();

		self.enter_substate(gas_limit, false);

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

		if self.config.create_increase_nonce {
			self.account_mut(address).basic.nonce += U256::one();
		}

		let mut runtime = Runtime::new(
			Rc::new(init_code),
			Rc::new(Vec::new()),
			context,
			self.config,
		);

		// Process opcode
		loop {
			if let Some((opcode, stack)) = runtime.machine().inspect() {
				let is_static = self
					.substates
					.last()
					.expect("substate vec always have length greater than one; qed")
					.is_static;

				let (opcode_cost, _memory_cost) =
					gasometer::opcode_cost(address, opcode, stack, is_static, &self.config, self)
						.unwrap();

				let gasometer_instance = self.substates.last().unwrap().gasometer.clone();

				let gas_cost = gasometer_instance
					.clone()
					.inner
					.unwrap()
					.gas_cost(opcode_cost, gasometer_instance.clone().gas());

				debug::debug!(
					target: "evm",
					"!---> Opcode {:?}", opcode
				);
				debug::debug!(
					target: "evm",
					"!---> Opcode cost {:?}", gas_cost
				);
				debug::debug!(
					target: "evm",
					"!---> Used gas {:?}", self.used_gas()
				);
			} else {
				break;
			}
			let step = runtime.step(self);
			// ...
		}

		debug::debug!(
			target: "evm",
			"----> Create inner exit"
		);

		Capture::Exit((
			ExitReason::Succeed(ExitSucceed::Returned),
			Some(H160::default()),
			Vec::new(),
		))
	}
}
