pub use evm::{
	backend::{Backend as BackendT, Basic},
	executor::StackExecutor,
	gasometer::{self as gasometer},
	Capture, Context, CreateScheme, ExitReason, ExitSucceed, Handler as HandlerT, Opcode, Runtime,
	Transfer,
};
use frame_support::debug;
use sp_core::{H160, H256, U256};
use sp_std::{cmp::min, collections::btree_map::BTreeMap, convert::Infallible, rc::Rc, vec::Vec};

#[derive(Debug)]
pub struct TraceExecutorResponse {
	gas: U256,
	return_value: Vec<u8>,
	step_logs: Vec<StepLog>,
}

#[derive(Debug)]
pub struct StepLog {
	depth: U256,
	//error: TODO
	gas: U256,
	gas_cost: U256,
	memory: Vec<u8>,
	op: Opcode,
	pc: U256,
	stack: Vec<H256>,
	//storage: BTreeMap<H256, H256>, TODO
}

pub trait TraceExecutor {
	fn trace_call(
		&mut self,
		caller: H160,
		address: H160,
		value: U256,
		data: Vec<u8>,
		gas_limit: u64,
	) -> TraceExecutorResponse;

	fn trace_create(
		&mut self,
		caller: H160,
		value: U256,
		init_code: Vec<u8>,
		gas_limit: u64,
	) -> TraceExecutorResponse;
}

impl<'backend, 'config, B: BackendT> TraceExecutor for StackExecutor<'backend, 'config, B> {
	fn trace_call(
		&mut self,
		caller: H160,
		address: H160,
		value: U256,
		data: Vec<u8>,
		gas_limit: u64,
	) -> TraceExecutorResponse {
		let context = Context {
			caller,
			address,
			apparent_value: value,
		};

		let code = self.code(address);

		self.enter_substate(gas_limit, false);
		self.account_mut(context.address);

		let mut runtime = Runtime::new(Rc::new(code), Rc::new(data), context, self.config);

		let mut step_logs = Vec::new();

		// Step opcodes
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

				let substate = self.substates.last().unwrap();

				let gasometer_instance = substate.gasometer.clone();

				let gas_cost = gasometer_instance
					.clone()
					.inner
					.unwrap()
					.gas_cost(opcode_cost, gasometer_instance.clone().gas());

				// TODO: what is the behaviour on Err(ExternalOpcode)? ignore? include?
				if let Ok(opcode) = opcode {
					step_logs.push(StepLog {
						depth: U256::from(substate.depth.unwrap()), //Some -> U256,
						gas: U256::from(self.used_gas()),           //U256,
						gas_cost: U256::from(gas_cost.unwrap()),    //Result->U256,
						memory: runtime.machine().memory().data.clone(), //Vec<u8>,
						op: opcode,                                 //Opcode,
						pc: U256::from(runtime.machine().position.clone().unwrap()), //Result -> U256,
						stack: runtime.machine().stack().data.clone(), //Vec<H256>,
					});
				}
			} else {
				break;
			}
			let step = runtime.step(self);
			// ...
		}

		TraceExecutorResponse {
			gas: U256::zero(),
			return_value: Vec::new(),
			step_logs,
		}
	}

	fn trace_create(
		&mut self,
		caller: H160,
		value: U256,
		init_code: Vec<u8>,
		gas_limit: u64,
	) -> TraceExecutorResponse {
		let scheme = CreateScheme::Legacy { caller };

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

		let mut step_logs = Vec::new();

		// Step opcodes
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

				let substate = self.substates.last().unwrap();

				let gasometer_instance = substate.gasometer.clone();

				let gas_cost = gasometer_instance
					.clone()
					.inner
					.unwrap()
					.gas_cost(opcode_cost, gasometer_instance.clone().gas());

				// TODO: what is the behaviour on Err(ExternalOpcode)? ignore? include?
				if let Ok(opcode) = opcode {
					step_logs.push(StepLog {
						depth: U256::from(substate.depth.unwrap()), //Some -> U256,
						gas: U256::from(self.used_gas()),           //U256,
						gas_cost: U256::from(gas_cost.unwrap()),    //Result->U256,
						memory: runtime.machine().memory().data.clone(), //Vec<u8>,
						op: opcode,                                 //Opcode,
						pc: U256::from(runtime.machine().position.clone().unwrap()), //Result -> U256,
						stack: runtime.machine().stack().data.clone(), //Vec<H256>,
					});
				}
			} else {
				break;
			}
			let step = runtime.step(self);
			// ...
		}

		TraceExecutorResponse {
			gas: U256::zero(),
			return_value: Vec::new(),
			step_logs,
		}
	}
}
