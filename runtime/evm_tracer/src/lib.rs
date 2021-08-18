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

//! Substrate EVM tracing.
//!
//! The purpose of this crate is enable tracing the EVM opcode execution and will be used by
//! both Dapp developers - to get a granular view on their transactions - and indexers to access
//! the EVM callstack (internal transactions).
//!
//! Proxies EVM messages to the host functions.

#![cfg_attr(not(feature = "std"), no_std)]

use crate::util::*;

mod call_list;
mod util;

pub use call_list::CallListTracer;
use codec::Encode;
pub use util::{EvmListener, GasometerListener, RuntimeListener};

use moonbeam_rpc_primitives_debug::proxy::types::{EvmEvent, GasometerEvent, RuntimeEvent};

pub struct EvmTracer;

impl EvmTracer {
	pub fn new() -> Self {
		Self
	}
	/// Setup event listeners and execute provided closure.
	///
	/// Consume the tracer and return it alongside the return value of
	/// the closure.
	pub fn trace<R, F: FnOnce() -> R>(self, f: F) {
		let wrapped = Rc::new(RefCell::new(self));

		let mut gasometer = ListenerProxy(Rc::clone(&wrapped));
		let mut runtime = ListenerProxy(Rc::clone(&wrapped));
		let mut evm = ListenerProxy(Rc::clone(&wrapped));

		// Each line wraps the previous `f` into a `using` call.
		// Listening to new events results in adding one new line.
		// Order is irrelevant when registering listeners.
		let f = || runtime_using(&mut runtime, f);
		let f = || gasometer_using(&mut gasometer, f);
		let f = || evm_using(&mut evm, f);
		f();
	}
}

impl EvmListener for EvmTracer {
	fn event(&mut self, event: evm::tracing::Event) {
		let event: EvmEvent = event.into();
		let message = event.encode();
		moonbeam_primitives_ext::moonbeam_ext::evm_event(message);
	}
}

impl GasometerListener for EvmTracer {
	fn event(&mut self, event: evm_gasometer::tracing::Event) {
		let event: GasometerEvent = event.into();
		let message = event.encode();
		moonbeam_primitives_ext::moonbeam_ext::gasometer_event(message);
	}
}

impl RuntimeListener for EvmTracer {
	fn event(&mut self, event: evm_runtime::tracing::Event) {
		let event: RuntimeEvent = event.into();
		let message = event.encode();
		moonbeam_primitives_ext::moonbeam_ext::runtime_event(message);
	}
}
