use crate::executor::stack::TraceExecutor as TraceExecutorT;
use evm::{executor::StackExecutor, Config as EvmConfig, ExitReason};
use moonbeam_rpc_primitives_debug::TraceExecutorResponse;
use pallet_evm::{
	runner::stack::{Backend, Runner},
	CallInfo, Config, CreateInfo, Error, ExecutionInfo, PrecompileSet, Vicinity,
};
use sp_core::{H160, U256};
use sp_std::vec::Vec;

pub trait TraceRunner<T: Config> {
	type Error: Into<sp_runtime::DispatchError>;

	fn trace_execute<F>(
		source: H160,
		value: U256,
		gas_limit: u32,
		gas_price: Option<U256>,
		nonce: Option<U256>,
		config: &EvmConfig,
		f: F,
	) -> Result<TraceExecutorResponse, Error<T>>
	where
		F: FnOnce(&mut StackExecutor<Backend<T>>) -> TraceExecutorResponse;

	fn trace_call(
		source: H160,
		target: H160,
		input: Vec<u8>,
		value: U256,
		gas_limit: u32,
		gas_price: Option<U256>,
		nonce: Option<U256>,
		config: &EvmConfig,
	) -> Result<TraceExecutorResponse, Self::Error>;

	fn trace_create(
		source: H160,
		init: Vec<u8>,
		value: U256,
		gas_limit: u32,
		gas_price: Option<U256>,
		nonce: Option<U256>,
		config: &EvmConfig,
	) -> Result<TraceExecutorResponse, Self::Error>;
}

impl<T: Config> TraceRunner<T> for Runner<T> {
	type Error = Error<T>;
	fn trace_execute<F>(
		source: H160,
		value: U256,
		gas_limit: u32,
		gas_price: Option<U256>,
		nonce: Option<U256>,
		config: &EvmConfig,
		f: F,
	) -> Result<TraceExecutorResponse, Error<T>>
	where
		F: FnOnce(&mut StackExecutor<Backend<T>>) -> TraceExecutorResponse,
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

		Ok(f(&mut executor))
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
	) -> Result<TraceExecutorResponse, Self::Error> {
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
	) -> Result<TraceExecutorResponse, Self::Error> {
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
