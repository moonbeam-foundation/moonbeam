// Copyright 2024 Moonbeam foundation
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

use sc_client_api::execution_extensions::ExecutionExtensions;
use sc_executor::{RuntimeVersion, RuntimeVersionOf};
use sp_api::ProofRecorder;
use sp_core::traits::{CallContext, Externalities};
use sp_runtime::traits::{Block as BlockT, HashingFor};
use sp_state_machine::{OverlayedChanges, StorageProof};
use std::cell::RefCell;
use std::marker::PhantomData;

/// Call executor that executes methods locally, querying all required
/// data from local backend.
#[derive(Clone)]
pub struct LazyLoadingCallExecutor<Block, Executor> {
	inner: Executor,
	_phantom_data: PhantomData<Block>,
}

impl<Block: BlockT, Executor> LazyLoadingCallExecutor<Block, Executor>
where
	Executor: sc_client_api::CallExecutor<Block> + Clone + 'static,
{
	/// Creates new instance of local call executor.
	pub fn new(executor: Executor) -> sp_blockchain::Result<Self> {
		Ok(LazyLoadingCallExecutor {
			inner: executor,
			_phantom_data: Default::default(),
		})
	}
}

impl<Block, Executor> sc_client_api::CallExecutor<Block>
	for LazyLoadingCallExecutor<Block, Executor>
where
	Executor: sc_client_api::CallExecutor<Block>,
	Block: BlockT,
{
	type Error = Executor::Error;

	type Backend = Executor::Backend;

	fn execution_extensions(&self) -> &ExecutionExtensions<Block> {
		&self.inner.execution_extensions()
	}

	fn call(
		&self,
		at_hash: Block::Hash,
		method: &str,
		call_data: &[u8],
		context: CallContext,
	) -> sp_blockchain::Result<Vec<u8>> {
		self.inner.call(at_hash, method, call_data, context)
	}

	fn contextual_call(
		&self,
		at_hash: Block::Hash,
		method: &str,
		call_data: &[u8],
		changes: &RefCell<OverlayedChanges<HashingFor<Block>>>,
		// not used in lazy loading
		_recorder: &Option<ProofRecorder<Block>>,
		call_context: CallContext,
		extensions: &RefCell<sp_externalities::Extensions>,
	) -> Result<Vec<u8>, sp_blockchain::Error> {
		self.inner.contextual_call(
			at_hash,
			method,
			call_data,
			changes,
			&None,
			call_context,
			extensions,
		)
	}

	fn runtime_version(&self, at_hash: Block::Hash) -> sp_blockchain::Result<RuntimeVersion> {
		sc_client_api::CallExecutor::runtime_version(&self.inner, at_hash)
	}

	fn prove_execution(
		&self,
		at_hash: Block::Hash,
		method: &str,
		call_data: &[u8],
	) -> sp_blockchain::Result<(Vec<u8>, StorageProof)> {
		self.inner.prove_execution(at_hash, method, call_data)
	}
}

impl<Block, Executor> RuntimeVersionOf for LazyLoadingCallExecutor<Block, Executor>
where
	Executor: RuntimeVersionOf,
	Block: BlockT,
{
	fn runtime_version(
		&self,
		ext: &mut dyn Externalities,
		runtime_code: &sp_core::traits::RuntimeCode,
	) -> Result<RuntimeVersion, sc_executor::error::Error> {
		self.inner.runtime_version(ext, runtime_code)
	}
}
