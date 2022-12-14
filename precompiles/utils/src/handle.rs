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
	/// Returns a reader of the input, skipping the selector.
	fn read_after_selector(&self) -> MayRevert<EvmDataReader> {
		EvmDataReader::new_skip_selector(self.input())
	}
}
