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

#![cfg_attr(not(feature = "std"), no_std)]
// These functions are quite usefull, shoud it be moved into its own crate ?
#[cfg(feature = "std")]
pub mod serialization;

pub mod api;

pub mod v1;
pub mod v2;

use crate::api::*;
use ethereum::TransactionV0 as Transaction;
use sp_std::vec::Vec;

sp_api::decl_runtime_apis! {
	#[api_version(3)]
	pub trait DebugRuntimeApi {

		#[changed_in(2)]
		fn trace_transaction(
			extrinsics: Vec<Block::Extrinsic>,
			transaction: &Transaction,
			trace_type: single::TraceType,
		) -> Result<single::TransactionTrace, sp_runtime::DispatchError>;

		#[changed_in(2)]
		fn trace_block(
			extrinsics: Vec<Block::Extrinsic>,
		) -> Result<Vec<block::TransactionTrace>, sp_runtime::DispatchError>;

		#[changed_in(3)]
		fn trace_transaction(
			header: &Block::Header,
			extrinsics: Vec<Block::Extrinsic>,
			transaction: &Transaction,
			trace_type: single::TraceType,
		) -> Result<(), sp_runtime::DispatchError>;

		fn trace_transaction(
			header: &Block::Header,
			extrinsics: Vec<Block::Extrinsic>,
			transaction: &Transaction,
		) -> Result<(), sp_runtime::DispatchError>;

		fn trace_block(
			header: &Block::Header,
			extrinsics: Vec<Block::Extrinsic>,
		) -> Result<(), sp_runtime::DispatchError>;
	}
}

/// Runtime api closure result.
#[derive(Debug)]
pub enum Response {
	V1(v1::Response),
	V2(v2::Response),
}

impl From<v1::Response> for Response {
	fn from(source: v1::Response) -> Self {
		Self::V1(source)
	}
}

impl From<v2::Response> for Response {
	fn from(source: v2::Response) -> Self {
		Self::V2(source)
	}
}
