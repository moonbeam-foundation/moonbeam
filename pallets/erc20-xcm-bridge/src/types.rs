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

use frame_support::BoundedVec;
use scale_info::TypeInfo;
use sp_core::{ConstU32, U256};
use sp_runtime::{
	codec::{Decode, Encode, MaxEncodedLen},
	RuntimeDebug,
};

pub const MAX_NAME_LEN: u32 = 50;
pub const MAX_SYMBOL_LEN: u32 = 50;

#[derive(Default, Decode, Encode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct Erc20AssetData {
	pub bridge_supply: U256,
	pub metadata: Option<Erc20MetaData>,
}

#[derive(Default, Decode, Encode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct Erc20MetaData {
	pub name: BoundedVec<u8, ConstU32<MAX_NAME_LEN>>,
	pub symbol: BoundedVec<u8, ConstU32<MAX_SYMBOL_LEN>>,
	pub decimals: u8,
}
