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
use ethereum_types::{H256, U256};
use serde::{Serialize, Serializer};
use std::collections::BTreeMap;

#[derive(Debug)]
pub enum Opcode {
	/// `STOP`
	Stop,
	/// `ADD`
	Add,
	/// `MUL`
	Mul,
	/// `SUB`
	Sub,
	/// `DIV`
	Div,
	/// `SDIV`
	SDiv,
	/// `MOD`
	Mod,
	/// `SMOD`
	SMod,
	/// `ADDMOD`
	AddMod,
	/// `MULMOD`
	MulMod,
	/// `EXP`
	Exp,
	/// `SIGNEXTEND`
	SignExtend,

	/// `LT`
	Lt,
	/// `GT`
	Gt,
	/// `SLT`
	SLt,
	/// `SGT`
	SGt,
	/// `EQ`
	Eq,
	/// `ISZERO`
	IsZero,
	/// `AND`
	And,
	/// `OR`
	Or,
	/// `XOR`
	Xor,
	/// `NOT`
	Not,
	/// `BYTE`
	Byte,

	/// `CALLDATALOAD`
	CallDataLoad,
	/// `CALLDATASIZE`
	CallDataSize,
	/// `CALLDATACOPY`
	CallDataCopy,
	/// `CODESIZE`
	CodeSize,
	/// `CODECOPY`
	CodeCopy,

	/// `SHL`
	Shl,
	/// `SHR`
	Shr,
	/// `SAR`
	Sar,

	/// `POP`
	Pop,
	/// `MLOAD`
	MLoad,
	/// `MSTORE`
	MStore,
	/// `MSTORE8`
	MStore8,
	/// `JUMP`
	Jump,
	/// `JUMPI`
	JumpI,
	/// `PC`
	PC,
	/// `MSIZE`
	MSize,
	/// `JUMPDEST`
	JumpDest,

	/// `PUSHn`
	Push(u8),
	/// `DUPn`
	Dup(u8),
	/// `SWAPn`
	Swap(u8),

	/// `RETURN`
	Return,
	/// `REVERT`
	Revert,

	/// `INVALID`
	Invalid,
}

#[derive(Debug, Serialize)]
pub struct StepLog {
	depth: U256,
	//error:
	gas: U256,
	gas_cost: U256,
	memory: Vec<H256>,
	//pc:
	stack: Vec<H256>,
	storage: BTreeMap<H256, H256>,
}

impl std::fmt::Display for Opcode {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{:?}", self)
	}
}

fn opcode_serialize<S>(opcode: &Opcode, serializer: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	serializer.serialize_str(&format!(
		"{:?}",
		opcode
			.to_string()
			.to_uppercase()
			.replace("(", "")
			.replace(")", "")
	))
}
