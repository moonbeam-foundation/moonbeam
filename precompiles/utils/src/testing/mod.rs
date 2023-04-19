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

pub mod account;
pub mod execution;
pub mod handle;
pub mod modifier;
mod solidity;

pub use {
	account::*,
	execution::*,
	handle::*,
	modifier::*,
	solidity::{check_precompile_implements_solidity_interfaces, compute_selector},
};

use fp_evm::Log;

pub fn decode_revert_message(encoded: &[u8]) -> &[u8] {
	let encoded_len = encoded.len();
	// selector 4 + offset 32 + string length 32
	if encoded_len > 68 {
		let message_len = encoded[36..68].iter().sum::<u8>();
		if encoded_len >= 68 + message_len as usize {
			return &encoded[68..68 + message_len as usize];
		}
	}
	b"decode_revert_message: error"
}

#[derive(Clone, PartialEq, Eq)]
pub struct PrettyLog(Log);

impl core::fmt::Debug for PrettyLog {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
		let bytes = self
			.0
			.data
			.iter()
			.map(|b| format!("{:02X}", b))
			.collect::<Vec<String>>()
			.join("");

		let message = String::from_utf8(self.0.data.clone()).ok();

		f.debug_struct("Log")
			.field("address", &self.0.address)
			.field("topics", &self.0.topics)
			.field("data", &bytes)
			.field("data_utf8", &message)
			.finish()
	}
}

/// Panics if an event is not found in the system log of events
#[macro_export]
macro_rules! assert_event_emitted {
	($event:expr) => {
		match &$event {
			e => {
				assert!(
					crate::mock::events().iter().find(|x| *x == e).is_some(),
					"Event {:?} was not found in events: \n {:?}",
					e,
					crate::mock::events()
				);
			}
		}
	};
}

// Panics if an event is found in the system log of events
#[macro_export]
macro_rules! assert_event_not_emitted {
	($event:expr) => {
		match &$event {
			e => {
				assert!(
					crate::mock::events().iter().find(|x| *x == e).is_none(),
					"Event {:?} was found in events: \n {:?}",
					e,
					crate::mock::events()
				);
			}
		}
	};
}
