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

use crate::proxy::v2::call_list::Listener;
use crate::single::TransactionTrace;

pub struct Response;

#[cfg(feature = "std")]
impl super::TraceResponseBuilder for Response {
	type Listener = Listener;
	type Response = TransactionTrace;

	fn build(listener: Listener) -> Option<TransactionTrace> {
		if let Some(entry) = listener.entries.last() {
			return Some(TransactionTrace::CallList(
				entry.into_iter().map(|(_, value)| value.clone()).collect(),
			));
		}
		None
	}
}
