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

use {
	crate::{
		assert_eq_events,
		mock::{Event as RuntimeEvent, ExtBuilder, LiquidStaking, Origin},
		pallet::{Event, SharesOrStake},
	},
	frame_support::{assert_noop, assert_ok},
};

#[test]
fn stake_auto_compounding() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(LiquidStaking::stake_auto_compounding(
			Origin::signed(2),
			1,
			SharesOrStake::Shares(1)
		));
		assert_eq_events!(vec![
			Event::StakedAutoCompounding {
				candidate: 1,
				delegator: 2,
				shares: 1,
				stake: 1_000_000_000, // InitialManualClaimShareValue
			},
			Event::UpdatedCandidatePosition {
				candidate: 1,
				stake: 1_000_000_000, // InitialManualClaimShareValue
				self_delegation: 0,
				before: None,
				after: None,
			},
			Event::IncreasedStake {
				candidate: 1,
				stake: 1_000_000_000 // InitialManualClaimShareValue
			}
		]);
	});
}
