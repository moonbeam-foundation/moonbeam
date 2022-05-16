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

use crate::command::{FullBackend, FullClient, TestContext};

use sc_service::NativeExecutionDispatch;
use service::{Block, RuntimeApiCollection};
use sp_api::ConstructRuntimeApi;

use cli_table::{format::Justify, Table};
use serde::Serialize;

mod block_creation;
mod fibonacci;
mod storage;
pub use block_creation::BlockCreationPerfTest;
pub use fibonacci::FibonacciPerfTest;
pub use storage::StoragePerfTest;

/// struct representing the test results of a single test
#[derive(Default, Clone, Table, Serialize)]
pub struct TestResults {
	#[table(title = "Test Name")]
	pub test_name: String,
	#[table(
		title = "Overall Time",
		display_fn = "display_duration_usecs",
		justify = "Justify::Right"
	)]
	pub overall_duration_usecs: u128,
	#[table(
		title = "Reference",
		display_fn = "display_duration_usecs",
		justify = "Justify::Right"
	)]
	pub reference_duration_usecs: u128,
	#[table(
		title = "Relative",
		display_fn = "display_relative",
		justify = "Justify::Right"
	)]
	pub relative: f64,
}

impl TestResults {
	pub fn new(name: &str, duration_usecs: u128, reference_duration_usecs: u128) -> Self {
		let ours = duration_usecs as f64;
		let reference = reference_duration_usecs as f64;

		std::assert!(reference > 0f64, "make sure reference is set and > 0");
		let relative = if ours > reference {
			// the reference is better -- negative % expected
			-(1f64 - (reference / ours))
		} else {
			// we beat the reference -- positive % expected
			(reference / ours) - 1f64
		};

		TestResults {
			test_name: name.into(),
			overall_duration_usecs: duration_usecs,
			reference_duration_usecs,
			relative,
		}
	}
}

pub trait TestRunner<RuntimeApi, Executor>
where
	RuntimeApi:
		ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>> + Send + Sync + 'static,
	RuntimeApi::RuntimeApi:
		RuntimeApiCollection<StateBackend = sc_client_api::StateBackendFor<FullBackend, Block>>,
	Executor: NativeExecutionDispatch + 'static,
{
	/// Return a globally unique name for this test. This is used as a filter on the command line
	/// so a CLI-friendly name is preferred.
	fn name(&self) -> String;

	/// Run the test
	fn run(
		&mut self,
		context: &TestContext<RuntimeApi, Executor>,
	) -> Result<Vec<TestResults>, String>;
}

fn display_duration_usecs(duration_usecs: &u128) -> impl std::fmt::Display {
	let ms = duration_usecs / 1000;
	let us = duration_usecs % 1000;
	let as_decimal: f64 = ms as f64 + (us as f64 / 1000.0);
	format!("{:.3} ms", as_decimal)
}

fn display_relative(relative: &f64) -> impl std::fmt::Display {
	format!("{:.1} %", relative * 100f64)
}
