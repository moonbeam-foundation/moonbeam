use crate::executor::stack::TraceExecutor as TraceExecutorT;
use evm::{executor::StackExecutor, Config as EvmConfig, ExitReason};
use pallet_evm::{
	runner::stack::{Backend, Runner},
	CallInfo, Config, CreateInfo, Error, ExecutionInfo, PrecompileSet, Vicinity,
};
use sp_core::{H160, U256};

pub trait TraceRunner<T: Config> {
	type Error: Into<sp_runtime::DispatchError>;

	fn trace_execute<F, R>(
		source: H160,
		value: U256,
		gas_limit: u32,
		gas_price: Option<U256>,
		nonce: Option<U256>,
		config: &EvmConfig,
		f: F,
	) -> Result<ExecutionInfo<R>, Error<T>>
	where
		F: FnOnce(&mut StackExecutor<Backend<T>>) -> (ExitReason, R);

	fn trace_call(
		source: H160,
		target: H160,
		input: Vec<u8>,
		value: U256,
		gas_limit: u32,
		gas_price: Option<U256>,
		nonce: Option<U256>,
		config: &EvmConfig,
	) -> Result<CallInfo, Self::Error>;

	fn trace_create(
		source: H160,
		init: Vec<u8>,
		value: U256,
		gas_limit: u32,
		gas_price: Option<U256>,
		nonce: Option<U256>,
		config: &EvmConfig,
	) -> Result<CreateInfo, Self::Error>;
}

impl<T: Config> TraceRunner<T> for Runner<T> {
	type Error = Error<T>;
	fn trace_execute<F, R>(
		source: H160,
		value: U256,
		gas_limit: u32,
		gas_price: Option<U256>,
		nonce: Option<U256>,
		config: &EvmConfig,
		f: F,
	) -> Result<ExecutionInfo<R>, Error<T>>
	where
		F: FnOnce(&mut StackExecutor<Backend<T>>) -> (ExitReason, R),
	{
		let gas_price = U256::zero(); // TODO price not really needed for this, or is it?

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

		// let total_fee = gas_price
		// 	.checked_mul(U256::from(gas_limit))
		// 	.ok_or(Error::<T>::FeeOverflow)?;
		// let total_payment = value
		// 	.checked_add(total_fee)
		// 	.ok_or(Error::<T>::PaymentOverflow)?;
		// let source_account = Module::<T>::account_basic(&source);
		// ensure!(
		// 	source_account.balance >= total_payment,
		// 	Error::<T>::BalanceLow
		// );
		// executor
		// 	.withdraw(source, total_fee)
		// 	.map_err(|_| Error::<T>::WithdrawFailed)?;

		// if let Some(nonce) = nonce {
		// 	ensure!(source_account.nonce == nonce, Error::<T>::InvalidNonce);
		// }

		let (reason, retv) = f(&mut executor);

		let used_gas = U256::from(executor.used_gas());

		Ok(ExecutionInfo {
			value: retv,
			exit_reason: reason,
			used_gas,
			logs: Vec::new(),
		})
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
	) -> Result<CallInfo, Self::Error> {
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
	) -> Result<CreateInfo, Self::Error> {
		Self::trace_execute(
			source,
			value,
			gas_limit,
			gas_price,
			nonce,
			config,
			|executor| {
				let address = executor.create_address(evm::CreateScheme::Legacy { caller: source });
				(
					executor.trace_create(source, value, init, gas_limit as u64),
					address,
				)
			},
		)
	}
}
