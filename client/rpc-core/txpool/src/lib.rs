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

use ethereum_types::U256;
use jsonrpc_core::Result;
use jsonrpc_derive::rpc;

mod types;

pub use crate::types::{Get as GetT, Summary, Transaction, TransactionMap, TxPoolResult};

pub use rpc_impl_TxPool::gen_server::TxPool as TxPoolServer;

#[rpc(server)]
pub trait TxPool {
	#[rpc(name = "txpool_content")]
	fn content(&self) -> Result<TxPoolResult<TransactionMap<Transaction>>>;

	#[rpc(name = "txpool_inspect")]
	fn inspect(&self) -> Result<TxPoolResult<TransactionMap<Summary>>>;

	#[rpc(name = "txpool_status")]
	fn status(&self) -> Result<TxPoolResult<U256>>;
}
