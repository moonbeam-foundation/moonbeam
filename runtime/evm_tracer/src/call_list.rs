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

use crate::util::*;
use moonbeam_rpc_primitives_debug::single::{Call, TransactionTrace};
use sp_std::collections::btree_map::BTreeMap;

#[derive(Debug)]
pub struct CallListTracer {
	entries: BTreeMap<u32, Call>,
}

impl CallListTracer {
	pub fn new() -> Self {
		Self {
			entries: BTreeMap::new(),
		}
	}

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

	pub fn into_tx_trace(self) -> TransactionTrace {
		todo!()
	}
}

impl GasometerListener for CallListTracer {
	fn event(&mut self, event: GasometerEvent) {
		todo!()
	}
}

impl RuntimeListener for CallListTracer {
	fn event(&mut self, event: RuntimeEvent) {
		todo!()
	}
}

impl EvmListener for CallListTracer {
	fn event(&mut self, event: EvmEvent) {
		todo!()
	}
}
