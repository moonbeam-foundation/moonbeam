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
use std::{time::Instant, marker::PhantomData, sync::Arc};
use jsonrpc_core::{
	Result as RpcResult,
};
pub use moonbeam_rpc_core_benchmark::{Benchmark as BenchmarkT, BenchmarkResults, BenchmarkServer};

use sp_api::{BlockId, ProvideRuntimeApi};
use sp_blockchain::{Error as BlockChainError, HeaderMetadata, HeaderBackend};
use sp_runtime::traits::Block as BlockT;
use ethereum_types::{H160, H256};
use fc_rpc::{internal_err};
use fc_rpc_core::types::{Bytes, CallRequest};
use fp_rpc::EthereumRuntimeRPCApi;
use sha3::{Keccak256, Digest};
use log;

pub struct Benchmark<B: BlockT, C> {
	client: Arc<C>,
	_marker: PhantomData<B>,
}

impl<B: BlockT, C> Benchmark<B, C> {
	pub fn new(client: Arc<C>) -> Self {
		Self {
			client,
			_marker: PhantomData,
		}
	}
}

impl<B, C> BenchmarkT for Benchmark<B, C>
where
	C: ProvideRuntimeApi<B>,
	C: HeaderBackend<B> + HeaderMetadata<B, Error=BlockChainError> + 'static,
	C::Api: EthereumRuntimeRPCApi<B>,
	C: Send + Sync + 'static,
	B: BlockT<Hash = H256> + Send + Sync + 'static,
{

	/// Handler for `debug_benchmarkRawTransaction` request. Times the execution of the provided raw
	/// transaction and discards the results.
	fn benchmark_raw_transaction(&self, bytes: Bytes) -> RpcResult<BenchmarkResults> {
		let transaction = match rlp::decode::<ethereum::Transaction>(&bytes.0[..]) {
			Ok(transaction) => transaction,
			Err(_) => return Err(internal_err("decode transaction failed"))
		};

		// TODO: copied from pallet_ethereum, import from there?
		fn recover_signer(transaction: &ethereum::Transaction) -> Option<H160> {
			let mut sig = [0u8; 65];
			let mut msg = [0u8; 32];
			sig[0..32].copy_from_slice(&transaction.signature.r()[..]);
			sig[32..64].copy_from_slice(&transaction.signature.s()[..]);
			sig[64] = transaction.signature.standard_v();
			msg.copy_from_slice(&pallet_ethereum::TransactionMessage::from(transaction.clone()).hash()[..]);

			let pubkey = sp_io::crypto::secp256k1_ecdsa_recover(&sig, &msg).ok()?;
			Some(H160::from(H256::from_slice(Keccak256::digest(&pubkey).as_slice())))
		}

		let to: Option<H160> = match transaction.action {
			pallet_ethereum::TransactionAction::Call(to) => Some(to),
			pallet_ethereum::TransactionAction::Create => None,
		};

		let data: Option<Bytes> = Some(Bytes::from(transaction.input.clone()));
		let request = CallRequest {
			from: recover_signer(&transaction),
			to,
			gas_price: Some(transaction.gas_price),
			gas: Some(transaction.gas_limit),
			value: Some(transaction.value),
			data,
			nonce: None,
		};

		self.benchmark_call(request)
	}

	fn benchmark_call(&self, request: CallRequest) -> RpcResult<BenchmarkResults> {
		let hash = self.client.info().best_hash;
		let runtime_api = self.client.runtime_api();

		// use given gas limit or query current block's limit
		let gas_limit = match request.gas {
			Some(amount) => amount,
			None => {
				log::warn!("querying gas limit...");
				let block = self.client.runtime_api().current_block(&BlockId::Hash(hash))
					.map_err(|err| internal_err(format!("runtime error: {:?}", err)))?;
				if let Some(block) = block {
					block.header.gas_limit
				} else {
					return Err(internal_err(format!("block unavailable, cannot query gas limit")));
				}
			},
		};

		let data = request.data.map(|d| d.0).unwrap_or_default();

		let start_time = Instant::now();

		return match request.to {
			Some(to) => {
				let info = runtime_api.call(
					&BlockId::Hash(hash),
					request.from.unwrap_or_default(),
					to,
					data,
					request.value.unwrap_or_default(),
					gas_limit,
					request.gas_price,
					request.nonce,
					true, // estimate (TODO: do we want estimate?)
				)
				.map_err(|err| internal_err(format!("runtime error: {:?}", err)))?
				.map_err(|err| internal_err(format!("execution fatal: {:?}", err)))?;

				let elapsed = start_time.elapsed();

				Ok(BenchmarkResults {
					gas_used: info.used_gas,
					evm_execution_time_us: elapsed.as_micros() as u64,
					// TODO: not sure request time is worth the effort, extra code, etc.
					request_execution_time_us: 0,
					result: None, // TODO: Bytes vs Vec<u8> is quite annoying...
				})
			},
			None => {
				Err(internal_err(format!("create not yet supported in benchmarking API"))) // TODO
			}
		}
	}
}
