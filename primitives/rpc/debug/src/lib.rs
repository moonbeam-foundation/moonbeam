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

#![cfg_attr(not(feature = "std"), no_std)]
// These functions are quite usefull, shoud it be moved into its own crate ?
#[cfg(feature = "std")]
pub mod serialization;

pub mod api;

pub mod v1;
pub mod v2;

/// Runtime api closure result.
#[derive(Debug)]
pub enum Response {
	V1(v1::Response),
	V2(v2::Response),
}

impl From<v1::Response> for Response {
	fn from(source: v1::Response) -> Self {
		Self::V1(source)
	}
}

impl From<v2::Response> for Response {
	fn from(source: v2::Response) -> Self {
		Self::V2(source)
	}
}
