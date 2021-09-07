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

pub mod blockscout;
pub mod raw;
pub mod trace_filter;

use crate::proxy::v2::Listener;
#[cfg(feature = "std")]
use serde::Serialize;

#[cfg(feature = "std")]
pub trait TraceResponseBuilder {
	type Listener: Listener;
	type Response: Serialize;

	fn build(listener: Self::Listener) -> Option<Self::Response>;
}
