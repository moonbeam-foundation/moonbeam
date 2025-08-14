// Copyright 2019-2025 PureStake Inc.
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

use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};

#[derive(
	Clone, Copy, Debug, Default, Deserialize, Serialize, Encode, Decode, TypeInfo, PartialEq, Eq,
)]
pub struct RelayChainIndices {
	// Pallet indices
	pub staking: u8,
	pub utility: u8,
	pub hrmp: u8,
	// Staking indices
	pub bond: u8,
	pub bond_extra: u8,
	pub unbond: u8,
	pub withdraw_unbonded: u8,
	pub validate: u8,
	pub nominate: u8,
	pub chill: u8,
	pub set_payee: u8,
	pub set_controller: u8,
	pub rebond: u8,
	// Utility indices
	pub as_derivative: u8,
	// Hrmp indices
	pub init_open_channel: u8,
	pub accept_open_channel: u8,
	pub close_channel: u8,
	pub cancel_open_request: u8,
}
