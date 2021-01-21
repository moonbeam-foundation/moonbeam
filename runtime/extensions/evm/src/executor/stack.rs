extern crate alloc;
use alloc::string::ToString;
use ethereum_types::{H160, H256, U256};
pub use evm::{
	backend::{Backend as BackendT, Basic},
	executor::StackExecutor,
	gasometer::{self as gasometer},
	Capture, Context, CreateScheme, ExitError, ExitReason, ExitSucceed,
	ExternalOpcode as EvmExternalOpcode, Handler as HandlerT, Opcode as EvmOpcode, Runtime,
	Transfer,
};
use frame_support::debug;
use moonbeam_rpc_primitives_debug::{StepLog, TraceExecutorResponse};
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
	) -> Result<TraceExecutorResponse, ExitError>;

	fn trace_create(
		&mut self,
		caller: H160,
		value: U256,
		code: Vec<u8>,
		gas_limit: u64,
	) -> Result<TraceExecutorResponse, ExitError>;

	fn trace(
		&mut self,
		caller: H160,
		contract_address: H160,
		value: U256,
		code: Vec<u8>,
		data: Vec<u8>,
	) -> Result<TraceExecutorResponse, ExitError>;
}

impl<'backend, 'config, B: BackendT> TraceExecutor for StackExecutor<'backend, 'config, B> {
	fn trace(
		&mut self,
		caller: H160,
		contract_address: H160,
		value: U256,
		code: Vec<u8>,
		data: Vec<u8>,
	) -> Result<TraceExecutorResponse, ExitError> {
		let context = Context {
			caller,
			address: contract_address,
			apparent_value: value,
		};
		let mut runtime = Runtime::new(Rc::new(code), Rc::new(data), context, self.config);
		let mut step_logs = Vec::new();
		loop {
			if let Some((opcode, stack)) = runtime.machine().inspect() {
				let substate = self
					.substates
					.last()
					.expect("substate vec always have length greater than one; qed");

				let (opcode_cost, _memory_cost) = gasometer::opcode_cost(
					contract_address,
					opcode,
					stack,
					substate.is_static,
					&self.config,
					self,
				)?;

				let gasometer_instance = substate.gasometer.clone();
				let gas = gasometer_instance.gas();
				let gas_cost = gasometer_instance.inner?.gas_cost(opcode_cost, gas)?;
				let position = match &runtime.machine().position {
					Ok(p) => p,
					Err(reason) => match reason {
						ExitReason::Error(e) => return Err(e.clone()),
						_ => break,
					},
				};

				step_logs.push(StepLog {
					depth: U256::from(substate.depth.unwrap_or_default()),
					gas: U256::from(self.used_gas()),
					gas_cost: U256::from(gas_cost),
					memory: runtime.machine().memory().data.clone(),
					op: match opcode {
						Ok(i) => Opcode(i).to_string().as_bytes().to_vec(),
						Err(e) => ExternalOpcode(e).to_string().as_bytes().to_vec(),
					},
					pc: U256::from(*position),
					stack: runtime.machine().stack().data.clone(),
					storage: match self.account(contract_address) {
						Some(account) => account.storage.clone(),
						_ => BTreeMap::new(),
					},
				});
			} else {
				break;
			}

			match runtime.step(self) {
				Ok(_) => continue,
				Err(_) => break,
			}
		}

		Ok(TraceExecutorResponse {
			gas: U256::from(self.used_gas()),
			return_value: runtime.machine().return_value(),
			step_logs,
		})
	}

	fn trace_call(
		&mut self,
		caller: H160,
		address: H160,
		value: U256,
		data: Vec<u8>,
		gas_limit: u64,
	) -> Result<TraceExecutorResponse, ExitError> {
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
	) -> Result<TraceExecutorResponse, ExitError> {
		let scheme = CreateScheme::Legacy { caller };
		let address = self.create_address(scheme);
		self.enter_substate(gas_limit, false);
		self.trace(caller, address, value, code, Vec::new())
	}
}
