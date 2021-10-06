// Copyright 2019-2021 PureStake Inc.
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

use std::time::Duration;

use crate::command::{TestContext, FullClient, FullBackend};

use sp_api::{ConstructRuntimeApi, ProvideRuntimeApi, BlockId};
use service::{chain_spec, RuntimeApiCollection, Block};
use sc_service::{
	Configuration, NativeExecutionDispatch, TFullClient, TFullBackend, TaskManager, TransactionPool,
};

mod fibonacci;
mod block_creation;
mod storage;
pub use fibonacci::FibonacciPerfTest;
pub use block_creation::BlockCreationPerfTest;
pub use storage::StoragePerfTest;

/// struct representing the test results of a single test
#[derive(Default, Clone)]
pub struct TestResults {
	pub test_name: String,
	pub overall_duration: Duration,
}

impl TestResults {
    pub fn new(name: &str, duration: Duration) -> Self {
        TestResults {
            test_name: name.into(),
            overall_duration: duration,
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
	fn run(&mut self, context: &TestContext<RuntimeApi, Executor>) -> Result<Vec<TestResults>, String>;
}

