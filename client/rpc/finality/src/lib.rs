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
use futures::{future::BoxFuture, FutureExt as _};
use jsonrpc_core::Result as RpcResult;
use jsonrpc_derive::rpc;
use std::sync::Arc;

use parity_scale_codec::Encode;

/// An RPC endpoint to check for finality of blocks and transactions in Moonbeam
#[rpc(server)]
pub trait MoonbeamFinalityApi {
	/// Reports whether a Moonbeam or Ethereum block is finalized.
	/// Errors if the block is not found. //TODO reevaluate design later. Could just return false.
	#[rpc(name = "moon_isBlockFinalized")]
	fn is_block_finalized(&self, block_hash: Vec<u8>) -> BoxFuture<'static, RpcResult<bool>>;

	/// Reports whether a Moonbeam or Ethereum transaction is finalized.
	#[rpc(name = "moon_isTxFinalized")]
	fn is_tx_finalized(&self, tx_hash: Vec<u8>) -> BoxFuture<'static, RpcResult<bool>>;
}

pub struct MoonbeamFinality<C> {
	pub client: Arc<C>,
}

impl<C> MoonbeamFinalityApi for MoonbeamFinality<C>
where
	C: Send + Sync + 'static,
{
	fn is_block_finalized(&self, block_hash: Vec<u8>) -> BoxFuture<'static, RpcResult<bool>> {
		async move {
			//TODO actually check the finality XD

			Ok(false)
		}
		.boxed()
	}

	fn is_tx_finalized(&self, tx_hash: Vec<u8>) -> BoxFuture<'static, RpcResult<bool>> {
		async move {
			//TODO actually check the finality XD

			Ok(false)
		}
		.boxed()
	}
}

// This bit cribbed from frontier.
pub fn internal_err<T: ToString>(message: T) -> jsonrpc_core::Error {
	jsonrpc_core::Error {
		code: jsonrpc_core::ErrorCode::InternalError,
		message: message.to_string(),
		data: None,
	}
}
