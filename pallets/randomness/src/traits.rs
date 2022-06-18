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

/// Read babe randomness info from the relay chain state proof
pub trait GetBabeData<BlockNumber, EpochIndex, Randomness> {
	fn get_relay_block_number() -> BlockNumber;
	fn get_relay_epoch_index() -> EpochIndex;
	fn get_current_block_randomness() -> Randomness;
	fn get_one_epoch_ago_randomness() -> Randomness;
	fn get_two_epochs_ago_randomness() -> Randomness;
}

/// Read VRF input from the relay chain state proof
pub trait GetVrfInput<Input> {
	fn get_vrf_input() -> Input;
}
