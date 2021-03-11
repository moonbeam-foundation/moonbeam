// Copyright 2019-2020 PureStake Inc.
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

use futures::{
	compat::Compat,
	future::{BoxFuture, FutureExt, TryFutureExt},
	sink::SinkExt,
};
use std::{future::Future, marker::PhantomData, pin::Pin, sync::Arc};
use tokio::sync::oneshot;

use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use sc_client_api::backend::{AuxStore, Backend, StateBackend};
use sp_api::{BlockId, HeaderT, ProvideRuntimeApi};
use sp_block_builder::BlockBuilder;
use sp_blockchain::{
	Backend as BlockchainBackend, Error as BlockChainError, HeaderBackend, HeaderMetadata,
};
use sp_runtime::traits::{BlakeTwo256, Block as BlockT};
use sp_utils::mpsc::TracingUnboundedSender;

use ethereum_types::{H128, H256};
use fp_rpc::EthereumRuntimeRPCApi;

use moonbeam_rpc_core_trace::{FilterRequest, FilterResponse, Trace as TraceT};
use moonbeam_rpc_primitives_debug::{DebugRuntimeApi, TraceType};

pub struct Trace {
	pub requester: TraceFilterCacheRequester,
}

impl TraceT for Trace {
	fn filter(
		&self,
		filter: FilterRequest,
	) -> Compat<BoxFuture<'static, jsonrpc_core::Result<FilterResponse>>> {
		let mut requester = self.requester.clone();

		async move {
			let (tx, rx) = oneshot::channel();

			requester.send((filter, tx)).await.map_err(|err| {
				internal_err(format!(
					"failed to send request to trace filter service : {:?}",
					err
				))
			})?;

			rx.await.map_err(|err| {
				internal_err(format!(
					"trace filter service dropped the channel : {:?}",
					err
				))
			})?
		}
		.boxed()
		.compat()
	}
}

fn internal_err<T: ToString>(message: T) -> RpcError {
	RpcError {
		code: ErrorCode::InternalError,
		message: message.to_string(),
		data: None,
	}
}

pub type TraceFilterCacheRequester =
	TracingUnboundedSender<(FilterRequest, oneshot::Sender<Result<FilterResponse>>)>;

pub struct TraceFilterCache<B, C, BE>(PhantomData<(B, C, BE)>);

impl<B, C, BE> TraceFilterCache<B, C, BE>
where
	BE: Backend<B> + 'static,
	BE::State: StateBackend<BlakeTwo256>,
	BE::Blockchain: BlockchainBackend<B>,
	C: ProvideRuntimeApi<B> + AuxStore,
	C: HeaderMetadata<B, Error = BlockChainError> + HeaderBackend<B>,
	C: Send + Sync + 'static,
	B: BlockT<Hash = H256> + Send + Sync + 'static,
	C::Api: BlockBuilder<B, Error = BlockChainError>,
	C::Api: DebugRuntimeApi<B>,
	C::Api: EthereumRuntimeRPCApi<B>,
{
	pub fn task(
		client: Arc<C>,
		backend: Arc<BE>,
	) -> (impl Future<Output = ()>, TraceFilterCacheRequester) {
		let (tx, rx) = sp_utils::mpsc::tracing_unbounded("trace-filter-cache-requester");

		let fut = async move {
			todo!()

			// TODO :
			// 1. Handle requests and add traces to the cache with expiration dates
			// 2. Remove expired cache
		};

		(fut, tx)
	}
}
