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

use super::*;
use core::assert_matches::assert_matches;
use fp_evm::{ExitSucceed, PrecompileOutput, PrecompileResult, PrecompileSet};

/// Mock handle to write tests for precompiles.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MockHandle {
	pub gas_limit: u64,
	pub gas_used: u64,
	pub logs: Vec<Log>,
}

impl MockHandle {
	pub fn new() -> Self {
		Self {
			gas_limit: u64::MAX,
			gas_used: 0,
			logs: vec![],
		}
	}

	pub fn with_gas_limit(gas_limit: u64) -> Self {
		Self {
			gas_limit,
			gas_used: 0,
			logs: vec![],
		}
	}
}

impl PrecompileHandle for MockHandle {
	/// Perform subcall in provided context.
	/// Precompile specifies in which context the subcall is executed.
	fn call(
		&mut self,
		_: H160,
		_: Option<Transfer>,
		_: Vec<u8>,
		_: Option<u64>,
		_: bool,
		_: &Context,
	) -> (ExitReason, Vec<u8>) {
		unimplemented!("sub calls are not supported in mock");
		// TODO : Allow Mock to store a Fn that can be called here.
		// Tests could provide a function that could inspect data,
		// register that it has been called, etc.
	}

	fn record_cost(&mut self, cost: u64) -> Result<(), ExitError> {
		self.gas_used += cost;

		if self.gas_used > self.gas_limit {
			Err(ExitError::OutOfGas)
		} else {
			Ok(())
		}
	}

	fn remaining_gas(&self) -> u64 {
		self.gas_limit - self.gas_used
	}

	fn log(&mut self, address: H160, topics: Vec<H256>, data: Vec<u8>) {
		self.logs.push(Log {
			address,
			topics,
			data,
		})
	}
}

pub struct PrecompilesTester<'p, P> {
	precompiles: &'p P,
	to: H160,
	data: Vec<u8>,
	target_gas: Option<u64>,
	context: Context,
	is_static: bool,

	expected_cost: Option<u64>,
	expected_logs: Option<Vec<Log>>,
}

impl<'p, P: PrecompileSet> PrecompilesTester<'p, P> {
	pub fn new(
		precompiles: &'p P,
		from: impl Into<H160>,
		to: impl Into<H160>,
		data: Vec<u8>,
	) -> Self {
		let to = to.into();
		Self {
			precompiles,
			to: to.clone(),
			data,
			target_gas: None,
			context: Context {
				address: to,
				caller: from.into(),
				apparent_value: U256::zero(),
			},
			is_static: false,

			expected_cost: None,
			expected_logs: None,
		}
	}

	pub fn with_value(mut self, value: impl Into<U256>) -> Self {
		self.context.apparent_value = value.into();
		self
	}

	pub fn expect_cost(mut self, cost: u64) -> Self {
		self.expected_cost = Some(cost);
		self
	}

	pub fn expect_no_logs(mut self) -> Self {
		self.expected_logs = Some(vec![]);
		self
	}

	pub fn expect_log(mut self, log: Log) -> Self {
		self.expected_logs = Some({
			let mut logs = self.expected_logs.unwrap_or_else(Vec::new);
			logs.push(log);
			logs
		});
		self
	}

	fn assert_optionals(&self, handle: &MockHandle) {
		if let Some(cost) = &self.expected_cost {
			assert_eq!(&handle.gas_used, cost);
		}

		if let Some(logs) = &self.expected_logs {
			assert_eq!(&handle.logs, logs);
		}
	}

	fn execute(&self) -> (Option<PrecompileResult>, MockHandle) {
		let mut handle = MockHandle::new();
		let res = self.precompiles.execute(
			&mut handle,
			self.to,
			&self.data,
			self.target_gas,
			&self.context,
			self.is_static,
		);
		(res, handle)
	}

	/// Execute the precompile set and expect some precompile to have been executed, regardless of the
	/// result.
	pub fn execute_some(self) {
		let (res, handle) = self.execute();
		assert!(res.is_some());
		self.assert_optionals(&handle);
	}

	/// Execute the precompile set and expect no precompile to have been executed.
	pub fn execute_none(self) {
		let (res, handle) = self.execute();
		assert!(res.is_some());
		self.assert_optionals(&handle);
	}

	/// Execute the precompile set and check it returns provided output.
	pub fn execute_returns(self, output: Vec<u8>) {
		let (res, handle) = self.execute();
		assert_eq!(
			res,
			Some(Ok(PrecompileOutput {
				exit_status: ExitSucceed::Returned,
				output
			}))
		);
		self.assert_optionals(&handle);
	}

	/// Execute the precompile set and check if it reverts.
	/// Take a closure allowing to perform custom matching on the output.
	pub fn execute_reverts(self, check: impl Fn(&[u8]) -> bool) {
		let (res, handle) = self.execute();
		assert_matches!(
			res,
			Some(Err(PrecompileFailure::Revert { output, ..}))
				if check(&output)
		);
		self.assert_optionals(&handle);
	}
}

pub trait PrecompileTesterExt: PrecompileSet + Sized {
	fn prepare_test(
		&self,
		from: impl Into<H160>,
		to: impl Into<H160>,
		data: Vec<u8>,
	) -> PrecompilesTester<Self>;
}

impl<T: PrecompileSet> PrecompileTesterExt for T {
	fn prepare_test(
		&self,
		from: impl Into<H160>,
		to: impl Into<H160>,
		data: Vec<u8>,
	) -> PrecompilesTester<Self> {
		PrecompilesTester::new(self, from, to, data)
	}
}
