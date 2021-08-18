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

extern crate alloc;

use alloc::vec::Vec;
use codec::{Decode, Encode};
use ethereum_types::{H160, H256, U256};

#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq)]
pub struct Transfer {
	/// Source address.
	pub source: H160,
	/// Target address.
	pub target: H160,
	/// Transfer value.
	pub value: U256,
}

impl From<evm_runtime::Transfer> for Transfer {
	fn from(i: evm_runtime::Transfer) -> Self {
		Self {
			source: i.source,
			target: i.target,
			value: i.value,
		}
	}
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, Encode, Decode)]
pub enum CreateScheme {
	/// Legacy create scheme of `CREATE`.
	Legacy {
		/// Caller of the create.
		caller: H160,
	},
	/// Create scheme of `CREATE2`.
	Create2 {
		/// Caller of the create.
		caller: H160,
		/// Code hash.
		code_hash: H256,
		/// Salt.
		salt: H256,
	},
	/// Create at a fixed location.
	Fixed(H160),
}

impl From<evm_runtime::CreateScheme> for CreateScheme {
	fn from(i: evm_runtime::CreateScheme) -> Self {
		match i {
			evm_runtime::CreateScheme::Legacy { caller } => Self::Legacy { caller },
			evm_runtime::CreateScheme::Create2 {
				caller,
				code_hash,
				salt,
			} => Self::Create2 {
				caller,
				code_hash,
				salt,
			},
			evm_runtime::CreateScheme::Fixed(address) => Self::Fixed(address),
		}
	}
}

#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq)]
pub struct Context {
	/// Execution address.
	pub address: H160,
	/// Caller of the EVM.
	pub caller: H160,
	/// Apparent value of the EVM.
	pub apparent_value: U256,
}

impl From<evm_runtime::Context> for Context {
	fn from(i: evm_runtime::Context) -> Self {
		Self {
			address: i.address,
			caller: i.caller,
			apparent_value: i.apparent_value,
		}
	}
}

#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq)]
pub enum EvmEvent {
	Call {
		code_address: H160,
		transfer: Option<Transfer>,
		input: Vec<u8>,
		target_gas: Option<u64>,
		is_static: bool,
		context: Context,
	},
	Create {
		caller: H160,
		address: H160,
		scheme: CreateScheme,
		value: U256,
		init_code: Vec<u8>,
		target_gas: Option<u64>,
	},
	Suicide {
		address: H160,
		target: H160,
		balance: U256,
	},
}

impl<'a> From<evm::tracing::Event<'a>> for EvmEvent {
	fn from(i: evm::tracing::Event<'a>) -> Self {
		match i {
			evm::tracing::Event::Call {
				code_address,
				transfer,
				input,
				target_gas,
				is_static,
				context,
			} => Self::Call {
				code_address,
				transfer: if let Some(transfer) = transfer {
					Some(transfer.clone().into())
				} else {
					None
				},
				input: input.to_vec(),
				target_gas,
				is_static,
				context: context.clone().into(),
			},
			evm::tracing::Event::Create {
				caller,
				address,
				scheme,
				value,
				init_code,
				target_gas,
			} => Self::Create {
				caller,
				address,
				scheme: scheme.into(),
				value,
				init_code: init_code.to_vec(),
				target_gas,
			},
			evm::tracing::Event::Suicide {
				address,
				target,
				balance,
			} => Self::Suicide {
				address,
				target,
				balance,
			},
		}
	}
}
