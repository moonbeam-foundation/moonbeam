use ethereum::{Transaction as EthereumTransaction, TransactionAction, TransactionMessage};
use ethereum_types::{H160, H256, U256};
use jsonrpc_core::Result as RpcResult;
use jsonrpc_core::{Error, ErrorCode};
pub use moonbeam_rpc_core_txpool::{
	Summary, Transaction, TransactionMap, TxPool as TxPoolT, TxPoolResult, TxPoolServer,
};
use sha3::{Digest, Keccak256};
use sp_api::{BlockId, ProvideRuntimeApi};
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};
use sp_runtime::traits::Block as BlockT;
use sp_transaction_pool::{InPoolTransaction, TransactionPool};
use std::collections::HashMap;
use std::{marker::PhantomData, sync::Arc};

use moonbeam_rpc_primitives_txpool::TxPoolRuntimeApi;

use fc_rpc_core::types::Bytes;

// Note: both `internal_err` and `public_key` could be imported from Frontier.
// However there are dependency collisions due to Frontier using a different Substrate branch.
// This may change in the future, and in that case we could just import them.
pub fn internal_err<T: ToString>(message: T) -> Error {
	Error {
		code: ErrorCode::InternalError,
		message: message.to_string(),
		data: None,
	}
}

pub fn public_key(
	transaction: &EthereumTransaction,
	hash: H256,
) -> Result<[u8; 64], sp_io::EcdsaVerifyError> {
	let mut sig = [0u8; 65];
	let mut msg = [0u8; 32];
	sig[0..32].copy_from_slice(&transaction.signature.r()[..]);
	sig[32..64].copy_from_slice(&transaction.signature.s()[..]);
	sig[64] = transaction.signature.standard_v();
	msg.copy_from_slice(&hash[..]);

	sp_io::crypto::secp256k1_ecdsa_recover(&sig, &msg)
}

pub struct TxPool<B: BlockT, C, P> {
	client: Arc<C>,
	pool: Arc<P>,
	_marker: PhantomData<B>,
}

impl<B: BlockT, C, P> TxPool<B, C, P> {
	pub fn new(client: Arc<C>, pool: Arc<P>) -> Self {
		Self {
			client,
			pool,
			_marker: PhantomData,
		}
	}
}

impl<B, C, P> TxPoolT for TxPool<B, C, P>
where
	C: ProvideRuntimeApi<B>,
	C: HeaderMetadata<B, Error = BlockChainError> + HeaderBackend<B> + 'static,
	C: Send + Sync + 'static,
	B: BlockT<Hash = H256> + Send + Sync + 'static,
	P: TransactionPool<Block = B> + Send + Sync + 'static,
	C::Api: TxPoolRuntimeApi<B>,
{
	fn content(&self) -> RpcResult<TxPoolResult<TransactionMap<Transaction>>> {
		let txs: Vec<<B as BlockT>::Extrinsic> = self
			.pool
			.ready()
			.map(|in_pool_tx| in_pool_tx.data().clone())
			.collect();

		let best_block: BlockId<B> = BlockId::Hash(self.client.info().best_hash);
		let ethereum_txns = self
			.client
			.runtime_api()
			.extrinsic_filter(&best_block, txs)
			.map_err(|err| {
				internal_err(format!("fetch runtime extrinsic filter failed: {:?}", err))
			})?;
		let mut pending: TransactionMap<Transaction> = HashMap::new();
		for txn in ethereum_txns.iter() {
			let transaction_message = TransactionMessage::from(txn.clone());
			let hash = transaction_message.hash();
			let from_address = match public_key(txn, hash) {
				Ok(pk) => H160::from(H256::from_slice(Keccak256::digest(&pk).as_slice())),
				Err(_e) => H160::default(),
			};

			if !pending.contains_key(&from_address) {
				pending.insert(from_address, HashMap::new());
			}

			if let Some(nonce_map) = pending.get_mut(&from_address) {
				nonce_map.insert(
					txn.nonce,
					Transaction {
						hash,
						nonce: txn.nonce,
						block_hash: Some(H256::default()), // or None?
						block_number: None,
						from: from_address,
						to: match txn.action {
							TransactionAction::Call(to) => Some(to),
							_ => None,
						},
						value: txn.value,
						gas_price: txn.gas_price,
						gas: txn.gas_limit,
						input: Bytes(txn.input.clone()),
					},
				);
			}
		}
		Ok(TxPoolResult {
			pending,
			// Future queue not yet supported. We need to do something like:
			// - Use InpoolTransaction::requires() to get the TransactionTag bytes.
			// - Somehow decode and identify the tag to either add it to the future or pending pool.
			queued: HashMap::new(),
		})
	}

	fn inspect(&self) -> RpcResult<TxPoolResult<TransactionMap<Summary>>> {
		unimplemented!();
	}

	fn status(&self) -> RpcResult<TxPoolResult<U256>> {
		let status = self.pool.status();
		Ok(TxPoolResult {
			pending: U256::from(status.ready),
			queued: U256::from(status.future),
		})
	}
}
