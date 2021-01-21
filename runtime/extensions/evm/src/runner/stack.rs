use crate::executor::stack::TraceExecutor as TraceExecutorT;
use ethereum_types::{H160, U256};
use evm::{executor::StackExecutor, Config as EvmConfig, ExitReason};
use moonbeam_rpc_primitives_debug::TraceExecutorResponse;
use pallet_evm::{
	runner::stack::{Backend, Runner},
	CallInfo, Config, CreateInfo, Error, ExecutionInfo, ExitError, PrecompileSet, Vicinity,
};
use sp_std::vec::Vec;

pub trait TraceRunner<T: Config> {
	fn trace_execute<F>(
		source: H160,
		value: U256,
		gas_limit: u32,
		gas_price: Option<U256>,
		nonce: Option<U256>,
		config: &EvmConfig,
		f: F,
	) -> Result<TraceExecutorResponse, ExitError>
	where
		F: FnOnce(&mut StackExecutor<Backend<T>>) -> Result<TraceExecutorResponse, ExitError>;

	fn trace_call(
		source: H160,
		target: H160,
		input: Vec<u8>,
		value: U256,
		gas_limit: u32,
		gas_price: Option<U256>,
		nonce: Option<U256>,
		config: &EvmConfig,
	) -> Result<TraceExecutorResponse, ExitError>;

	fn trace_create(
		source: H160,
		init: Vec<u8>,
		value: U256,
		gas_limit: u32,
		gas_price: Option<U256>,
		nonce: Option<U256>,
		config: &EvmConfig,
	) -> Result<TraceExecutorResponse, ExitError>;
}

impl<T: Config> TraceRunner<T> for Runner<T> {
	fn trace_execute<F>(
		source: H160,
		value: U256,
		gas_limit: u32,
		gas_price: Option<U256>,
		nonce: Option<U256>,
		config: &EvmConfig,
		f: F,
	) -> Result<TraceExecutorResponse, ExitError>
	where
		F: FnOnce(&mut StackExecutor<Backend<T>>) -> Result<TraceExecutorResponse, ExitError>,
	{
		let vicinity = Vicinity {
			gas_price: U256::zero(),
			origin: source,
		};

		let mut backend = Backend::<T>::new(&vicinity);
		let mut executor = StackExecutor::new_with_precompile(
			&backend,
			gas_limit as u64,
			config,
			T::Precompiles::execute,
		);

		f(&mut executor)
	}

	fn trace_call(
		source: H160,
		target: H160,
		input: Vec<u8>,
		value: U256,
		gas_limit: u32,
		gas_price: Option<U256>,
		nonce: Option<U256>,
		config: &EvmConfig,
	) -> Result<TraceExecutorResponse, ExitError> {
		Self::trace_execute(
			source,
			value,
			gas_limit,
			gas_price,
			nonce,
			config,
			|executor| executor.trace_call(source, target, value, input, gas_limit as u64),
		)
	}

	fn trace_create(
		source: H160,
		init: Vec<u8>,
		value: U256,
		gas_limit: u32,
		gas_price: Option<U256>,
		nonce: Option<U256>,
		config: &EvmConfig,
	) -> Result<TraceExecutorResponse, ExitError> {
		Self::trace_execute(
			source,
			value,
			gas_limit,
			gas_price,
			nonce,
			config,
			|executor| executor.trace_create(source, value, init, gas_limit as u64),
		)
	}
}
