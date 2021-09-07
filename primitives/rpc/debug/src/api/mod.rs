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

//! Runtime API allowing to debug/trace Ethereum

use codec::{Decode, Encode};
use ethereum::TransactionV0 as Transaction;
use ethereum_types::H160;
use sp_std::vec::Vec;

#[cfg(feature = "std")]
use crate::serialization::*;
#[cfg(feature = "std")]
use serde::Serialize;

pub mod block;
pub mod single;

pub const V2_RUNTIME_VERSION: u32 = 400;

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

#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Serialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub enum CallResult {
	Output(#[cfg_attr(feature = "std", serde(serialize_with = "bytes_0x_serialize"))] Vec<u8>),
	// field "error"
	Error(#[cfg_attr(feature = "std", serde(serialize_with = "string_serialize"))] Vec<u8>),
}

#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Serialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase", untagged))]
pub enum CreateResult {
	Error {
		#[cfg_attr(feature = "std", serde(serialize_with = "string_serialize"))]
		error: Vec<u8>,
	},
	Success {
		#[cfg_attr(feature = "std", serde(rename = "createdContractAddressHash"))]
		created_contract_address_hash: H160,
		#[cfg_attr(
			feature = "std",
			serde(serialize_with = "bytes_0x_serialize", rename = "createdContractCode")
		)]
		created_contract_code: Vec<u8>,
	},
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Serialize))]
#[cfg_attr(feature = "std", serde(rename_all = "lowercase"))]
pub enum CallType {
	Call,
	CallCode,
	DelegateCall,
	StaticCall,
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Serialize))]
#[cfg_attr(feature = "std", serde(rename_all = "lowercase"))]
pub enum CreateType {
	Create,
}
