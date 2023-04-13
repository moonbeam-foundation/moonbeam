// Copyright 2019-2023 PureStake Inc.
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

//! Provides utilities for compatibility with Solidity tooling.

pub mod codec;
pub mod modifier;
pub mod revert;

pub use codec::{
	decode_arguments, decode_event_data, decode_return_value, encode_arguments, encode_event_data,
	encode_return_value, encode_with_selector, Codec,
};
