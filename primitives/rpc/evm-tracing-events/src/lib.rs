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

//! A Proxy in this context is an environmental trait implementor meant to be used for capturing
//! EVM trace events sent to a Host function from the Runtime. Works like:
//! - Runtime Api call `using` environmental.
//! - Runtime calls a Host function with some scale-encoded Evm event.
//! - Host function emits an additional event to this Listener.
//! - Proxy listens for the event and format the actual trace response.
//!
//! There are two proxy types: `Raw` and `CallList`.
//! - `Raw` - used for opcode-level traces.
//! - `CallList` - used for block tracing (stack of call stacks) and custom tracing outputs.
//!
//! The EVM event types may contain references and not implement Encode/Decode.
//! This module provide mirror types and conversion into them from the original events.

#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

pub mod evm;
pub mod gasometer;
pub mod runtime;

pub use self::evm::EvmEvent;
pub use gasometer::GasometerEvent;
pub use runtime::RuntimeEvent;

use ::evm::Opcode;
use alloc::{vec, vec::Vec};
use codec::{Decode, Encode};
use ethereum_types::{H160, H256, U256};
use moonbeam_rpc_primitives_debug::api::CallType;

environmental::environmental!(listener: dyn Listener + 'static);

pub fn using<R, F: FnOnce() -> R>(l: &mut (dyn Listener + 'static), f: F) -> R {
	listener::using(l, f)
}

#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode)]
pub enum Event {
	Evm(evm::EvmEvent),
	Gasometer(gasometer::GasometerEvent),
	Runtime(runtime::RuntimeEvent),
	CallListNew(),
}

impl Event {
	/// Access the global reference and call it's `event` method, passing the `Event` itself as
	/// argument.
	///
	/// This only works if we are `using` a global reference to a `Listener` implementor.
	pub fn emit(self) {
		listener::with(|listener| listener.event(self));
	}
}

/// Main trait to proxy emitted messages.
/// Used 2 times :
/// - Inside the runtime to proxy the events throught the host functions
/// - Inside the client to forward those events to the client listener.
pub trait Listener {
	fn event(&mut self, event: Event);
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

#[derive(Debug)]
pub enum ContextType {
	Call(CallType),
	Create,
}

impl ContextType {
	pub fn from(opcode: Vec<u8>) -> Option<Self> {
		let opcode = match alloc::str::from_utf8(&opcode[..]) {
			Ok(op) => op.to_uppercase(),
			_ => return None,
		};
		match &opcode[..] {
			"CREATE" | "CREATE2" => Some(ContextType::Create),
			"CALL" => Some(ContextType::Call(CallType::Call)),
			"CALLCODE" => Some(ContextType::Call(CallType::CallCode)),
			"DELEGATECALL" => Some(ContextType::Call(CallType::DelegateCall)),
			"STATICCALL" => Some(ContextType::Call(CallType::StaticCall)),
			_ => None,
		}
	}
}

pub fn convert_memory(memory: Vec<u8>) -> Vec<H256> {
	let size = 32;
	memory
		.chunks(size)
		.map(|c| {
			let mut msg = [0u8; 32];
			let chunk = c.len();
			if chunk < size {
				let left = size - chunk;
				let remainder = vec![0; left];
				msg[0..left].copy_from_slice(&remainder[..]);
				msg[left..size].copy_from_slice(c);
			} else {
				msg[0..size].copy_from_slice(c)
			}
			H256::from_slice(&msg[..])
		})
		.collect()
}

/// Converts an Opcode into its name, stored in a `Vec<u8>`.
pub fn opcodes_string(opcode: Opcode) -> Vec<u8> {
	let tmp;
	let out = match opcode {
		Opcode(0) => "Stop",
		Opcode(1) => "Add",
		Opcode(2) => "Mul",
		Opcode(3) => "Sub",
		Opcode(4) => "Div",
		Opcode(5) => "SDiv",
		Opcode(6) => "Mod",
		Opcode(7) => "SMod",
		Opcode(8) => "AddMod",
		Opcode(9) => "MulMod",
		Opcode(10) => "Exp",
		Opcode(11) => "SignExtend",
		Opcode(16) => "Lt",
		Opcode(17) => "Gt",
		Opcode(18) => "Slt",
		Opcode(19) => "Sgt",
		Opcode(20) => "Eq",
		Opcode(21) => "IsZero",
		Opcode(22) => "And",
		Opcode(23) => "Or",
		Opcode(24) => "Xor",
		Opcode(25) => "Not",
		Opcode(26) => "Byte",
		Opcode(27) => "Shl",
		Opcode(28) => "Shr",
		Opcode(29) => "Sar",
		Opcode(32) => "Keccak256",
		Opcode(48) => "Address",
		Opcode(49) => "Balance",
		Opcode(50) => "Origin",
		Opcode(51) => "Caller",
		Opcode(52) => "CallValue",
		Opcode(53) => "CallDataLoad",
		Opcode(54) => "CallDataSize",
		Opcode(55) => "CallDataCopy",
		Opcode(56) => "CodeSize",
		Opcode(57) => "CodeCopy",
		Opcode(58) => "GasPrice",
		Opcode(59) => "ExtCodeSize",
		Opcode(60) => "ExtCodeCopy",
		Opcode(61) => "ReturnDataSize",
		Opcode(62) => "ReturnDataCopy",
		Opcode(63) => "ExtCodeHash",
		Opcode(64) => "BlockHash",
		Opcode(65) => "Coinbase",
		Opcode(66) => "Timestamp",
		Opcode(67) => "Number",
		Opcode(68) => "Difficulty",
		Opcode(69) => "GasLimit",
		Opcode(70) => "ChainId",
		Opcode(80) => "Pop",
		Opcode(81) => "MLoad",
		Opcode(82) => "MStore",
		Opcode(83) => "MStore8",
		Opcode(84) => "SLoad",
		Opcode(85) => "SStore",
		Opcode(86) => "Jump",
		Opcode(87) => "JumpI",
		Opcode(88) => "GetPc",
		Opcode(89) => "MSize",
		Opcode(90) => "Gas",
		Opcode(91) => "JumpDest",
		Opcode(96) => "Push1",
		Opcode(97) => "Push2",
		Opcode(98) => "Push3",
		Opcode(99) => "Push4",
		Opcode(100) => "Push5",
		Opcode(101) => "Push6",
		Opcode(102) => "Push7",
		Opcode(103) => "Push8",
		Opcode(104) => "Push9",
		Opcode(105) => "Push10",
		Opcode(106) => "Push11",
		Opcode(107) => "Push12",
		Opcode(108) => "Push13",
		Opcode(109) => "Push14",
		Opcode(110) => "Push15",
		Opcode(111) => "Push16",
		Opcode(112) => "Push17",
		Opcode(113) => "Push18",
		Opcode(114) => "Push19",
		Opcode(115) => "Push20",
		Opcode(116) => "Push21",
		Opcode(117) => "Push22",
		Opcode(118) => "Push23",
		Opcode(119) => "Push24",
		Opcode(120) => "Push25",
		Opcode(121) => "Push26",
		Opcode(122) => "Push27",
		Opcode(123) => "Push28",
		Opcode(124) => "Push29",
		Opcode(125) => "Push30",
		Opcode(126) => "Push31",
		Opcode(127) => "Push32",
		Opcode(128) => "Dup1",
		Opcode(129) => "Dup2",
		Opcode(130) => "Dup3",
		Opcode(131) => "Dup4",
		Opcode(132) => "Dup5",
		Opcode(133) => "Dup6",
		Opcode(134) => "Dup7",
		Opcode(135) => "Dup8",
		Opcode(136) => "Dup9",
		Opcode(137) => "Dup10",
		Opcode(138) => "Dup11",
		Opcode(139) => "Dup12",
		Opcode(140) => "Dup13",
		Opcode(141) => "Dup14",
		Opcode(142) => "Dup15",
		Opcode(143) => "Dup16",
		Opcode(144) => "Swap1",
		Opcode(145) => "Swap2",
		Opcode(146) => "Swap3",
		Opcode(147) => "Swap4",
		Opcode(148) => "Swap5",
		Opcode(149) => "Swap6",
		Opcode(150) => "Swap7",
		Opcode(151) => "Swap8",
		Opcode(152) => "Swap9",
		Opcode(153) => "Swap10",
		Opcode(154) => "Swap11",
		Opcode(155) => "Swap12",
		Opcode(156) => "Swap13",
		Opcode(157) => "Swap14",
		Opcode(158) => "Swap15",
		Opcode(159) => "Swap16",
		Opcode(160) => "Log0",
		Opcode(161) => "Log1",
		Opcode(162) => "Log2",
		Opcode(163) => "Log3",
		Opcode(164) => "Log4",
		Opcode(176) => "JumpTo",
		Opcode(177) => "JumpIf",
		Opcode(178) => "JumpSub",
		Opcode(180) => "JumpSubv",
		Opcode(181) => "BeginSub",
		Opcode(182) => "BeginData",
		Opcode(184) => "ReturnSub",
		Opcode(185) => "PutLocal",
		Opcode(186) => "GetLocal",
		Opcode(225) => "SLoadBytes",
		Opcode(226) => "SStoreBytes",
		Opcode(227) => "SSize",
		Opcode(240) => "Create",
		Opcode(241) => "Call",
		Opcode(242) => "CallCode",
		Opcode(243) => "Return",
		Opcode(244) => "DelegateCall",
		Opcode(245) => "Create2",
		Opcode(250) => "StaticCall",
		Opcode(252) => "TxExecGas",
		Opcode(253) => "Revert",
		Opcode(254) => "Invalid",
		Opcode(255) => "SelfDestruct",
		Opcode(n) => {
			tmp = alloc::format!("Unknown({})", n);
			&tmp
		}
	};
	out.as_bytes().to_vec()
}
