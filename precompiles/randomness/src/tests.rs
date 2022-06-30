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

//! Randomness precompile unit tests
use crate::Action;
// use crate::mock::*;

#[test]
fn selectors() {
	assert_eq!(Action::RequestBabeRandomnessCurrentBlock as u32, 0xc92142bc);
	assert_eq!(Action::RequestBabeRandomnessOneEpochAgo as u32, 0x73257347);
	assert_eq!(Action::RequestBabeRandomnessTwoEpochsAgo as u32, 0x8ef48c72);
	assert_eq!(Action::RequestLocalRandomness as u32, 0x3dbc0d19);
	assert_eq!(Action::FulfillRequest as u32, 0xb5983332);
	assert_eq!(Action::IncreaseRequestFee as u32, 0xf35d8354);
	assert_eq!(Action::ExecuteRequestExpiration as u32, 0x536b9ef1);
	assert_eq!(Action::InstantBabeRandomnessCurrentBlock as u32, 0xb0ea3938);
	assert_eq!(Action::InstantBabeRandomnessOneEpochAgo as u32, 0x0cd3aa4a);
	assert_eq!(Action::InstantBabeRandomnessTwoEpochsAgo as u32, 0xbc88ee5f);
	assert_eq!(Action::InstantLocalRandomness as u32, 0xf71c715f);
}
