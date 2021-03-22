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

use jsonrpc_core::futures::Future;

use ethereum_types::{H160, H256};
use futures::{compat::Compat, future::BoxFuture};
use jsonrpc_derive::rpc;
pub use moonbeam_rpc_primitives_debug::block::TransactionTrace;
use serde::{de::Error, Deserialize, Deserializer};
use serde_hex::{CompactCapPfx, SerHex};

pub use rpc_impl_Trace::gen_server::Trace as TraceServer;

#[rpc(server)]
pub trait Trace {
	#[rpc(name = "trace_filter")]
	fn filter(
		&self,
		filter: FilterRequest,
	) -> Compat<BoxFuture<'static, jsonrpc_core::Result<Vec<TransactionTrace>>>>;
}

#[derive(Clone, Eq, PartialEq, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilterRequest {
	/// (optional?) From this block.
	pub from_block: Option<RequestBlockId>,

	/// (optional?) To this block.
	pub to_block: Option<RequestBlockId>,

	/// (optional) Sent from these addresses.
	pub from_address: Option<Vec<H160>>,

	/// (optional) Sent to these addresses.
	pub to_address: Option<Vec<H160>>,

	/// (optional) The offset trace number
	pub after: Option<u32>,

	/// (optional) Integer number of traces to display in a batch.
	pub count: Option<u32>,
}

#[derive(Clone, Eq, PartialEq, Debug, Deserialize)]
#[serde(rename_all = "camelCase", untagged)]
pub enum RequestBlockId {
	Number(u32),
	Tag(RequestBlockTag),
}

#[derive(Clone, Eq, PartialEq, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RequestBlockTag {
	Earliest,
	Latest,
	Pending,
}
