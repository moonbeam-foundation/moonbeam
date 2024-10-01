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

use sp_blockchain::{CachedHeaderMetadata, HeaderMetadata};
use sp_core::storage::well_known_keys;
use sp_runtime::{
	generic::BlockId,
	traits::{Block as BlockT, HashingFor, Header as HeaderT, NumberFor, Zero},
	Justification, Justifications, StateVersion, Storage,
};
use sp_state_machine::{
	BackendTransaction, ChildStorageCollection, IndexOperation, StorageCollection, TrieBackend,
};
use std::future::Future;
use std::marker::PhantomData;
use std::ops::AddAssign;
use std::time::Duration;
use std::{
	collections::{HashMap, HashSet},
	ptr,
	sync::Arc,
};

use sc_client_api::{
	backend::{self, NewBlockState},
	blockchain::{self, BlockStatus, HeaderBackend},
	leaves::LeafSet,
	UsageInfo,
};

use jsonrpsee::http_client::HttpClient;
use sp_runtime::generic::SignedBlock;

use crate::chain_spec;
use crate::lazy_loading::lock::ReadWriteLock;
use crate::lazy_loading::state_overrides::StateEntry;
use crate::lazy_loading::{helpers, state_overrides};
use moonbeam_cli_opt::LazyLoadingConfig;
use moonbeam_core_primitives::BlockNumber;
use sc_client_api::StorageKey;
use sc_service::{Configuration, Error};
use serde::de::DeserializeOwned;
use sp_core::offchain::storage::InMemOffchainStorage;
use sp_core::{twox_128, H256};
use sp_rpc::list::ListOrValue;
use sp_rpc::number::NumberOrHex;
use sp_storage::{ChildInfo, StorageData};
use sp_trie::PrefixedMemoryDB;
use tokio_retry::strategy::FixedInterval;
use tokio_retry::Retry;

struct PendingBlock<B: BlockT> {
	block: StoredBlock<B>,
	state: NewBlockState,
}

#[derive(PartialEq, Eq, Clone)]
enum StoredBlock<B: BlockT> {
	Header(B::Header, Option<Justifications>),
	Full(B, Option<Justifications>),
}

impl<B: BlockT> StoredBlock<B> {
	fn new(
		header: B::Header,
		body: Option<Vec<B::Extrinsic>>,
		just: Option<Justifications>,
	) -> Self {
		match body {
			Some(body) => StoredBlock::Full(B::new(header, body), just),
			None => StoredBlock::Header(header, just),
		}
	}

	fn header(&self) -> &B::Header {
		match *self {
			StoredBlock::Header(ref h, _) => h,
			StoredBlock::Full(ref b, _) => b.header(),
		}
	}

	fn justifications(&self) -> Option<&Justifications> {
		match *self {
			StoredBlock::Header(_, ref j) | StoredBlock::Full(_, ref j) => j.as_ref(),
		}
	}

	fn extrinsics(&self) -> Option<&[B::Extrinsic]> {
		match *self {
			StoredBlock::Header(_, _) => None,
			StoredBlock::Full(ref b, _) => Some(b.extrinsics()),
		}
	}

	fn into_inner(self) -> (B::Header, Option<Vec<B::Extrinsic>>, Option<Justifications>) {
		match self {
			StoredBlock::Header(header, just) => (header, None, just),
			StoredBlock::Full(block, just) => {
				let (header, body) = block.deconstruct();
				(header, Some(body), just)
			}
		}
	}
}

#[derive(Clone)]
struct BlockchainStorage<Block: BlockT> {
	blocks: HashMap<Block::Hash, StoredBlock<Block>>,
	hashes: HashMap<NumberFor<Block>, Block::Hash>,
	best_hash: Block::Hash,
	best_number: NumberFor<Block>,
	finalized_hash: Block::Hash,
	finalized_number: NumberFor<Block>,
	genesis_hash: Block::Hash,
	header_cht_roots: HashMap<NumberFor<Block>, Block::Hash>,
	leaves: LeafSet<Block::Hash, NumberFor<Block>>,
	aux: HashMap<Vec<u8>, Vec<u8>>,
}

/// In-memory blockchain. Supports concurrent reads.
#[derive(Clone)]
pub struct Blockchain<Block: BlockT> {
	rpc_client: Arc<RPC>,
	storage: Arc<ReadWriteLock<BlockchainStorage<Block>>>,
}

impl<Block: BlockT + DeserializeOwned> Blockchain<Block> {
	/// Get header hash of given block.
	pub fn id(&self, id: BlockId<Block>) -> Option<Block::Hash> {
		match id {
			BlockId::Hash(h) => Some(h),
			BlockId::Number(n) => self.storage.read().hashes.get(&n).cloned(),
		}
	}

	/// Create new in-memory blockchain storage.
	fn new(rpc_client: Arc<RPC>) -> Blockchain<Block> {
		let storage = Arc::new(ReadWriteLock::new(BlockchainStorage {
			blocks: HashMap::new(),
			hashes: HashMap::new(),
			best_hash: Default::default(),
			best_number: Zero::zero(),
			finalized_hash: Default::default(),
			finalized_number: Zero::zero(),
			genesis_hash: Default::default(),
			header_cht_roots: HashMap::new(),
			leaves: LeafSet::new(),
			aux: HashMap::new(),
		}));
		Blockchain {
			rpc_client,
			storage,
		}
	}

	/// Insert a block header and associated data.
	pub fn insert(
		&self,
		hash: Block::Hash,
		header: <Block as BlockT>::Header,
		justifications: Option<Justifications>,
		body: Option<Vec<<Block as BlockT>::Extrinsic>>,
		new_state: NewBlockState,
	) -> sp_blockchain::Result<()> {
		let number = *header.number();
		if new_state.is_best() {
			self.apply_head(&header)?;
		}

		{
			let mut storage = self.storage.write();
			storage.leaves.import(hash, number, *header.parent_hash());
			storage
				.blocks
				.insert(hash, StoredBlock::new(header, body, justifications));

			if let NewBlockState::Final = new_state {
				storage.finalized_hash = hash;
				storage.finalized_number = number;
			}

			if number == Zero::zero() {
				storage.genesis_hash = hash;
			}
		}

		Ok(())
	}

	/// Get total number of blocks.
	pub fn blocks_count(&self) -> usize {
		let count = self.storage.read().blocks.len();

		log::debug!(
			target: super::LAZY_LOADING_LOG_TARGET,
			"Total number of blocks: {:?}",
			count
		);

		count
	}

	/// Compare this blockchain with another in-mem blockchain
	pub fn equals_to(&self, other: &Self) -> bool {
		// Check ptr equality first to avoid double read locks.
		if ptr::eq(self, other) {
			return true;
		}
		self.canon_equals_to(other) && self.storage.read().blocks == other.storage.read().blocks
	}

	/// Compare canonical chain to other canonical chain.
	pub fn canon_equals_to(&self, other: &Self) -> bool {
		// Check ptr equality first to avoid double read locks.
		if ptr::eq(self, other) {
			return true;
		}
		let this = self.storage.read();
		let other = other.storage.read();
		this.hashes == other.hashes
			&& this.best_hash == other.best_hash
			&& this.best_number == other.best_number
			&& this.genesis_hash == other.genesis_hash
	}

	/// Insert header CHT root.
	pub fn insert_cht_root(&self, block: NumberFor<Block>, cht_root: Block::Hash) {
		self.storage
			.write()
			.header_cht_roots
			.insert(block, cht_root);
	}

	/// Set an existing block as head.
	pub fn set_head(&self, hash: Block::Hash) -> sp_blockchain::Result<()> {
		let header = self
			.header(hash)?
			.ok_or_else(|| sp_blockchain::Error::UnknownBlock(format!("{}", hash)))?;

		self.apply_head(&header)
	}

	fn apply_head(&self, header: &<Block as BlockT>::Header) -> sp_blockchain::Result<()> {
		let mut storage = self.storage.write();

		let hash = header.hash();
		let number = header.number();

		storage.best_hash = hash;
		storage.best_number = *number;
		storage.hashes.insert(*number, hash);

		Ok(())
	}

	fn finalize_header(
		&self,
		block: Block::Hash,
		justification: Option<Justification>,
	) -> sp_blockchain::Result<()> {
		let mut storage = self.storage.write();
		storage.finalized_hash = block;

		if justification.is_some() {
			let block = storage
				.blocks
				.get_mut(&block)
				.expect("hash was fetched from a block in the db; qed");

			let block_justifications = match block {
				StoredBlock::Header(_, ref mut j) | StoredBlock::Full(_, ref mut j) => j,
			};

			*block_justifications = justification.map(Justifications::from);
		}

		Ok(())
	}

	fn append_justification(
		&self,
		hash: Block::Hash,
		justification: Justification,
	) -> sp_blockchain::Result<()> {
		let mut storage = self.storage.write();

		let block = storage
			.blocks
			.get_mut(&hash)
			.expect("hash was fetched from a block in the db; qed");

		let block_justifications = match block {
			StoredBlock::Header(_, ref mut j) | StoredBlock::Full(_, ref mut j) => j,
		};

		if let Some(stored_justifications) = block_justifications {
			if !stored_justifications.append(justification) {
				return Err(sp_blockchain::Error::BadJustification(
					"Duplicate consensus engine ID".into(),
				));
			}
		} else {
			*block_justifications = Some(Justifications::from(justification));
		};

		Ok(())
	}

	fn write_aux(&self, ops: Vec<(Vec<u8>, Option<Vec<u8>>)>) {
		let mut storage = self.storage.write();
		for (k, v) in ops {
			match v {
				Some(v) => storage.aux.insert(k, v),
				None => storage.aux.remove(&k),
			};
		}
	}
}

impl<Block: BlockT + DeserializeOwned> HeaderBackend<Block> for Blockchain<Block> {
	fn header(
		&self,
		hash: Block::Hash,
	) -> sp_blockchain::Result<Option<<Block as BlockT>::Header>> {
		// First, try to get the header from local storage
		if let Some(header) = self
			.storage
			.read()
			.blocks
			.get(&hash)
			.map(|b| b.header().clone())
		{
			return Ok(Some(header));
		}

		// If not found in local storage, fetch from RPC client
		let header = self
			.rpc_client
			.block::<Block, _>(Some(hash))
			.ok()
			.flatten()
			.map(|full_block| {
				// Cache block header
				let block = full_block.block.clone();
				self.storage.write().blocks.insert(
					hash,
					StoredBlock::Full(block.clone(), full_block.justifications),
				);

				block.header().clone()
			});

		if header.is_none() {
			log::warn!(
				target: super::LAZY_LOADING_LOG_TARGET,
				"Expected block {:x?} to exist.",
				&hash
			);
		}

		Ok(header)
	}

	fn info(&self) -> blockchain::Info<Block> {
		let storage = self.storage.read();
		blockchain::Info {
			best_hash: storage.best_hash,
			best_number: storage.best_number,
			genesis_hash: storage.genesis_hash,
			finalized_hash: storage.finalized_hash,
			finalized_number: storage.finalized_number,
			finalized_state: Some((storage.finalized_hash, storage.finalized_number)),
			number_leaves: storage.leaves.count(),
			block_gap: None,
		}
	}

	fn status(&self, hash: Block::Hash) -> sp_blockchain::Result<BlockStatus> {
		match self.storage.read().blocks.contains_key(&hash) {
			true => Ok(BlockStatus::InChain),
			false => Ok(BlockStatus::Unknown),
		}
	}

	fn number(&self, hash: Block::Hash) -> sp_blockchain::Result<Option<NumberFor<Block>>> {
		let number = match self.storage.read().blocks.get(&hash) {
			Some(block) => *block.header().number(),
			_ => match self.rpc_client.block::<Block, _>(Some(hash)) {
				Ok(Some(block)) => *block.block.header().number(),
				err => {
					return Err(sp_blockchain::Error::UnknownBlock(
						format!("Failed to fetch block number from RPC: {:?}", err).into(),
					));
				}
			},
		};

		Ok(Some(number))
	}

	fn hash(
		&self,
		number: <<Block as BlockT>::Header as HeaderT>::Number,
	) -> sp_blockchain::Result<Option<Block::Hash>> {
		Ok(self.id(BlockId::Number(number)))
	}
}

impl<Block: BlockT + DeserializeOwned> HeaderMetadata<Block> for Blockchain<Block> {
	type Error = sp_blockchain::Error;

	fn header_metadata(
		&self,
		hash: Block::Hash,
	) -> Result<CachedHeaderMetadata<Block>, Self::Error> {
		self.header(hash)?
			.map(|header| CachedHeaderMetadata::from(&header))
			.ok_or_else(|| {
				sp_blockchain::Error::UnknownBlock(format!("header not found: {}", hash))
			})
	}

	fn insert_header_metadata(&self, _hash: Block::Hash, _metadata: CachedHeaderMetadata<Block>) {
		// No need to implement.
		unimplemented!("insert_header_metadata")
	}
	fn remove_header_metadata(&self, _hash: Block::Hash) {
		// No need to implement.
		unimplemented!("remove_header_metadata")
	}
}

impl<Block: BlockT + DeserializeOwned> blockchain::Backend<Block> for Blockchain<Block> {
	fn body(
		&self,
		hash: Block::Hash,
	) -> sp_blockchain::Result<Option<Vec<<Block as BlockT>::Extrinsic>>> {
		// First, try to get the header from local storage
		if let Some(extrinsics) = self
			.storage
			.read()
			.blocks
			.get(&hash)
			.and_then(|b| b.extrinsics().map(|x| x.to_vec()))
		{
			return Ok(Some(extrinsics));
		}
		let extrinsics = self
			.rpc_client
			.block::<Block, Block::Hash>(Some(hash))
			.ok()
			.flatten()
			.map(|b| b.block.extrinsics().to_vec());

		Ok(extrinsics)
	}

	fn justifications(&self, hash: Block::Hash) -> sp_blockchain::Result<Option<Justifications>> {
		Ok(self
			.storage
			.read()
			.blocks
			.get(&hash)
			.and_then(|b| b.justifications().cloned()))
	}

	fn last_finalized(&self) -> sp_blockchain::Result<Block::Hash> {
		let last_finalized = self.storage.read().finalized_hash;

		Ok(last_finalized)
	}

	fn leaves(&self) -> sp_blockchain::Result<Vec<Block::Hash>> {
		Ok(self.storage.read().leaves.hashes())
	}

	fn displaced_leaves_after_finalizing(
		&self,
		block_number: NumberFor<Block>,
	) -> sp_blockchain::Result<Vec<Block::Hash>> {
		Ok(self
			.storage
			.read()
			.leaves
			.displaced_by_finalize_height(block_number)
			.leaves()
			.cloned()
			.collect::<Vec<_>>())
	}

	fn children(&self, _parent_hash: Block::Hash) -> sp_blockchain::Result<Vec<Block::Hash>> {
		unimplemented!("Not supported by the `lazy-loading` backend.")
	}

	fn indexed_transaction(&self, _hash: Block::Hash) -> sp_blockchain::Result<Option<Vec<u8>>> {
		unimplemented!("Not supported by the `lazy-loading` backend.")
	}

	fn block_indexed_body(
		&self,
		_hash: Block::Hash,
	) -> sp_blockchain::Result<Option<Vec<Vec<u8>>>> {
		unimplemented!("Not supported by the `lazy-loading` backend.")
	}
}

impl<Block: BlockT + DeserializeOwned> backend::AuxStore for Blockchain<Block> {
	fn insert_aux<
		'a,
		'b: 'a,
		'c: 'a,
		I: IntoIterator<Item = &'a (&'c [u8], &'c [u8])>,
		D: IntoIterator<Item = &'a &'b [u8]>,
	>(
		&self,
		insert: I,
		delete: D,
	) -> sp_blockchain::Result<()> {
		let mut storage = self.storage.write();
		for (k, v) in insert {
			storage.aux.insert(k.to_vec(), v.to_vec());
		}
		for k in delete {
			storage.aux.remove(*k);
		}
		Ok(())
	}

	fn get_aux(&self, key: &[u8]) -> sp_blockchain::Result<Option<Vec<u8>>> {
		Ok(self.storage.read().aux.get(key).cloned())
	}
}

pub struct BlockImportOperation<Block: BlockT> {
	pending_block: Option<PendingBlock<Block>>,
	old_state: ForkedLazyBackend<Block>,
	new_state: Option<BackendTransaction<HashingFor<Block>>>,
	aux: Vec<(Vec<u8>, Option<Vec<u8>>)>,
	storage_updates: StorageCollection,
	finalized_blocks: Vec<(Block::Hash, Option<Justification>)>,
	set_head: Option<Block::Hash>,
	pub(crate) before_fork: bool,
}

impl<Block: BlockT + DeserializeOwned> BlockImportOperation<Block> {
	fn apply_storage(
		&mut self,
		storage: Storage,
		commit: bool,
		state_version: StateVersion,
	) -> sp_blockchain::Result<Block::Hash> {
		use sp_state_machine::Backend;
		check_genesis_storage(&storage)?;

		let child_delta = storage.children_default.values().map(|child_content| {
			(
				&child_content.child_info,
				child_content
					.data
					.iter()
					.map(|(k, v)| (k.as_ref(), Some(v.as_ref()))),
			)
		});

		let (root, transaction) = self.old_state.full_storage_root(
			storage
				.top
				.iter()
				.map(|(k, v)| (k.as_ref(), Some(v.as_ref()))),
			child_delta,
			state_version,
		);

		if commit {
			self.new_state = Some(transaction);
			self.storage_updates = storage
				.top
				.iter()
				.map(|(k, v)| (k.clone(), Some(v.clone())))
				.collect();
		}
		Ok(root)
	}
}

impl<Block: BlockT + DeserializeOwned> backend::BlockImportOperation<Block>
	for BlockImportOperation<Block>
{
	type State = ForkedLazyBackend<Block>;

	fn state(&self) -> sp_blockchain::Result<Option<&Self::State>> {
		Ok(Some(&self.old_state))
	}

	fn set_block_data(
		&mut self,
		header: <Block as BlockT>::Header,
		body: Option<Vec<<Block as BlockT>::Extrinsic>>,
		_indexed_body: Option<Vec<Vec<u8>>>,
		justifications: Option<Justifications>,
		state: NewBlockState,
	) -> sp_blockchain::Result<()> {
		assert!(
			self.pending_block.is_none(),
			"Only one block per operation is allowed"
		);
		self.pending_block = Some(PendingBlock {
			block: StoredBlock::new(header, body, justifications),
			state,
		});
		Ok(())
	}

	fn update_db_storage(
		&mut self,
		update: BackendTransaction<HashingFor<Block>>,
	) -> sp_blockchain::Result<()> {
		self.new_state = Some(update);
		Ok(())
	}

	fn set_genesis_state(
		&mut self,
		storage: Storage,
		commit: bool,
		state_version: StateVersion,
	) -> sp_blockchain::Result<Block::Hash> {
		self.apply_storage(storage, commit, state_version)
	}

	fn reset_storage(
		&mut self,
		storage: Storage,
		state_version: StateVersion,
	) -> sp_blockchain::Result<Block::Hash> {
		self.apply_storage(storage, true, state_version)
	}

	fn insert_aux<I>(&mut self, ops: I) -> sp_blockchain::Result<()>
	where
		I: IntoIterator<Item = (Vec<u8>, Option<Vec<u8>>)>,
	{
		self.aux.append(&mut ops.into_iter().collect());
		Ok(())
	}

	fn update_storage(
		&mut self,
		update: StorageCollection,
		_child_update: ChildStorageCollection,
	) -> sp_blockchain::Result<()> {
		self.storage_updates = update.clone();
		Ok(())
	}

	fn mark_finalized(
		&mut self,
		hash: Block::Hash,
		justification: Option<Justification>,
	) -> sp_blockchain::Result<()> {
		self.finalized_blocks.push((hash, justification));
		Ok(())
	}

	fn mark_head(&mut self, hash: Block::Hash) -> sp_blockchain::Result<()> {
		assert!(
			self.pending_block.is_none(),
			"Only one set block per operation is allowed"
		);
		self.set_head = Some(hash);
		Ok(())
	}

	fn update_transaction_index(
		&mut self,
		_index: Vec<IndexOperation>,
	) -> sp_blockchain::Result<()> {
		Ok(())
	}
}

/// DB-backed patricia trie state, transaction type is an overlay of changes to commit.
pub type DbState<B> = TrieBackend<Arc<dyn sp_state_machine::Storage<HashingFor<B>>>, HashingFor<B>>;

/// A struct containing arguments for iterating over the storage.
#[derive(Default)]
pub struct RawIterArgs {
	/// The prefix of the keys over which to iterate.
	pub prefix: Option<Vec<u8>>,

	/// The prefix from which to start the iteration from.
	///
	/// This is inclusive and the iteration will include the key which is specified here.
	pub start_at: Option<Vec<u8>>,

	/// If this is `true` then the iteration will *not* include
	/// the key specified in `start_at`, if there is such a key.
	pub start_at_exclusive: bool,
}

/// A raw iterator over the `BenchmarkingState`.
pub struct RawIter<Block: BlockT> {
	pub(crate) args: RawIterArgs,
	complete: bool,
	_phantom: PhantomData<Block>,
}

impl<Block: BlockT + DeserializeOwned> sp_state_machine::StorageIterator<HashingFor<Block>>
	for RawIter<Block>
{
	type Backend = ForkedLazyBackend<Block>;
	type Error = String;

	fn next_key(
		&mut self,
		backend: &Self::Backend,
	) -> Option<Result<sp_state_machine::StorageKey, Self::Error>> {
		use sp_state_machine::Backend;

		let remote_fetch =
			|key: Option<StorageKey>, start_key: Option<StorageKey>, block: Option<Block::Hash>| {
				let result = backend
					.rpc_client
					.storage_keys_paged(key, 5, start_key, block);

				match result {
					Ok(keys) => keys.first().map(|key| key.clone()),
					Err(err) => {
						log::trace!(
							target: super::LAZY_LOADING_LOG_TARGET,
							"Failed to fetch `next key` from RPC: {:?}",
							err
						);

						None
					}
				}
			};

		let prefix = self.args.prefix.clone().map(|k| StorageKey(k));
		let start_key = self.args.start_at.clone().map(|k| StorageKey(k));

		let maybe_next_key = if backend.before_fork {
			remote_fetch(prefix, start_key, backend.block_hash)
		} else {
			let mut iter_args = sp_state_machine::backend::IterArgs::default();
			iter_args.prefix = self.args.prefix.as_ref().map(|b| b.as_slice());
			iter_args.start_at = self.args.start_at.as_ref().map(|b| b.as_slice());
			iter_args.start_at_exclusive = true;
			iter_args.stop_on_incomplete_database = true;

			let readable_db = backend.db.read();
			let next_storage_key = readable_db
				.raw_iter(iter_args)
				.map(|mut iter| iter.next_key(&readable_db))
				.map(|op| op.map(|result| result.ok()).flatten())
				.ok()
				.flatten();

			// IMPORTANT: free storage read lock
			drop(readable_db);

			let removed_key = start_key
				.clone()
				.or(prefix.clone())
				.map(|key| backend.removed_keys.read().contains_key(&key.0))
				.unwrap_or(false);
			if next_storage_key.is_none() && !removed_key {
				let maybe_next_key = remote_fetch(prefix, start_key, Some(backend.fork_block));
				match maybe_next_key {
					Some(key) if !backend.removed_keys.read().contains_key(&key) => Some(key),
					_ => None,
				}
			} else {
				next_storage_key
			}
		};

		log::trace!(
			target: super::LAZY_LOADING_LOG_TARGET,
			"next_key: (prefix: {:?}, start_at: {:?}, next_key: {:?})",
			self.args.prefix.clone().map(|key| hex::encode(key)),
			self.args.start_at.clone().map(|key| hex::encode(key)),
			maybe_next_key.clone().map(|key| hex::encode(key))
		);

		if let Some(next_key) = maybe_next_key {
			if self
				.args
				.prefix
				.clone()
				.map(|filter_key| next_key.starts_with(&filter_key))
				.unwrap_or(false)
			{
				self.args.start_at = Some(next_key.clone());
				Some(Ok(next_key))
			} else {
				self.complete = true;
				None
			}
		} else {
			self.complete = true;
			None
		}
	}

	fn next_pair(
		&mut self,
		backend: &Self::Backend,
	) -> Option<Result<(sp_state_machine::StorageKey, sp_state_machine::StorageValue), Self::Error>>
	{
		use sp_state_machine::Backend;

		let remote_fetch =
			|key: Option<StorageKey>, start_key: Option<StorageKey>, block: Option<Block::Hash>| {
				let result = backend
					.rpc_client
					.storage_keys_paged(key, 5, start_key, block);

				match result {
					Ok(keys) => keys.first().map(|key| key.clone()),
					Err(err) => {
						log::trace!(
							target: super::LAZY_LOADING_LOG_TARGET,
							"Failed to fetch `next key` from RPC: {:?}",
							err
						);

						None
					}
				}
			};

		let prefix = self.args.prefix.clone().map(|k| StorageKey(k));
		let start_key = self.args.start_at.clone().map(|k| StorageKey(k));

		let maybe_next_key = if backend.before_fork {
			remote_fetch(prefix, start_key, backend.block_hash)
		} else {
			let mut iter_args = sp_state_machine::backend::IterArgs::default();
			iter_args.prefix = self.args.prefix.as_ref().map(|b| b.as_slice());
			iter_args.start_at = self.args.start_at.as_ref().map(|b| b.as_slice());
			iter_args.start_at_exclusive = true;
			iter_args.stop_on_incomplete_database = true;

			let readable_db = backend.db.read();
			let next_storage_key = readable_db
				.raw_iter(iter_args)
				.map(|mut iter| iter.next_key(&readable_db))
				.map(|op| op.map(|result| result.ok()).flatten())
				.ok()
				.flatten();

			// IMPORTANT: free storage read lock
			drop(readable_db);

			let removed_key = start_key
				.clone()
				.or(prefix.clone())
				.map(|key| backend.removed_keys.read().contains_key(&key.0))
				.unwrap_or(false);
			if next_storage_key.is_none() && !removed_key {
				let maybe_next_key = remote_fetch(prefix, start_key, Some(backend.fork_block));
				match maybe_next_key {
					Some(key) if !backend.removed_keys.read().contains_key(&key) => Some(key),
					_ => None,
				}
			} else {
				next_storage_key
			}
		};

		log::trace!(
			target: super::LAZY_LOADING_LOG_TARGET,
			"next_pair: (prefix: {:?}, start_at: {:?}, next_key: {:?})",
			self.args.prefix.clone().map(|key| hex::encode(key)),
			self.args.start_at.clone().map(|key| hex::encode(key)),
			maybe_next_key.clone().map(|key| hex::encode(key))
		);

		let maybe_value = maybe_next_key
			.clone()
			.map(|key| (*backend).storage(key.as_slice()).ok())
			.flatten()
			.flatten();

		if let Some(next_key) = maybe_next_key {
			if self
				.args
				.prefix
				.clone()
				.map(|filter_key| next_key.starts_with(&filter_key))
				.unwrap_or(false)
			{
				self.args.start_at = Some(next_key.clone());

				match maybe_value {
					Some(value) => Some(Ok((next_key, value))),
					_ => None,
				}
			} else {
				self.complete = true;
				None
			}
		} else {
			self.complete = true;
			None
		}
	}

	fn was_complete(&self) -> bool {
		self.complete
	}
}

#[derive(Debug, Clone)]
pub struct ForkedLazyBackend<Block: BlockT> {
	rpc_client: Arc<RPC>,
	block_hash: Option<Block::Hash>,
	fork_block: Block::Hash,
	pub(crate) db: Arc<ReadWriteLock<sp_state_machine::InMemoryBackend<HashingFor<Block>>>>,
	pub(crate) removed_keys: Arc<ReadWriteLock<HashMap<Vec<u8>, ()>>>,
	before_fork: bool,
}

impl<Block: BlockT> ForkedLazyBackend<Block> {
	fn update_storage(&self, key: &[u8], value: &Option<Vec<u8>>) {
		if let Some(ref val) = value {
			let mut entries: HashMap<Option<ChildInfo>, StorageCollection> = Default::default();
			entries.insert(None, vec![(key.to_vec(), Some(val.clone()))]);

			self.db.write().insert(entries, StateVersion::V0);
		}
	}
}

impl<Block: BlockT + DeserializeOwned> sp_state_machine::Backend<HashingFor<Block>>
	for ForkedLazyBackend<Block>
{
	type Error = <DbState<Block> as sp_state_machine::Backend<HashingFor<Block>>>::Error;
	type TrieBackendStorage = PrefixedMemoryDB<HashingFor<Block>>;
	type RawIter = RawIter<Block>;

	fn storage(&self, key: &[u8]) -> Result<Option<sp_state_machine::StorageValue>, Self::Error> {
		let remote_fetch = |block: Option<Block::Hash>| {
			let result = self.rpc_client.storage(StorageKey(key.to_vec()), block);

			match result {
				Ok(data) => data.map(|v| v.0),
				Err(err) => {
					log::debug!(
						target: super::LAZY_LOADING_LOG_TARGET,
						"Failed to fetch storage from live network: {:?}",
						err
					);
					None
				}
			}
		};

		if self.before_fork {
			return Ok(remote_fetch(self.block_hash));
		}

		let readable_db = self.db.read();
		let maybe_storage = readable_db.storage(key);
		let value = match maybe_storage {
			Ok(Some(data)) => Some(data),
			_ if !self.removed_keys.read().contains_key(key) => {
				let result = remote_fetch(Some(self.fork_block));

				// Cache state
				drop(readable_db);
				self.update_storage(key, &result);

				result
			}
			_ => None,
		};

		Ok(value)
	}

	fn storage_hash(
		&self,
		key: &[u8],
	) -> Result<Option<<HashingFor<Block> as sp_core::Hasher>::Out>, Self::Error> {
		let remote_fetch = |block: Option<Block::Hash>| {
			let result = self
				.rpc_client
				.storage_hash(StorageKey(key.to_vec()), block);

			match result {
				Ok(hash) => Ok(hash),
				Err(err) => Err(format!("Failed to fetch storage hash from RPC: {:?}", err).into()),
			}
		};

		if self.before_fork {
			return remote_fetch(self.block_hash);
		}

		let storage_hash = self.db.read().storage_hash(key);
		match storage_hash {
			Ok(Some(hash)) => Ok(Some(hash)),
			_ if !self.removed_keys.read().contains_key(key) => remote_fetch(Some(self.fork_block)),
			_ => Ok(None),
		}
	}

	fn closest_merkle_value(
		&self,
		_key: &[u8],
	) -> Result<
		Option<sp_trie::MerkleValue<<HashingFor<Block> as sp_core::Hasher>::Out>>,
		Self::Error,
	> {
		unimplemented!("closest_merkle_value: unsupported feature for lazy loading")
	}

	fn child_closest_merkle_value(
		&self,
		_child_info: &sp_storage::ChildInfo,
		_key: &[u8],
	) -> Result<
		Option<sp_trie::MerkleValue<<HashingFor<Block> as sp_core::Hasher>::Out>>,
		Self::Error,
	> {
		unimplemented!("child_closest_merkle_value: unsupported feature for lazy loading")
	}

	fn child_storage(
		&self,
		_child_info: &sp_storage::ChildInfo,
		_key: &[u8],
	) -> Result<Option<sp_state_machine::StorageValue>, Self::Error> {
		unimplemented!("child_storage: unsupported feature for lazy loading");
	}

	fn child_storage_hash(
		&self,
		_child_info: &sp_storage::ChildInfo,
		_key: &[u8],
	) -> Result<Option<<HashingFor<Block> as sp_core::Hasher>::Out>, Self::Error> {
		unimplemented!("child_storage_hash: unsupported feature for lazy loading");
	}

	fn next_storage_key(
		&self,
		key: &[u8],
	) -> Result<Option<sp_state_machine::StorageKey>, Self::Error> {
		let remote_fetch = |block: Option<Block::Hash>| {
			let start_key = Some(StorageKey(key.to_vec()));
			let result = self
				.rpc_client
				.storage_keys_paged(start_key.clone(), 2, None, block);

			match result {
				Ok(keys) => keys.last().cloned(),
				Err(err) => {
					log::trace!(
						target: super::LAZY_LOADING_LOG_TARGET,
						"Failed to fetch `next storage key` from RPC: {:?}",
						err
					);

					None
				}
			}
		};

		let maybe_next_key = if self.before_fork {
			remote_fetch(self.block_hash)
		} else {
			let next_storage_key = self.db.read().next_storage_key(key);
			match next_storage_key {
				Ok(Some(key)) => Some(key),
				_ if !self.removed_keys.read().contains_key(key) => {
					remote_fetch(Some(self.fork_block))
				}
				_ => None,
			}
		};

		log::trace!(
			target: super::LAZY_LOADING_LOG_TARGET,
			"next_storage_key: (key: {:?}, next_key: {:?})",
			hex::encode(key),
			maybe_next_key.clone().map(|key| hex::encode(key))
		);

		Ok(maybe_next_key)
	}

	fn next_child_storage_key(
		&self,
		_child_info: &sp_storage::ChildInfo,
		_key: &[u8],
	) -> Result<Option<sp_state_machine::StorageKey>, Self::Error> {
		unimplemented!("next_child_storage_key: unsupported feature for lazy loading");
	}

	fn storage_root<'a>(
		&self,
		delta: impl Iterator<Item = (&'a [u8], Option<&'a [u8]>)>,
		state_version: StateVersion,
	) -> (
		<HashingFor<Block> as sp_core::Hasher>::Out,
		BackendTransaction<HashingFor<Block>>,
	)
	where
		<HashingFor<Block> as sp_core::Hasher>::Out: Ord,
	{
		self.db.read().storage_root(delta, state_version)
	}

	fn child_storage_root<'a>(
		&self,
		_child_info: &sp_storage::ChildInfo,
		_delta: impl Iterator<Item = (&'a [u8], Option<&'a [u8]>)>,
		_state_version: StateVersion,
	) -> (
		<HashingFor<Block> as sp_core::Hasher>::Out,
		bool,
		BackendTransaction<HashingFor<Block>>,
	)
	where
		<HashingFor<Block> as sp_core::Hasher>::Out: Ord,
	{
		unimplemented!("child_storage_root: unsupported in lazy loading")
	}

	fn raw_iter(&self, args: sp_state_machine::IterArgs) -> Result<Self::RawIter, Self::Error> {
		let mut clone: RawIterArgs = Default::default();
		clone.start_at_exclusive = args.start_at_exclusive.clone();
		clone.prefix = args.prefix.map(|v| v.to_vec());
		clone.start_at = args.start_at.map(|v| v.to_vec());

		Ok(RawIter::<Block> {
			args: clone,
			complete: false,
			_phantom: Default::default(),
		})
	}

	fn register_overlay_stats(&self, stats: &sp_state_machine::StateMachineStats) {
		self.db.read().register_overlay_stats(stats)
	}

	fn usage_info(&self) -> sp_state_machine::UsageInfo {
		self.db.read().usage_info()
	}
}

impl<B: BlockT> sp_state_machine::backend::AsTrieBackend<HashingFor<B>> for ForkedLazyBackend<B> {
	type TrieBackendStorage = PrefixedMemoryDB<HashingFor<B>>;

	fn as_trie_backend(
		&self,
	) -> &sp_state_machine::TrieBackend<Self::TrieBackendStorage, HashingFor<B>> {
		unimplemented!("`as_trie_backend` is not supported in lazy loading mode.")
	}
}

/// Lazy loading (In-memory) backend. Keeps all states and blocks in memory.
pub struct Backend<Block: BlockT> {
	pub(crate) rpc_client: Arc<RPC>,
	states: ReadWriteLock<HashMap<Block::Hash, ForkedLazyBackend<Block>>>,
	pub(crate) blockchain: Blockchain<Block>,
	import_lock: parking_lot::RwLock<()>,
	pinned_blocks: ReadWriteLock<HashMap<Block::Hash, i64>>,
	pub(crate) fork_checkpoint: Block::Header,
}

impl<Block: BlockT + DeserializeOwned> Backend<Block> {
	fn new(rpc_client: Arc<RPC>, fork_checkpoint: Block::Header) -> Self {
		Backend {
			rpc_client: rpc_client.clone(),
			states: Default::default(),
			blockchain: Blockchain::new(rpc_client),
			import_lock: Default::default(),
			pinned_blocks: Default::default(),
			fork_checkpoint,
		}
	}
}

impl<Block: BlockT + DeserializeOwned> backend::AuxStore for Backend<Block> {
	fn insert_aux<
		'a,
		'b: 'a,
		'c: 'a,
		I: IntoIterator<Item = &'a (&'c [u8], &'c [u8])>,
		D: IntoIterator<Item = &'a &'b [u8]>,
	>(
		&self,
		_insert: I,
		_delete: D,
	) -> sp_blockchain::Result<()> {
		unimplemented!("`insert_aux` is not supported in lazy loading mode.")
	}

	fn get_aux(&self, _key: &[u8]) -> sp_blockchain::Result<Option<Vec<u8>>> {
		unimplemented!("`get_aux` is not supported in lazy loading mode.")
	}
}

impl<Block: BlockT + DeserializeOwned> backend::Backend<Block> for Backend<Block> {
	type BlockImportOperation = BlockImportOperation<Block>;
	type Blockchain = Blockchain<Block>;
	type State = ForkedLazyBackend<Block>;
	type OffchainStorage = InMemOffchainStorage;

	fn begin_operation(&self) -> sp_blockchain::Result<Self::BlockImportOperation> {
		let old_state = self.state_at(Default::default())?;
		Ok(BlockImportOperation {
			pending_block: None,
			old_state,
			new_state: None,
			aux: Default::default(),
			storage_updates: Default::default(),
			finalized_blocks: Default::default(),
			set_head: None,
			before_fork: false,
		})
	}

	fn begin_state_operation(
		&self,
		operation: &mut Self::BlockImportOperation,
		block: Block::Hash,
	) -> sp_blockchain::Result<()> {
		operation.old_state = self.state_at(block)?;
		Ok(())
	}

	fn commit_operation(&self, operation: Self::BlockImportOperation) -> sp_blockchain::Result<()> {
		if !operation.finalized_blocks.is_empty() {
			for (block, justification) in operation.finalized_blocks {
				self.blockchain.finalize_header(block, justification)?;
			}
		}

		if let Some(pending_block) = operation.pending_block {
			let old_state = &operation.old_state;
			let (header, body, justification) = pending_block.block.into_inner();
			let hash = header.hash();

			let new_removed_keys = old_state.removed_keys.clone();
			for (key, value) in operation.storage_updates.clone() {
				if value.is_some() {
					new_removed_keys.write().remove(&key.clone());
				} else {
					new_removed_keys.write().insert(key.clone(), ());
				}
			}

			let new_db = old_state.db.clone();
			new_db.write().insert(
				vec![(None::<ChildInfo>, operation.storage_updates)],
				StateVersion::V0,
			);
			let new_state = ForkedLazyBackend {
				rpc_client: self.rpc_client.clone(),
				block_hash: Some(hash.clone()),
				fork_block: self.fork_checkpoint.hash(),
				db: new_db,
				removed_keys: new_removed_keys,
				before_fork: operation.before_fork,
			};
			self.states.write().insert(hash, new_state);

			self.blockchain
				.insert(hash, header, justification, body, pending_block.state)?;
		}

		if !operation.aux.is_empty() {
			self.blockchain.write_aux(operation.aux);
		}

		if let Some(set_head) = operation.set_head {
			self.blockchain.set_head(set_head)?;
		}

		Ok(())
	}

	fn finalize_block(
		&self,
		hash: Block::Hash,
		justification: Option<Justification>,
	) -> sp_blockchain::Result<()> {
		self.blockchain.finalize_header(hash, justification)
	}

	fn append_justification(
		&self,
		hash: Block::Hash,
		justification: Justification,
	) -> sp_blockchain::Result<()> {
		self.blockchain.append_justification(hash, justification)
	}

	fn blockchain(&self) -> &Self::Blockchain {
		&self.blockchain
	}

	fn usage_info(&self) -> Option<UsageInfo> {
		None
	}

	fn offchain_storage(&self) -> Option<Self::OffchainStorage> {
		None
	}

	fn state_at(&self, hash: Block::Hash) -> sp_blockchain::Result<Self::State> {
		if hash == Default::default() {
			return Ok(ForkedLazyBackend::<Block> {
				rpc_client: self.rpc_client.clone(),
				block_hash: Some(hash),
				fork_block: self.fork_checkpoint.hash(),
				db: Default::default(),
				removed_keys: Default::default(),
				before_fork: true,
			});
		}

		let (backend, should_write) = self
			.states
			.read()
			.get(&hash)
			.cloned()
			.map(|state| (state, false))
			.unwrap_or_else(|| {
				let header: Block::Header = self
					.rpc_client
					.header::<Block>(Some(hash))
					.ok()
					.flatten()
					.expect("block header");

				let checkpoint = self.fork_checkpoint.clone();
				let state = if header.number().gt(checkpoint.number()) {
					let parent = self.state_at(*header.parent_hash()).ok();

					ForkedLazyBackend::<Block> {
						rpc_client: self.rpc_client.clone(),
						block_hash: Some(hash),
						fork_block: checkpoint.hash(),
						db: parent.clone().map_or(Default::default(), |p| p.db),
						removed_keys: parent.map_or(Default::default(), |p| p.removed_keys),
						before_fork: false,
					}
				} else {
					ForkedLazyBackend::<Block> {
						rpc_client: self.rpc_client.clone(),
						block_hash: Some(hash),
						fork_block: checkpoint.hash(),
						db: Default::default(),
						removed_keys: Default::default(),
						before_fork: true,
					}
				};

				(state, true)
			});

		if should_write {
			self.states.write().insert(hash, backend.clone());
		}

		Ok(backend)
	}

	fn revert(
		&self,
		_n: NumberFor<Block>,
		_revert_finalized: bool,
	) -> sp_blockchain::Result<(NumberFor<Block>, HashSet<Block::Hash>)> {
		Ok((Zero::zero(), HashSet::new()))
	}

	fn remove_leaf_block(&self, _hash: Block::Hash) -> sp_blockchain::Result<()> {
		Ok(())
	}

	fn get_import_lock(&self) -> &parking_lot::RwLock<()> {
		&self.import_lock
	}

	fn requires_full_sync(&self) -> bool {
		false
	}

	fn pin_block(&self, hash: <Block as BlockT>::Hash) -> blockchain::Result<()> {
		let mut blocks = self.pinned_blocks.write();
		*blocks.entry(hash).or_default() += 1;
		Ok(())
	}

	fn unpin_block(&self, hash: <Block as BlockT>::Hash) {
		let mut blocks = self.pinned_blocks.write();
		blocks
			.entry(hash)
			.and_modify(|counter| *counter -= 1)
			.or_insert(-1);
	}
}

impl<Block: BlockT + DeserializeOwned> backend::LocalBackend<Block> for Backend<Block> {}

/// Check that genesis storage is valid.
pub fn check_genesis_storage(storage: &Storage) -> sp_blockchain::Result<()> {
	if storage
		.top
		.iter()
		.any(|(k, _)| well_known_keys::is_child_storage_key(k))
	{
		return Err(sp_blockchain::Error::InvalidState);
	}

	if storage
		.children_default
		.keys()
		.any(|child_key| !well_known_keys::is_child_storage_key(child_key))
	{
		return Err(sp_blockchain::Error::InvalidState);
	}

	Ok(())
}

#[derive(Debug, Clone)]
pub struct RPC {
	http_client: HttpClient,
	delay_between_requests_ms: u64,
	max_retries_per_request: usize,
	counter: Arc<ReadWriteLock<u64>>,
}

impl RPC {
	pub fn new(
		http_client: HttpClient,
		delay_between_requests_ms: u64,
		max_retries_per_request: usize,
	) -> Self {
		Self {
			http_client,
			delay_between_requests_ms,
			max_retries_per_request,
			counter: Default::default(),
		}
	}
	pub fn system_chain(&self) -> Result<String, jsonrpsee::core::ClientError> {
		let request = &|| {
			substrate_rpc_client::SystemApi::<H256, BlockNumber>::system_chain(&self.http_client)
		};

		self.block_on(request)
	}

	pub fn system_properties(
		&self,
	) -> Result<sc_chain_spec::Properties, jsonrpsee::core::ClientError> {
		let request = &|| {
			substrate_rpc_client::SystemApi::<H256, BlockNumber>::system_properties(
				&self.http_client,
			)
		};

		self.block_on(request)
	}

	pub fn system_name(&self) -> Result<String, jsonrpsee::core::ClientError> {
		let request = &|| {
			substrate_rpc_client::SystemApi::<H256, BlockNumber>::system_name(&self.http_client)
		};

		self.block_on(request)
	}

	pub fn block<Block, Hash: Clone>(
		&self,
		hash: Option<Hash>,
	) -> Result<Option<SignedBlock<Block>>, jsonrpsee::core::ClientError>
	where
		Block: BlockT + DeserializeOwned,
		Hash: 'static + Send + Sync + sp_runtime::Serialize + DeserializeOwned,
	{
		let request = &|| {
			substrate_rpc_client::ChainApi::<
				BlockNumber,
				Hash,
				Block::Header,
				SignedBlock<Block>,
			>::block(&self.http_client, hash.clone())
		};

		self.block_on(request)
	}

	pub fn block_hash<Block: BlockT + DeserializeOwned>(
		&self,
		block_number: Option<BlockNumber>,
	) -> Result<Option<Block::Hash>, jsonrpsee::core::ClientError> {
		let request = &|| {
			substrate_rpc_client::ChainApi::<
				BlockNumber,
				Block::Hash,
				Block::Header,
				SignedBlock<Block>,
			>::block_hash(
				&self.http_client,
				block_number.map(|n| ListOrValue::Value(NumberOrHex::Number(n.into()))),
			)
		};

		self.block_on(request).map(|ok| match ok {
			ListOrValue::List(v) => v.get(0).map_or(None, |some| *some),
			ListOrValue::Value(v) => v,
		})
	}

	pub fn header<Block: BlockT + DeserializeOwned>(
		&self,
		hash: Option<Block::Hash>,
	) -> Result<Option<Block::Header>, jsonrpsee::core::ClientError> {
		let request = &|| {
			substrate_rpc_client::ChainApi::<
				BlockNumber,
				Block::Hash,
				Block::Header,
				SignedBlock<Block>,
			>::header(&self.http_client, hash)
		};

		self.block_on(request)
	}

	pub fn storage_hash<
		Hash: 'static + Clone + Sync + Send + DeserializeOwned + sp_runtime::Serialize,
	>(
		&self,
		key: StorageKey,
		at: Option<Hash>,
	) -> Result<Option<Hash>, jsonrpsee::core::ClientError> {
		let request = &|| {
			substrate_rpc_client::StateApi::<Hash>::storage_hash(
				&self.http_client,
				key.clone(),
				at.clone(),
			)
		};

		self.block_on(request)
	}

	pub fn storage<
		Hash: 'static + Clone + Sync + Send + DeserializeOwned + sp_runtime::Serialize + core::fmt::Debug,
	>(
		&self,
		key: StorageKey,
		at: Option<Hash>,
	) -> Result<Option<StorageData>, jsonrpsee::core::ClientError> {
		let request = &|| {
			substrate_rpc_client::StateApi::<Hash>::storage(
				&self.http_client,
				key.clone(),
				at.clone(),
			)
		};

		self.block_on(request)
	}

	pub fn storage_keys_paged<
		Hash: 'static + Clone + Sync + Send + DeserializeOwned + sp_runtime::Serialize,
	>(
		&self,
		key: Option<StorageKey>,
		count: u32,
		start_key: Option<StorageKey>,
		at: Option<Hash>,
	) -> Result<Vec<sp_state_machine::StorageKey>, jsonrpsee::core::ClientError> {
		let request = &|| {
			substrate_rpc_client::StateApi::<Hash>::storage_keys_paged(
				&self.http_client,
				key.clone(),
				count.clone(),
				start_key.clone(),
				at.clone(),
			)
		};
		let result = self.block_on(request);

		match result {
			Ok(result) => Ok(result.iter().map(|item| item.0.clone()).collect()),
			Err(err) => Err(err),
		}
	}

	pub fn query_storage_at<
		Hash: 'static + Clone + Sync + Send + DeserializeOwned + sp_runtime::Serialize,
	>(
		&self,
		keys: Vec<StorageKey>,
		from_block: Option<Hash>,
	) -> Result<Vec<(StorageKey, Option<StorageData>)>, jsonrpsee::core::ClientError> {
		let request = &|| {
			substrate_rpc_client::StateApi::<Hash>::query_storage_at(
				&self.http_client,
				keys.clone(),
				from_block.clone(),
			)
		};
		let result = self.block_on(request);

		match result {
			Ok(result) => Ok(result
				.iter()
				.flat_map(|item| item.changes.clone())
				.collect()),
			Err(err) => Err(err),
		}
	}

	fn block_on<F, T, E>(&self, f: &dyn Fn() -> F) -> Result<T, E>
	where
		F: Future<Output = Result<T, E>>,
	{
		use tokio::runtime::Handle;

		tokio::task::block_in_place(move || {
			Handle::current().block_on(async move {
				let delay_between_requests = Duration::from_millis(self.delay_between_requests_ms);

				let start = std::time::Instant::now();
				self.counter.write().add_assign(1);
				log::debug!(
					target: super::LAZY_LOADING_LOG_TARGET,
					"Sending request: {}",
					self.counter.read()
				);

				// Explicit request delay, to avoid getting 429 errors
				let _ = tokio::time::sleep(delay_between_requests).await;

				// Retry request in case of failure
				// The maximum number of retries is specified by `self.max_retries_per_request`
				let retry_strategy =
					FixedInterval::new(delay_between_requests).take(self.max_retries_per_request);
				let result = Retry::spawn(retry_strategy, f).await;

				log::debug!(
					target: super::LAZY_LOADING_LOG_TARGET,
					"Completed request (id: {}, successful: {}, elapsed_time: {:?})",
					self.counter.read(),
					result.is_ok(),
					start.elapsed()
				);

				result
			})
		})
	}
}

/// Create an instance of a lazy loading memory backend.
pub fn new_lazy_loading_backend<Block>(
	config: &mut Configuration,
	lazy_loading_config: &LazyLoadingConfig,
) -> Result<Arc<Backend<Block>>, Error>
where
	Block: BlockT + DeserializeOwned,
	Block::Hash: From<H256>,
{
	let uri: String = lazy_loading_config.state_rpc.clone().into();

	let http_client = jsonrpsee::http_client::HttpClientBuilder::default()
		.max_request_size(u32::MAX)
		.max_response_size(u32::MAX)
		.request_timeout(Duration::from_secs(10))
		.build(uri)
		.map_err(|e| {
			sp_blockchain::Error::Backend(
				format!("failed to build http client: {:?}", e).to_string(),
			)
		})?;

	let rpc = RPC::new(http_client, 100, 10);
	let block_hash = lazy_loading_config
		.from_block
		.map(|block| Into::<Block::Hash>::into(block));
	let checkpoint: Block = rpc
		.block::<Block, _>(block_hash)
		.ok()
		.flatten()
		.expect("Fetching fork checkpoint")
		.block;

	let backend = Arc::new(Backend::new(Arc::new(rpc), checkpoint.header().clone()));

	let chain_name = backend
		.rpc_client
		.system_chain()
		.expect("Should fetch chain id");
	let chain_properties = backend
		.rpc_client
		.system_properties()
		.expect("Should fetch chain properties");

	let spec_builder = chain_spec::test_spec::lazy_loading_spec_builder(Default::default())
		.with_name(chain_name.as_str())
		.with_properties(chain_properties);
	config.chain_spec = Box::new(spec_builder.build());

	let base_overrides =
		state_overrides::base_state_overrides(lazy_loading_config.runtime_override.clone());
	let custom_overrides = if let Some(path) = lazy_loading_config.state_overrides_path.clone() {
		state_overrides::read(path)?
	} else {
		Default::default()
	};
	let state_overrides: Vec<(Vec<u8>, Vec<u8>)> = [base_overrides, custom_overrides]
		.concat()
		.iter()
		.map(|entry| match entry {
			StateEntry::Concrete(v) => {
				let key = [
					&twox_128(v.pallet.as_bytes()),
					&twox_128(v.storage.as_bytes()),
					v.key.clone().unwrap_or(Vec::new()).as_slice(),
				]
				.concat();

				(key, v.value.clone())
			}
			StateEntry::Raw(raw) => (raw.key.clone(), raw.value.clone()),
		})
		.collect();

	let _ = helpers::produce_genesis_block(backend.clone());

	// Produce first block after the fork
	let _ = helpers::produce_first_block(backend.clone(), checkpoint, state_overrides)?;

	Ok(backend)
}
