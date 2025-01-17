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

use cumulus_primitives_core::BlockT;
use frame_benchmarking::__private::codec;
use futures::{Stream, StreamExt, TryFutureExt};
use sc_client_api::backend::Backend as ClientBackend;
use sc_client_api::Finalizer;
use sc_consensus::{BlockImport, BlockImportParams, ForkChoiceStrategy, ImportResult, StateAction};
use sc_consensus_manual_seal::{
	finalize_block, rpc, CreatedBlock, EngineCommand, Error, FinalizeBlockParams, ManualSealParams,
	SealBlockParams, MANUAL_SEAL_ENGINE_ID,
};
use sc_transaction_pool_api::TransactionPool;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_consensus::{BlockOrigin, Environment, Proposer, SelectChain};
use sp_inherents::{CreateInherentDataProviders, InherentDataProvider};
use sp_runtime::traits::Header;
use std::marker::PhantomData;
use std::time::Duration;

pub async fn run_manual_seal<B, BI, CB, E, C, TP, SC, CS, CIDP, P>(
	ManualSealParams {
		mut block_import,
		mut env,
		client,
		pool,
		mut commands_stream,
		select_chain,
		consensus_data_provider,
		create_inherent_data_providers,
	}: ManualSealParams<B, BI, E, C, TP, SC, CS, CIDP, P>,
) where
	B: BlockT + 'static,
	BI: BlockImport<B, Error = sp_consensus::Error> + Send + Sync + 'static,
	C: HeaderBackend<B> + Finalizer<B, CB> + ProvideRuntimeApi<B> + 'static,
	CB: ClientBackend<B> + 'static,
	E: Environment<B> + 'static,
	E::Proposer: Proposer<B, Proof = P>,
	CS: Stream<Item = EngineCommand<<B as BlockT>::Hash>> + Unpin + 'static,
	SC: SelectChain<B> + 'static,
	TP: TransactionPool<Block = B>,
	CIDP: CreateInherentDataProviders<B, ()>,
	P: codec::Encode + Send + Sync + 'static,
{
	while let Some(command) = commands_stream.next().await {
		match command {
			EngineCommand::SealNewBlock {
				create_empty,
				finalize,
				parent_hash,
				sender,
			} => {
				seal_block(SealBlockParams {
					sender,
					parent_hash,
					finalize,
					create_empty,
					env: &mut env,
					select_chain: &select_chain,
					block_import: &mut block_import,
					consensus_data_provider: consensus_data_provider.as_deref(),
					pool: pool.clone(),
					client: client.clone(),
					create_inherent_data_providers: &create_inherent_data_providers,
				})
				.await;
			}
			EngineCommand::FinalizeBlock {
				hash,
				sender,
				justification,
			} => {
				let justification = justification.map(|j| (MANUAL_SEAL_ENGINE_ID, j));
				finalize_block(FinalizeBlockParams {
					hash,
					sender,
					justification,
					finalizer: client.clone(),
					_phantom: PhantomData,
				})
				.await
			}
		}
	}
}

/// max duration for creating a proposal in secs
pub const MAX_PROPOSAL_DURATION: u64 = 60;

/// seals a new block with the given params
pub async fn seal_block<B, BI, SC, C, E, TP, CIDP, P>(
	SealBlockParams {
		create_empty,
		finalize,
		pool,
		parent_hash,
		client,
		select_chain,
		block_import,
		env,
		create_inherent_data_providers,
		consensus_data_provider: digest_provider,
		mut sender,
	}: SealBlockParams<'_, B, BI, SC, C, E, TP, CIDP, P>,
) where
	B: BlockT,
	BI: BlockImport<B, Error = sp_consensus::Error> + Send + Sync + 'static,
	C: HeaderBackend<B> + ProvideRuntimeApi<B>,
	E: Environment<B>,
	E::Proposer: Proposer<B, Proof = P>,
	TP: TransactionPool<Block = B>,
	SC: SelectChain<B>,
	CIDP: CreateInherentDataProviders<B, ()>,
	P: codec::Encode + Send + Sync + 'static,
{
	let future = async {
		if pool.status().ready == 0 && !create_empty {
			return Err(Error::EmptyTransactionPool);
		}

		// get the header to build this new block on.
		// use the parent_hash supplied via `EngineCommand`
		// or fetch the best_block.
		let parent = match parent_hash {
			Some(hash) => client
				.header(hash)?
				.ok_or_else(|| Error::BlockNotFound(format!("{}", hash)))?,
			None => select_chain.best_chain().await?,
		};

		let inherent_data_providers = create_inherent_data_providers
			.create_inherent_data_providers(parent.hash(), ())
			.await
			.map_err(|e| Error::Other(e))?;

		let inherent_data = inherent_data_providers.create_inherent_data().await?;

		let proposer = env
			.init(&parent)
			.map_err(|err| Error::StringError(err.to_string()))
			.await?;
		let inherents_len = inherent_data.len();

		let digest = if let Some(digest_provider) = digest_provider {
			digest_provider.create_digest(&parent, &inherent_data)?
		} else {
			Default::default()
		};

		let proposal = proposer
			.propose(
				inherent_data.clone(),
				digest,
				Duration::from_secs(MAX_PROPOSAL_DURATION),
				None,
			)
			.map_err(|err| Error::StringError(err.to_string()))
			.await?;

		if proposal.block.extrinsics().len() == inherents_len && !create_empty {
			return Err(Error::EmptyTransactionPool);
		}

		let (header, body) = proposal.block.deconstruct();
		let proof = proposal.proof;
		let proof_size = proof.encoded_size();
		let mut params = BlockImportParams::new(BlockOrigin::Own, header.clone());
		params.body = Some(body);
		params.finalized = finalize;
		params.fork_choice = Some(ForkChoiceStrategy::LongestChain);
		params.state_action = StateAction::ApplyChanges(sc_consensus::StorageChanges::Changes(
			proposal.storage_changes,
		));

		if let Some(digest_provider) = digest_provider {
			digest_provider.append_block_import(&parent, &mut params, &inherent_data, proof)?;
		}

		// Make sure we return the same post-hash that will be calculated when importing the block
		// This is important in case the digest_provider added any signature, seal, ect.
		let mut post_header = header.clone();
		post_header
			.digest_mut()
			.logs
			.extend(params.post_digests.iter().cloned());

		match block_import.import_block(params).await? {
			ImportResult::Imported(aux) => Ok(CreatedBlock {
				hash: <B as BlockT>::Header::hash(&post_header),
				aux,
				proof_size,
			}),
			other => Err(other.into()),
		}
	};

	rpc::send_result(&mut sender, future.await)
}
