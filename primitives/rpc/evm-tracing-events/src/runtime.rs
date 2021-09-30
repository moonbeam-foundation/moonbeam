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

use super::{opcodes_string, Context};
use alloc::vec::Vec;
use codec::{Decode, Encode};
use ethereum_types::{H160, H256, U256};
pub use evm::{ExitError, ExitReason, ExitSucceed, Opcode};

#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq)]
pub struct Stack {
	pub data: Vec<H256>,
	pub limit: u64,
}

impl From<&evm::Stack> for Stack {
	fn from(i: &evm::Stack) -> Self {
		Self {
			data: i.data().clone(),
			limit: i.limit() as u64,
		}
	}
}

#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq)]
pub struct Memory {
	pub data: Vec<u8>,
	pub effective_len: U256,
	pub limit: u64,
}

impl From<&evm::Memory> for Memory {
	fn from(i: &evm::Memory) -> Self {
		Self {
			data: i.data().clone(),
			effective_len: i.effective_len(),
			limit: i.limit() as u64,
		}
	}
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Encode, Decode)]
pub enum Capture<E, T> {
	/// The machine has exited. It cannot be executed again.
	Exit(E),
	/// The machine has trapped. It is waiting for external information, and can
	/// be executed again.
	Trap(T),
}

pub type Trap = Vec<u8>; // Should hold the marshalled Opcode.

#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq)]
pub enum RuntimeEvent {
	Step {
		context: Context,
		// This needs to be marshalled in the runtime no matter what.
		opcode: Vec<u8>,
		// We can use ExitReason with `with-codec` feature,
		position: Result<u64, ExitReason>,
		stack: Stack,
		memory: Memory,
	},
	StepResult {
		result: Result<(), Capture<ExitReason, Trap>>,
		return_value: Vec<u8>,
	},
	SLoad {
		address: H160,
		index: H256,
		value: H256,
	},
	SStore {
		address: H160,
		index: H256,
		value: H256,
	},
}

#[cfg(feature = "evm-tracing")]
impl<'a> From<evm_runtime::tracing::Event<'a>> for RuntimeEvent {
	fn from(i: evm_runtime::tracing::Event<'a>) -> Self {
		match i {
			evm_runtime::tracing::Event::Step {
				context,
				opcode,
				position,
				stack,
				memory,
			} => Self::Step {
				context: context.clone().into(),
				opcode: opcodes_string(opcode),
				position: match position {
					Ok(position) => Ok(*position as u64),
					Err(e) => Err(e.clone()),
				},
				stack: stack.into(),
				memory: memory.into(),
			},
			evm_runtime::tracing::Event::StepResult {
				result,
				return_value,
			} => Self::StepResult {
				result: match result {
					Ok(_) => Ok(()),
					Err(capture) => match capture {
						evm::Capture::Exit(e) => Err(Capture::Exit(e.clone())),
						evm::Capture::Trap(t) => Err(Capture::Trap(opcodes_string(*t))),
					},
				},
				return_value: return_value.to_vec(),
			},
			evm_runtime::tracing::Event::SLoad {
				address,
				index,
				value,
			} => Self::SLoad {
				address,
				index,
				value,
			},
			evm_runtime::tracing::Event::SStore {
				address,
				index,
				value,
			} => Self::SStore {
				address,
				index,
				value,
			},
		}
	}
}
