// Copyright 2019-2022 PureStake Inc.
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

use ethereum::AccessListItem;
use ethereum_types::{H160, H256, U256};
use fc_rpc_core::types::Bytes;
use jsonrpsee::{core::RpcResult, proc_macros::rpc};
use moonbeam_client_evm_tracing::types::single;
use moonbeam_rpc_core_types::RequestBlockId;
use serde::Deserialize;

#[derive(Clone, Eq, PartialEq, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TraceParams {
	pub disable_storage: Option<bool>,
	pub disable_memory: Option<bool>,
	pub disable_stack: Option<bool>,
	/// Javascript tracer (we just check if it's Blockscout tracer string)
	pub tracer: Option<String>,
	pub tracer_config: Option<single::TraceCallConfig>,
	pub timeout: Option<String>,
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TraceCallParams {
	/// Sender
	pub from: Option<H160>,
	/// Recipient
	pub to: H160,
	/// Gas Price, legacy.
	pub gas_price: Option<U256>,
	/// Max BaseFeePerGas the user is willing to pay.
	pub max_fee_per_gas: Option<U256>,
	/// The miner's tip.
	pub max_priority_fee_per_gas: Option<U256>,
	/// Gas
	pub gas: Option<U256>,
	/// Value of transaction in wei
	pub value: Option<U256>,
	/// Additional data sent with transaction
	pub data: Option<Bytes>,
	/// Nonce
	pub nonce: Option<U256>,
	/// EIP-2930 access list
	pub access_list: Option<Vec<AccessListItem>>,
	/// EIP-2718 type
	#[serde(rename = "type")]
	pub transaction_type: Option<U256>,
}

#[rpc(server)]
#[jsonrpsee::core::async_trait]
pub trait Debug {
	#[method(name = "debug_traceTransaction")]
	async fn trace_transaction(
		&self,
		transaction_hash: H256,
		params: Option<TraceParams>,
	) -> RpcResult<single::TransactionTrace>;
	#[method(name = "debug_traceCall")]
	async fn trace_call(
		&self,
		call_params: TraceCallParams,
		id: RequestBlockId,
		params: Option<TraceParams>,
	) -> RpcResult<single::TransactionTrace>;
	#[method(name = "debug_traceBlockByNumber", aliases = ["debug_traceBlockByHash"])]
	async fn trace_block(
		&self,
		id: RequestBlockId,
		params: Option<TraceParams>,
	) -> RpcResult<Vec<single::TransactionTrace>>;
}
