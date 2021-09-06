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
// use moonbeam_rpc_primitives_debug::{
// 	proxy::types::{EvmEvent, GasometerEvent, RuntimeEvent},
// 	proxy::v1::Event as EventV1,
// 	proxy::v2::Event as EventV2,
// 	single::{Call, RawStepLog},
// };

use moonbeam_rpc_primitives_debug::{
	api::single::{Call, RawStepLog},
	v1::Event as EventV1,
	v2::{Event as EventV2, EvmEvent, GasometerEvent, RuntimeEvent},
};

#[runtime_interface]
pub trait MoonbeamExt {
	// Old format to be deprecated.
	fn raw_step(&mut self, data: Vec<u8>) {
		if let Ok(data) = RawStepLog::decode(&mut &data[..]) {
			EventV1::RawStep(data).emit();
		} else {
			tracing::warn!("Failed to decode RawStepLog from bytes : {:?}", data);
		}
	}
	fn raw_gas(&mut self, data: Vec<u8>) {
		if let Ok(data) = U256::decode(&mut &data[..]) {
			EventV1::RawGas(data).emit();
		} else {
			tracing::warn!("Failed to decode U256 (raw_gas) from bytes : {:?}", data);
		}
	}
	fn raw_return_value(&mut self, data: Vec<u8>) {
		EventV1::RawReturnValue(data).emit();
	}
	fn call_list_entry(&mut self, index: u32, value: Vec<u8>) {
		if let Ok(value) = Call::decode(&mut &value[..]) {
			EventV1::CallListEntry((index, value)).emit();
		} else {
			tracing::warn!(
				"Failed to decode Call (call_list_entry) with index {} from bytes : {:?}",
				index,
				value
			);
		}
	}

	fn call_list_new(&mut self) {
		EventV1::CallListNew().emit();
	}

	// New design, proxy events.
	/// An `Evm` event proxied by the Moonbeam runtime to this host function.
	/// evm -> moonbeam_runtime -> host.
	fn evm_event(&mut self, event: Vec<u8>) {
		if let Ok(event) = EvmEvent::decode(&mut &event[..]) {
			EventV2::Evm(event).emit();
		} else {
			tracing::warn!("Failed to decode EvmEvent from bytes : {:?}", event);
		}
	}
	/// A `Gasometer` event proxied by the Moonbeam runtime to this host function.
	/// evm_gasometer -> moonbeam_runtime -> host.
	fn gasometer_event(&mut self, event: Vec<u8>) {
		if let Ok(event) = GasometerEvent::decode(&mut &event[..]) {
			EventV2::Gasometer(event).emit();
		} else {
			tracing::warn!("Failed to decode GasometerEvent from bytes : {:?}", event);
		}
	}
	/// A `Runtime` event proxied by the Moonbeam runtime to this host function.
	/// evm_runtime -> moonbeam_runtime -> host.
	fn runtime_event(&mut self, event: Vec<u8>) {
		if let Ok(event) = RuntimeEvent::decode(&mut &event[..]) {
			EventV2::Runtime(event).emit();
		} else {
			tracing::warn!("Failed to decode RuntimeEvent from bytes : {:?}", event);
		}
	}
	/// An event to create a new CallList (currently a new transaction when tracing a block).
	#[version(2)]
	fn call_list_new(&mut self) {
		EventV2::CallListNew().emit();
	}
}
