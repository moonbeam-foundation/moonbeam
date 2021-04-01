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

//! Substrate EVM tracing.
//!
//! The purpose of this extension is enable tracing the EVM opcode execution and will be used by
//! both Dapp developers - to get a granular view on their transactions - and indexers to access
//! the EVM callstack (internal transactions).
//!
//! The extension composed of two modules:
//!
//! - Runner - interfaces the runtime Api with the EVM Executor wrapper.
//! - Executor Wrapper - an evm::Handler implementor that wraps an evm::StackExecutor.
//!
//! The wrapper replaces the original recursive opcode execution done by the evm's
//! `create_inner` and `call_inner` methods, with one that allows capturing the intermediate machine
//! state between opcode executions (stepping), resulting in either a granular per opcode response:
//!
//! ```json
//! {
//!   "pc": 230,
//!   "op": "SSTORE",
//!   "gas": 62841,
//!   "gasCost": 20000,
//!   "depth": 1,
//!   "stack": [
//!     "00000000000000000000000000000000000000000000000000000000398f7223",
//!   ],
//!   "memory": [
//!     "0000000000000000000000000000000000000000000000000000000000000000",
//!   ],
//!   "storage": {"0x":"0x"}
//! }
//! ```
//!
//! or an overview of the internal transactions in a context type.
//!
//! ```json
//! [
//!  {
//!    "type": "call",
//!    "callType": "call",
//!    "from": "0xfe2882ac0a337a976aa73023c2a2a917f57ba2ed",
//!    "to": "0x3ca17a1c4995b95c600275e52da93d2e64dd591f",
//!    "input": "0x",
//!    "output": "0x",
//!    "traceAddress": [],
//!    "value": "0x0",
//!    "gas": "0xf9be",
//!    "gasUsed": "0xf9be"
//!  },
//!  {
//!    "type": "call",
//!    "callType": "call",
//!    "from": "0x3ca17a1c4995b95c600275e52da93d2e64dd591f",
//!    "to": "0x1416aa2a27db08ce3a81a01cdfc981417d28a6e6",
//!    "input": "0xfd63983b0000000000000000000000000000000000000000000000000000000000000006",
//!    "output": "0x000000000000000000000000000000000000000000000000000000000000000d",
//!    "traceAddress": [0],
//!    "value": "0x0",
//!    "gas": "0x9b9b",
//!    "gasUsed": "0x4f6d"
//!  }
//! ]
//! ```

#![cfg_attr(not(feature = "std"), no_std)]
pub mod executor;
pub mod runner;
