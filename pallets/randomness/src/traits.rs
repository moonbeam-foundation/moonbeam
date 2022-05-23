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

use frame_support::pallet_prelude::*;

/// Get the epoch index
pub trait GetEpochIndex<Index> {
	fn get_epoch_index() -> (Index, Weight);
}

/// Get babe randomness to insert into runtime
pub trait GetRelayRandomness<R> {
	fn get_current_block_randomness() -> (Option<R>, Weight);
	fn get_one_epoch_ago_randomness() -> (Option<R>, Weight);
	fn get_two_epochs_ago_randomness() -> (Option<R>, Weight);
}
