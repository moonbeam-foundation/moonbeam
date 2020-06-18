//! A collection of node-specific RPC methods.

use std::{fmt, sync::Arc};

use moonbeam_runtime::{opaque::Block, AccountId, Balance, Index, UncheckedExtrinsic};
use sc_client_api::backend::{Backend, StateBackend, StorageProvider};
use sc_rpc_api::DenyUnsafe;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};
use sp_consensus::SelectChain;
use sp_runtime::traits::BlakeTwo256;
use sp_transaction_pool::TransactionPool;

/// Light client extra dependencies.
pub struct LightDeps<C, F, P> {
	/// The client instance to use.
	pub client: Arc<C>,
	/// Transaction pool instance.
	pub pool: Arc<P>,
	/// Remote access to the blockchain (async).
	pub remote_blockchain: Arc<dyn sc_client_api::light::RemoteBlockchain<Block>>,
	/// Fetcher instance.
	pub fetcher: Arc<F>,
}

/// Full client dependencies.
pub struct FullDeps<C, P, SC> {
	/// The client instance to use.
	pub client: Arc<C>,
	/// Transaction pool instance.
	pub pool: Arc<P>,
	/// The SelectChain Strategy
	pub select_chain: SC,
	/// Whether to deny unsafe calls
	pub deny_unsafe: DenyUnsafe,
	/// The Node authority flag
	pub is_authority: bool,
}

/// Instantiate all Full RPC extensions.
pub fn create_full<C, P, M, SC, BE>(deps: FullDeps<C, P, SC>) -> jsonrpc_core::IoHandler<M>
where
	BE: Backend<Block> + 'static,
	BE::State: StateBackend<BlakeTwo256>,
	C: ProvideRuntimeApi<Block> + StorageProvider<Block, BE>,
	C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError> + 'static,
	C: Send + Sync + 'static,
	C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Index>,
	C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<
		Block,
		Balance,
		UncheckedExtrinsic,
	>,
	C::Api: frontier_rpc_primitives::EthereumRuntimeApi<Block>,
	<C::Api as sp_api::ApiErrorExt>::Error: fmt::Debug,
	P: TransactionPool<Block = Block> + 'static,
	M: jsonrpc_core::Metadata + Default,
	SC: SelectChain<Block> + 'static,
{
	use frontier_rpc::{EthApi, EthApiServer};
	use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApi};
	use substrate_frame_rpc_system::{FullSystem, SystemApi};

	let mut io = jsonrpc_core::IoHandler::default();
	let FullDeps {
		client,
		pool,
		select_chain,
		deny_unsafe: _,
		is_authority,
	} = deps;

	io.extend_with(SystemApi::to_delegate(FullSystem::new(
		client.clone(),
		pool.clone(),
	)));
	io.extend_with(TransactionPaymentApi::to_delegate(TransactionPayment::new(
		client.clone(),
	)));
	io.extend_with(EthApiServer::to_delegate(EthApi::new(
		client.clone(),
		select_chain,
		pool.clone(),
		moonbeam_runtime::TransactionConverter,
		is_authority,
	)));

	io
}

/// Instantiate all Light RPC extensions.
pub fn create_light<C, P, M, F>(deps: LightDeps<C, F, P>) -> jsonrpc_core::IoHandler<M>
where
	C: sp_blockchain::HeaderBackend<Block>,
	C: Send + Sync + 'static,
	F: sc_client_api::light::Fetcher<Block> + 'static,
	P: TransactionPool + 'static,
	M: jsonrpc_core::Metadata + Default,
{
	use substrate_frame_rpc_system::{LightSystem, SystemApi};

	let LightDeps {
		client,
		pool,
		remote_blockchain,
		fetcher,
	} = deps;
	let mut io = jsonrpc_core::IoHandler::default();
	io.extend_with(SystemApi::<AccountId, Index>::to_delegate(
		LightSystem::new(client, remote_blockchain, fetcher, pool),
	));

	io
}
