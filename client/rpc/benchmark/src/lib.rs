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
use jsonrpc_core::{
	Result as RpcResult,
	types::{Error, ErrorCode},
};
use moonbeam_rpc_core_benchmark::{Benchmark as BenchmarkT, BenchmarkResults};

use ethereum_types::{H160, H256};
use fc_rpc::{internal_err};
use fc_rpc_core::types::{Bytes, CallRequest};
use sha3::{Keccak256, Digest};
use log;

pub struct Benchmark {}

impl BenchmarkT for Benchmark {

	/// Handler for `debug_benchmarkRawTransaction` request. Times the execution of the provided raw
	/// transaction and discards the results.
	fn benchmark_raw_transaction(&self, bytes: Bytes) -> RpcResult<BenchmarkResults> {

		log::warn!("benchmark_raw_transaction()");

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

		return match transaction.action {
			pallet_ethereum::TransactionAction::Call(to) => {

				let data: Option<Bytes> = Some(Bytes::from(transaction.input.clone()));
				let request = CallRequest {
					from: recover_signer(&transaction),
					to: Some(to),
					gas_price: Some(transaction.gas_price),
					gas: Some(transaction.gas_limit),
					value: Some(transaction.value),
					data,
					nonce: None,
				};
				self.benchmark_call(request)
			},
			pallet_ethereum::TransactionAction::Create => {
				Err(Error{
					code: ErrorCode::InternalError,
					message: "Create not supported in Benchmark API".into(),
					data: None
				})
			}
		}
	}

	fn benchmark_call(&self, _request: CallRequest) -> RpcResult<BenchmarkResults> {
		// TODO: this should behave much like the existing estimate_gas RPC call
		Ok(Default::default())
	}
}
