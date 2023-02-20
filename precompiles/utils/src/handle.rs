// Copyright 2019-2022 PureStake Inc.
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

use {
	crate::{data::EvmDataReader, modifier::FunctionModifier, revert::MayRevert, EvmResult},
	fp_evm::{Log, PrecompileHandle},
};

pub trait PrecompileHandleExt: PrecompileHandle {
	/// Record cost of a log manually.
	/// This can be useful to record log costs early when their content have static size.
	#[must_use]
	fn record_log_costs_manual(&mut self, topics: usize, data_len: usize) -> EvmResult;

	/// Record cost of logs.
	#[must_use]
	fn record_log_costs(&mut self, logs: &[&Log]) -> EvmResult;

	#[must_use]
	/// Check that a function call is compatible with the context it is
	/// called into.
	fn check_function_modifier(&self, modifier: FunctionModifier) -> MayRevert;

	#[must_use]
	/// Read the selector from the input data.
	fn read_selector<T>(&self) -> MayRevert<T>
	where
		T: num_enum::TryFromPrimitive<Primitive = u32>;

	#[must_use]
	/// Read the selector from the input data.
	fn read_u32_selector(&self) -> MayRevert<u32>;

	#[must_use]
	/// Returns a reader of the input, skipping the selector.
	fn read_after_selector(&self) -> MayRevert<EvmDataReader>;
}

impl<T: PrecompileHandle> PrecompileHandleExt for T {
	/// Record cost of a log manualy.
	/// This can be useful to record log costs early when their content have static size.
	#[must_use]
	fn record_log_costs_manual(&mut self, topics: usize, data_len: usize) -> EvmResult {
		self.record_cost(crate::costs::log_costs(topics, data_len)?)?;

		Ok(())
	}

	/// Record cost of logs.
	#[must_use]
	fn record_log_costs(&mut self, logs: &[&Log]) -> EvmResult {
		for log in logs {
			self.record_log_costs_manual(log.topics.len(), log.data.len())?;
		}

		Ok(())
	}

	#[must_use]
	/// Check that a function call is compatible with the context it is
	/// called into.
	fn check_function_modifier(&self, modifier: FunctionModifier) -> MayRevert {
		crate::modifier::check_function_modifier(self.context(), self.is_static(), modifier)
	}

	#[must_use]
	/// Read the selector from the input data.
	fn read_selector<S>(&self) -> MayRevert<S>
	where
		S: num_enum::TryFromPrimitive<Primitive = u32>,
	{
		EvmDataReader::read_selector(self.input())
	}

	#[must_use]
	/// Read the selector from the input data as u32.
	fn read_u32_selector(&self) -> MayRevert<u32> {
		EvmDataReader::read_u32_selector(self.input())
	}

	#[must_use]
	/// Returns a reader of the input, skipping the selector.
	fn read_after_selector(&self) -> MayRevert<EvmDataReader> {
		EvmDataReader::new_skip_selector(self.input())
	}
}

environmental::environmental!(EVM_CONTEXT: trait PrecompileHandle);

pub fn using_precompile_handle<'a, R, F: FnOnce() -> R>(
	precompile_handle: &'a mut dyn PrecompileHandle,
	mutator: F,
) -> R {
	// # Safety
	//
	// unsafe rust does not mean unsafe, but "the compiler cannot guarantee the safety of the
	// memory".
	//
	// The only risk here is that the lifetime 'a comes to its end while the global variable
	// `EVM_CONTEXT` still contains the reference to the precompile handle.
	// The `using` method guarantee that it can't happen because the global variable is freed right
	// after the execution of the `mutator` closure (whatever the result of the execution).
	unsafe {
		EVM_CONTEXT::using(
			core::mem::transmute::<&'a mut dyn PrecompileHandle, &'static mut dyn PrecompileHandle>(
				precompile_handle,
			),
			mutator,
		)
	}
}

pub fn with_precompile_handle<R, F: FnOnce(&mut dyn PrecompileHandle) -> R>(f: F) -> Option<R> {
	EVM_CONTEXT::with(|precompile_handle| f(precompile_handle))
}

#[cfg(test)]
mod tests {
	use super::*;

	struct MockPrecompileHandle;
	impl PrecompileHandle for MockPrecompileHandle {
		fn call(
			&mut self,
			_: sp_core::H160,
			_: Option<evm::Transfer>,
			_: Vec<u8>,
			_: Option<u64>,
			_: bool,
			_: &evm::Context,
		) -> (evm::ExitReason, Vec<u8>) {
			unimplemented!()
		}

		fn record_cost(&mut self, _: u64) -> Result<(), evm::ExitError> {
			unimplemented!()
		}

		fn remaining_gas(&self) -> u64 {
			unimplemented!()
		}

		fn log(
			&mut self,
			_: sp_core::H160,
			_: Vec<sp_core::H256>,
			_: Vec<u8>,
		) -> Result<(), evm::ExitError> {
			unimplemented!()
		}

		fn code_address(&self) -> sp_core::H160 {
			unimplemented!()
		}

		fn input(&self) -> &[u8] {
			unimplemented!()
		}

		fn context(&self) -> &evm::Context {
			unimplemented!()
		}

		fn is_static(&self) -> bool {
			true
		}

		fn gas_limit(&self) -> Option<u64> {
			unimplemented!()
		}
	}

	#[test]
	fn with_precompile_handle_without_context() {
		assert_eq!(with_precompile_handle(|_| {}), None);
	}

	#[test]
	fn with_precompile_handle_with_context() {
		let mut precompile_handle = MockPrecompileHandle;

		assert_eq!(
			using_precompile_handle(&mut precompile_handle, || with_precompile_handle(
				|handle| handle.is_static()
			)),
			Some(true)
		);
	}
}
