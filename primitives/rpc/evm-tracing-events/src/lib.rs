// Copyright 2019-2022 PureStake Inc.
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

use ethereum_types::{H160, U256};
use parity_scale_codec::{Decode, Encode};
use sp_runtime_interface::pass_by::PassByCodec;

environmental::environmental!(listener: dyn Listener + 'static);

pub fn using<R, F: FnOnce() -> R>(l: &mut (dyn Listener + 'static), f: F) -> R {
	listener::using(l, f)
}

/// Allow to configure which data of the Step event
/// we want to keep or discard. Not discarding the data requires cloning the data
/// in the runtime which have a significant cost for each step.
#[derive(Clone, Copy, Eq, PartialEq, Debug, Encode, Decode, Default, PassByCodec)]
pub struct StepEventFilter {
	pub enable_stack: bool,
	pub enable_memory: bool,
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
/// - Inside the runtime to proxy the events through the host functions
/// - Inside the client to forward those events to the client listener.
pub trait Listener {
	fn event(&mut self, event: Event);

	/// Allow the runtime to know which data should be discarded and not cloned.
	/// WARNING: It is only called once when the runtime tracing is instantiated to avoid
	/// performing many ext calls.
	fn step_event_filter(&self) -> StepEventFilter;
}

pub fn step_event_filter() -> Option<StepEventFilter> {
	let mut filter = None;
	listener::with(|listener| filter = Some(listener.step_event_filter()));
	filter
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
