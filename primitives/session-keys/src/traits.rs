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

/// A Trait to lookup keys from AuthorIds
pub trait KeysLookup<AuthorId, Keys> {
	fn lookup_keys(author: &AuthorId) -> Option<Keys>;
}

// A dummy impl used in simple tests
impl<AuthorId, Keys> KeysLookup<AuthorId, Keys> for () {
	fn lookup_keys(_: &AuthorId) -> Option<Keys> {
		None
	}
}

/// Exposes randomness in pallet-vrf to the runtime
pub trait MaybeGetRandomness<R> {
	fn maybe_get_randomness() -> Option<R>;
}

/// To read and set data from relay chain from runtime into pallet-{randomness, vrf}
pub trait SetRelayData {
	fn set_relay_data();
}

impl SetRelayData for () {
	fn set_relay_data() {}
}
