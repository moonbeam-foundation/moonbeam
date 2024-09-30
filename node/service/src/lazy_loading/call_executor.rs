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

use crate::lazy_loading::wasm_override::WasmOverride;
use crate::lazy_loading::wasm_substitutes::WasmSubstitutes;
use moonbeam_cli_opt::LazyLoadingConfig;
use sc_client_api::{
	backend, call_executor::CallExecutor, execution_extensions::ExecutionExtensions, HeaderBackend,
};
use sc_executor::{NativeVersion, RuntimeVersion, RuntimeVersionOf};
use sc_service::ClientConfig;
use sp_api::ProofRecorder;
use sp_core::traits::{CallContext, CodeExecutor, Externalities, RuntimeCode};
use sp_runtime::{
	generic::BlockId,
	traits::{Block as BlockT, HashingFor},
};
use sp_state_machine::{backend::AsTrieBackend, Ext, OverlayedChanges, StateMachine, StorageProof};
use sp_version::{GetNativeVersion, GetRuntimeVersionAt};
use std::{cell::RefCell, path::PathBuf, sync::Arc};

/// Call executor that executes methods locally, querying all required
/// data from local backend.
pub struct LazyLoadingCallExecutor<Block: BlockT, B, E> {
	backend: Arc<B>,
	lazy_loading_config: LazyLoadingConfig,
	executor: E,
	wasm_override: Arc<Option<WasmOverride>>,
	wasm_substitutes: WasmSubstitutes<Block, E, B>,
	wasmtime_precompiled_path: Option<PathBuf>,
	execution_extensions: Arc<ExecutionExtensions<Block>>,
}

impl<Block: BlockT + sp_runtime::DeserializeOwned, B, E> LazyLoadingCallExecutor<Block, B, E>
where
	E: CodeExecutor + RuntimeVersionOf + Clone + 'static,
	B: backend::Backend<Block>,
	Block::Hash: From<sp_core::H256>,
{
	/// Creates new instance of local call executor.
	pub fn new(
		backend: Arc<B>,
		lazy_loading_config: &LazyLoadingConfig,
		executor: E,
		client_config: ClientConfig<Block>,
		execution_extensions: ExecutionExtensions<Block>,
	) -> sp_blockchain::Result<Self> {
		let wasm_override = client_config
			.wasm_runtime_overrides
			.as_ref()
			.map(|p| WasmOverride::new(p.clone(), &executor))
			.transpose()?;

		let wasm_substitutes = WasmSubstitutes::new(
			client_config.wasm_runtime_substitutes,
			executor.clone(),
			backend.clone(),
		)?;

		Ok(LazyLoadingCallExecutor {
			backend,
			lazy_loading_config: (*lazy_loading_config).clone(),
			executor,
			wasm_override: Arc::new(wasm_override),
			wasm_substitutes,
			wasmtime_precompiled_path: client_config.wasmtime_precompiled,
			execution_extensions: Arc::new(execution_extensions),
		})
	}

	/// Check if local runtime code overrides are enabled and one is available
	/// for the given `BlockId`. If yes, return it; otherwise return the same
	/// `RuntimeCode` instance that was passed.
	fn check_override<'a>(
		&'a self,
		onchain_code: RuntimeCode<'a>,
		state: &B::State,
		hash: Block::Hash,
	) -> sp_blockchain::Result<(RuntimeCode<'a>, RuntimeVersion)>
	where
		Block: BlockT,
		B: backend::Backend<Block>,
	{
		let on_chain_version = self.on_chain_runtime_version(&onchain_code, state)?;
		let code_and_version = if let Some(d) = self.wasm_override.as_ref().as_ref().and_then(|o| {
			o.get(
				&on_chain_version.spec_version,
				onchain_code.heap_pages,
				&on_chain_version.spec_name,
			)
		}) {
			log::debug!(target: "wasm_overrides", "using WASM override for block {}", hash);
			d
		} else if let Some(s) =
			self.wasm_substitutes
				.get(on_chain_version.spec_version, onchain_code.heap_pages, hash)
		{
			log::debug!(target: "wasm_substitutes", "Using WASM substitute for block {:?}", hash);
			s
		} else {
			log::debug!(
				target: "wasm_overrides",
				"Neither WASM override nor substitute available for block {hash}, using onchain code",
			);
			(onchain_code, on_chain_version)
		};

		Ok(code_and_version)
	}

	/// Returns the on chain runtime version.
	fn on_chain_runtime_version(
		&self,
		code: &RuntimeCode,
		state: &B::State,
	) -> sp_blockchain::Result<RuntimeVersion> {
		let mut overlay = OverlayedChanges::default();

		let mut ext = Ext::new(&mut overlay, state, None);

		self.executor
			.runtime_version(&mut ext, code)
			.map_err(|e| sp_blockchain::Error::VersionInvalid(e.to_string()))
	}
}

impl<Block: BlockT + sp_runtime::DeserializeOwned, B, E> Clone
	for LazyLoadingCallExecutor<Block, B, E>
where
	E: Clone,
{
	fn clone(&self) -> Self {
		LazyLoadingCallExecutor {
			backend: self.backend.clone(),
			lazy_loading_config: self.lazy_loading_config.clone(),
			executor: self.executor.clone(),
			wasm_override: self.wasm_override.clone(),
			wasm_substitutes: self.wasm_substitutes.clone(),
			wasmtime_precompiled_path: self.wasmtime_precompiled_path.clone(),
			execution_extensions: self.execution_extensions.clone(),
		}
	}
}

impl<B, E, Block> CallExecutor<Block> for LazyLoadingCallExecutor<Block, B, E>
where
	B: backend::Backend<Block>,
	E: CodeExecutor + RuntimeVersionOf + Clone + 'static,
	Block: BlockT + sp_runtime::DeserializeOwned,
	Block::Hash: From<sp_core::H256>,
{
	type Error = E::Error;

	type Backend = B;

	fn execution_extensions(&self) -> &ExecutionExtensions<Block> {
		&self.execution_extensions
	}

	fn call(
		&self,
		at_hash: Block::Hash,
		method: &str,
		call_data: &[u8],
		context: CallContext,
	) -> sp_blockchain::Result<Vec<u8>> {
		let mut changes = OverlayedChanges::default();
		let at_number = self
			.backend
			.blockchain()
			.expect_block_number_from_id(&BlockId::Hash(at_hash))?;
		let state = self.backend.state_at(at_hash)?;

		let state_runtime_code = sp_state_machine::backend::BackendRuntimeCode::new(&state);
		let runtime_code = state_runtime_code
			.runtime_code()
			.map_err(sp_blockchain::Error::RuntimeCode)?;

		let runtime_code = self.check_override(runtime_code, &state, at_hash)?.0;

		let mut extensions = self.execution_extensions.extensions(at_hash, at_number);

		let mut sm = StateMachine::new(
			&state,
			&mut changes,
			&self.executor,
			method,
			call_data,
			&mut extensions,
			&runtime_code,
			context,
		)
		.set_parent_hash(at_hash);

		sm.execute().map_err(Into::into)
	}

	fn contextual_call(
		&self,
		at_hash: Block::Hash,
		method: &str,
		call_data: &[u8],
		changes: &RefCell<OverlayedChanges<HashingFor<Block>>>,
		// TODO: Confirm that `recorder` is not needed.
		_recorder: &Option<ProofRecorder<Block>>,
		call_context: CallContext,
		extensions: &RefCell<sp_externalities::Extensions>,
	) -> Result<Vec<u8>, sp_blockchain::Error> {
		let state = self.backend.state_at(at_hash)?;

		let changes = &mut *changes.borrow_mut();

		// It is important to extract the runtime code here before we create the proof
		// recorder to not record it. We also need to fetch the runtime code from `state` to
		// make sure we use the caching layers.
		let state_runtime_code = sp_state_machine::backend::BackendRuntimeCode::new(&state);

		let runtime_code = state_runtime_code
			.runtime_code()
			.map_err(sp_blockchain::Error::RuntimeCode)?;
		let runtime_code = self.check_override(runtime_code, &state, at_hash)?.0;
		let mut extensions = extensions.borrow_mut();

		let mut state_machine = StateMachine::new(
			&state,
			changes,
			&self.executor,
			method,
			call_data,
			&mut extensions,
			&runtime_code,
			call_context,
		)
		.set_parent_hash(at_hash);
		state_machine.execute().map_err(Into::into)
	}

	fn runtime_version(&self, at_hash: Block::Hash) -> sp_blockchain::Result<RuntimeVersion> {
		let state = self.backend.state_at(at_hash)?;
		let state_runtime_code = sp_state_machine::backend::BackendRuntimeCode::new(&state);

		let runtime_code = state_runtime_code
			.runtime_code()
			.map_err(sp_blockchain::Error::RuntimeCode)?;
		self.check_override(runtime_code, &state, at_hash)
			.map(|(_, v)| v)
	}

	fn prove_execution(
		&self,
		at_hash: Block::Hash,
		method: &str,
		call_data: &[u8],
	) -> sp_blockchain::Result<(Vec<u8>, StorageProof)> {
		let at_number = self
			.backend
			.blockchain()
			.expect_block_number_from_id(&BlockId::Hash(at_hash))?;
		let state = self.backend.state_at(at_hash)?;

		let trie_backend = state.as_trie_backend();

		let state_runtime_code = sp_state_machine::backend::BackendRuntimeCode::new(trie_backend);
		let runtime_code = state_runtime_code
			.runtime_code()
			.map_err(sp_blockchain::Error::RuntimeCode)?;
		let runtime_code = self.check_override(runtime_code, &state, at_hash)?.0;

		sp_state_machine::prove_execution_on_trie_backend(
			trie_backend,
			&mut Default::default(),
			&self.executor,
			method,
			call_data,
			&runtime_code,
			&mut self.execution_extensions.extensions(at_hash, at_number),
		)
		.map_err(Into::into)
	}
}

impl<B, E, Block> RuntimeVersionOf for LazyLoadingCallExecutor<Block, B, E>
where
	E: RuntimeVersionOf,
	Block: BlockT,
{
	fn runtime_version(
		&self,
		ext: &mut dyn Externalities,
		runtime_code: &sp_core::traits::RuntimeCode,
	) -> Result<RuntimeVersion, sc_executor::error::Error> {
		RuntimeVersionOf::runtime_version(&self.executor, ext, runtime_code)
	}
}

impl<Block, B, E> GetRuntimeVersionAt<Block> for LazyLoadingCallExecutor<Block, B, E>
where
	B: backend::Backend<Block>,
	E: CodeExecutor + RuntimeVersionOf + Clone + 'static,
	Block: BlockT + sp_runtime::DeserializeOwned,
	Block::Hash: From<sp_core::H256>,
{
	fn runtime_version(&self, at: Block::Hash) -> Result<sp_version::RuntimeVersion, String> {
		CallExecutor::runtime_version(self, at).map_err(|e| e.to_string())
	}
}

impl<Block, B, E> GetNativeVersion for LazyLoadingCallExecutor<Block, B, E>
where
	B: backend::Backend<Block>,
	E: CodeExecutor + sp_version::GetNativeVersion + Clone + 'static,
	Block: BlockT + sp_runtime::DeserializeOwned,
{
	fn native_version(&self) -> &NativeVersion {
		self.executor.native_version()
	}
}
