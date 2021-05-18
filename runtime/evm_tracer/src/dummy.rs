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

use crate::util::*;

#[derive(Debug, Clone)]
pub struct DummyTracer;

impl DummyTracer {
	pub fn trace<R, F: FnOnce() -> R>(self, f: F) -> (Self, R) {
		let wrapped = Rc::new(RefCell::new(self));

		let result = {
			let mut gasometer = ListenerProxy(Rc::clone(&wrapped));
			let mut runtime = ListenerProxy(Rc::clone(&wrapped));
			let mut evm = ListenerProxy(Rc::clone(&wrapped));

			let f = || runtime_using(&mut runtime, f);
			let f = || gasometer_using(&mut gasometer, f);
			let f = || evm_using(&mut evm, f);
			f()
		};

		(Rc::try_unwrap(wrapped).unwrap().into_inner(), result)
	}
}

impl GasometerListener for DummyTracer {
	#[cfg(feature = "std")]
	fn event(&mut self, event: GasometerEvent) {
		tracing::trace!("event: {:?}", event);
	}

	#[cfg(not(feature = "std"))]
	fn event(&mut self, _event: GasometerEvent) {}
}

impl RuntimeListener for DummyTracer {
	#[cfg(feature = "std")]
	fn event(&mut self, event: RuntimeEvent) {
		match event {
			RuntimeEvent::Step { opcode, .. } => {
				tracing::trace!("event: Step( opcode: {:?}, ..)", opcode)
			}
			event => tracing::trace!("event: {:?}", event),
		}
	}

	#[cfg(not(feature = "std"))]
	fn event(&mut self, _event: RuntimeEvent) {}
}

impl EvmListener for DummyTracer {
	#[cfg(feature = "std")]
	fn event(&mut self, event: EvmEvent) {
		match event {
			EvmEvent::Call { code_address, .. } => {
				tracing::trace!("event: Call( code_address: {:?}, ..)", code_address)
			}
			EvmEvent::Create { caller, .. } => {
				tracing::trace!("event: Create( caller: {:?}, ..)", caller)
			}
			event => tracing::trace!("event: {:?}", event),
		}
	}

	#[cfg(not(feature = "std"))]
	fn event(&mut self, _event: EvmEvent) {}
}
