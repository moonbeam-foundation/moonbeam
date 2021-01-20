extern crate alloc;
use alloc::string::ToString;
pub use evm::{
	backend::{Backend as BackendT, Basic},
	executor::StackExecutor,
	gasometer::{self as gasometer},
	Capture, Context, CreateScheme, ExitReason, ExitSucceed, ExternalOpcode as EvmExternalOpcode,
	Handler as HandlerT, Opcode as EvmOpcode, Runtime, Transfer,
};
use frame_support::debug;
use moonbeam_rpc_primitives_debug::{StepLog, TraceExecutorResponse};
use sp_core::{H160, H256, U256};
use sp_std::{cmp::min, collections::btree_map::BTreeMap, convert::Infallible, rc::Rc, vec::Vec};

macro_rules! displayable {
	($t:ty) => {
		impl sp_std::fmt::Display for $t {
			fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
				write!(f, "{:?}", self.0)
			}
		}
	};
}

#[derive(Debug)]
pub struct Opcode(EvmOpcode);

#[derive(Debug)]
pub struct ExternalOpcode(EvmExternalOpcode);

displayable!(Opcode);
displayable!(ExternalOpcode);

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
		code: Vec<u8>,
		gas_limit: u64,
	) -> TraceExecutorResponse;

	fn trace(
		&mut self,
		caller: H160,
		contract_address: H160,
		value: U256,
		code: Vec<u8>,
		data: Vec<u8>,
	) -> TraceExecutorResponse;
}

impl<'backend, 'config, B: BackendT> TraceExecutor for StackExecutor<'backend, 'config, B> {
	fn trace(
		&mut self,
		caller: H160,
		contract_address: H160,
		value: U256,
		code: Vec<u8>,
		data: Vec<u8>,
	) -> TraceExecutorResponse {
		let context = Context {
			caller,
			address: contract_address,
			apparent_value: value,
		};
		let mut runtime = Runtime::new(Rc::new(code), Rc::new(data), context, self.config);
		let mut step_logs = Vec::new();
		loop {
			if let Some((opcode, stack)) = runtime.machine().inspect() {
				let is_static = self
					.substates
					.last()
					.expect("substate vec always have length greater than one; qed")
					.is_static;

				let (opcode_cost, _memory_cost) = gasometer::opcode_cost(
					contract_address,
					opcode,
					stack,
					is_static,
					&self.config,
					self,
				)
				.unwrap();

				let substate = self.substates.last().unwrap();

				let gasometer_instance = substate.gasometer.clone();

				let gas_cost = gasometer_instance
					.clone()
					.inner
					.unwrap()
					.gas_cost(opcode_cost, gasometer_instance.clone().gas());

				step_logs.push(StepLog {
					depth: U256::from(substate.depth.unwrap()), //Some -> U256,
					gas: U256::from(self.used_gas()),           //U256,
					gas_cost: U256::from(gas_cost.unwrap()),    //Result->U256,
					memory: runtime.machine().memory().data.clone(), //Vec<u8>,
					op: match opcode {
						Ok(i) => Opcode(i).to_string().as_bytes().to_vec(),
						Err(e) => ExternalOpcode(e).to_string().as_bytes().to_vec(),
					}, // Result -> Vec<u8>
					pc: U256::from(runtime.machine().position.clone().unwrap()), //Result -> U256,
					stack: runtime.machine().stack().data.clone(), //Vec<H256>,
				});
			} else {
				break;
			}

			match runtime.step(self) {
				Ok(_) => continue,
				Err(_) => break,
			}
		}

		TraceExecutorResponse {
			gas: U256::from(self.used_gas()),
			return_value: runtime.machine().return_value(),
			step_logs,
		}
	}

	fn trace_call(
		&mut self,
		caller: H160,
		address: H160,
		value: U256,
		data: Vec<u8>,
		gas_limit: u64,
	) -> TraceExecutorResponse {
		let code = self.code(address);
		self.enter_substate(gas_limit, false);
		self.account_mut(address);
		self.trace(caller, address, value, code, data)
	}

	fn trace_create(
		&mut self,
		caller: H160,
		value: U256,
		code: Vec<u8>,
		gas_limit: u64,
	) -> TraceExecutorResponse {
		let scheme = CreateScheme::Legacy { caller };
		let address = self.create_address(scheme);
		self.enter_substate(gas_limit, false);
		self.trace(caller, address, value, code, Vec::new())
	}
}
