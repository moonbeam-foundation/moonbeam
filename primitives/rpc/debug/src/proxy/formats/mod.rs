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

use codec::{Decode, Encode};
use ethereum_types::U256;
use sp_std::vec::Vec;

pub mod blockscout;
pub mod call_tracer;
pub mod raw;
pub mod trace_filter;

pub use blockscout::BlockscoutCall;
pub use call_tracer::CallTracerCall;

use crate::proxy::v2::Listener;
#[cfg(feature = "std")]
use serde::Serialize;

#[cfg(feature = "std")]
pub trait TraceResponseBuilder {
	type Listener: Listener;
	type Response: Serialize;

	fn build(listener: Self::Listener) -> Option<Self::Response>;
}

/// Single transaction trace.
#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Serialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase", untagged))]
pub enum TransactionTrace {
	/// Classical output of `debug_trace`.
	#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
	Raw {
		gas: U256,
		#[cfg_attr(feature = "std", serde(with = "hex"))]
		return_value: Vec<u8>,
		step_logs: Vec<crate::single::RawStepLog>,
	},
	/// Matches the formatter used by Blockscout.
	/// Is also used to built output of OpenEthereum's `trace_filter`.
	CallList(Vec<Call>),
	/// Used by Geth's callTracer.
	CallListNested(Call),
}

#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Serialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase", untagged))]
pub enum Call {
	Blockscout(BlockscoutCall),
	CallTracer(CallTracerCall),
}
