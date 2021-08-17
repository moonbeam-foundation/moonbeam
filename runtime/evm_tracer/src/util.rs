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

pub use evm::tracing::{using as evm_using, Event as EvmEvent, EventListener as EvmListener};
pub use evm::Opcode;
pub use evm_gasometer::tracing::{
	using as gasometer_using, Event as GasometerEvent, EventListener as GasometerListener,
};
pub use evm_runtime::tracing::{
	using as runtime_using, Event as RuntimeEvent, EventListener as RuntimeListener,
};
use moonbeam_rpc_primitives_debug::CallType;
pub use sp_std::{cell::RefCell, fmt::Debug, rc::Rc, vec, vec::Vec};
pub struct ListenerProxy<T>(pub Rc<RefCell<T>>);

impl<T: GasometerListener> GasometerListener for ListenerProxy<T> {
	fn event(&mut self, event: GasometerEvent) {
		self.0.borrow_mut().event(event);
	}
}

impl<T: RuntimeListener> RuntimeListener for ListenerProxy<T> {
	fn event(&mut self, event: RuntimeEvent) {
		self.0.borrow_mut().event(event);
	}
}

impl<T: EvmListener> EvmListener for ListenerProxy<T> {
	fn event(&mut self, event: EvmEvent) {
		self.0.borrow_mut().event(event);
	}
}

#[derive(Debug)]
pub enum ContextType {
	Call(CallType),
	Create,
}

impl ContextType {
	pub fn from(opcode: Opcode) -> Option<Self> {
		match opcode.0 {
			0xF0 | 0xF5 => Some(ContextType::Create),
			0xF1 => Some(ContextType::Call(CallType::Call)),
			0xF2 => Some(ContextType::Call(CallType::CallCode)),
			0xF4 => Some(ContextType::Call(CallType::DelegateCall)),
			0xFA => Some(ContextType::Call(CallType::StaticCall)),
			_ => None,
		}
	}
}
