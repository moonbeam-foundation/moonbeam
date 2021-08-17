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

//! Environmental-aware externalities for EVM tracing in Wasm runtime. This enables
//! capturing the - potentially large - trace output data in the host and keep
//! a low memory footprint in `--execution=wasm`.
//!
//! - The original trace Runtime Api call is wrapped `using` environmental (thread local).
//! - Arguments are scale-encoded known types in the host.
//! - Host functions will decode the input and emit an event `with` environmental.

#![cfg_attr(not(feature = "std"), no_std)]
use sp_runtime_interface::runtime_interface;

use codec::Decode;
use sp_std::vec::Vec;

use ethereum_types::U256;
use moonbeam_rpc_primitives_debug::{
	proxy_v1::Event as EventV1,
	proxy_v2::Event as EventV2,
	single::{Call, RawStepLog},
	types::{EvmEvent, GasometerEvent, RuntimeEvent},
};

#[runtime_interface]
pub trait MoonbeamExt {
	// Old format to be deprecated.
	fn raw_step(&mut self, data: Vec<u8>) {
		let data: RawStepLog = Decode::decode(&mut &data[..]).unwrap();
		EventV1::RawStep(data).emit();
	}
	fn raw_gas(&mut self, data: Vec<u8>) {
		let data: U256 = Decode::decode(&mut &data[..]).unwrap();
		EventV1::RawGas(data).emit();
	}
	fn raw_return_value(&mut self, data: Vec<u8>) {
		EventV1::RawReturnValue(data).emit();
	}
	fn call_list_entry(&mut self, index: u32, value: Vec<u8>) {
		let value: Call = Decode::decode(&mut &value[..]).unwrap();
		EventV1::CallListEntry((index, value)).emit();
	}
	fn call_list_new(&mut self) {
		EventV1::CallListNew().emit();
	}
	// New design, proxy events.
	fn evm_event(&mut self, event: Vec<u8>) {
		let event: EvmEvent = Decode::decode(&mut &event[..]).unwrap();
		EventV2::Evm(event).emit();
	}
	fn gasometer_event(&mut self, event: Vec<u8>) {
		let event: GasometerEvent = Decode::decode(&mut &event[..]).unwrap();
		EventV2::Gasometer(event).emit();
	}
	fn runtime_event(&mut self, event: Vec<u8>) {
		let event: RuntimeEvent = Decode::decode(&mut &event[..]).unwrap();
		EventV2::Runtime(event).emit();
	}
}
