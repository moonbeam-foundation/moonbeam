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

//! Unit testing
use crate::*;
use frame_support::{assert_noop, assert_ok, PalletId};
use mock::*;
use sp_runtime::traits::AccountIdConversion;
const main_account: PalletId = PalletId(*b"pc/lqstk");
use substrate_fixed::types::U64F64;

#[test]
fn set_ratio_works() {
	ExtBuilder::default()
		.with_balances(vec![(1, 1000)])
		.build()
		.execute_with(|| {
			// Insert contributors
			roll_to(4);
			// 1000 in supply now, nothing staked, error
			assert_noop!(
				LiquidStaking::set_ratio(Origin::root(), 1000u32.into()),
				Error::<Test>::NothingStakedToSetRatio
			);
			assert_ok!(LiquidStaking::stake_dot(
				Origin::signed(1),
				500u32.into(),
				0
			));

			assert_eq!(Balances::free_balance(&1), 500);
			assert_eq!(Balances::free_balance(&main_account.into_account()), 500);

			// Let's say now we have 1500 tokens in the sovereign account. The ratio we should get is 2:!
			assert_ok!(LiquidStaking::set_ratio(Origin::root(), 1500u32.into()));

			println!("{:?}", LiquidStaking::staked_map(&1));

			assert_eq!(LiquidStaking::current_ratio(), U64F64::from_num(2));
			// If I unstake now I shouls get double the money
			assert_ok!(LiquidStaking::unstake_dot(Origin::signed(1), 500u32.into(),));
			println!("{:?}", LiquidStaking::staked_map(&1));

			assert_noop!(
				LiquidStaking::unstake_dot(Origin::signed(1), 500u32.into(),),
				Error::<Test>::NoRewardsAvailable
			);
			assert_eq!(Balances::free_balance(&1), 1500);
		});
}

#[test]
fn stake_at_different_prices_works() {
	ExtBuilder::default()
		.with_balances(vec![(1, 1000)])
		.build()
		.execute_with(|| {
			// Insert contributors
			roll_to(4);
			assert_ok!(LiquidStaking::stake_dot(
				Origin::signed(1),
				500u32.into(),
				0
			));

			assert_eq!(Balances::free_balance(&1), 500);
			assert_eq!(Balances::free_balance(&main_account.into_account()), 500);

			// Let's say now we have 1500 tokens in the sovereign account. The ratio we should get is 2:!
			assert_ok!(LiquidStaking::set_ratio(Origin::root(), 1500u32.into()));

			assert_ok!(LiquidStaking::stake_dot(
				Origin::signed(1),
				300u32.into(),
				0
			));

			println!("{:?}", LiquidStaking::staked_map(&1));

			// Let's say now we have 2150 tokens in the sovereign account. The ratio we should get is 3:!
			assert_ok!(LiquidStaking::set_ratio(Origin::root(), 2150u32.into()));

			assert_eq!(LiquidStaking::current_ratio(), U64F64::from_num(3));
			// If I unstake now I shouls get double the money
			assert_ok!(LiquidStaking::unstake_dot(Origin::signed(1), 500u32.into(),));
			assert_eq!(Balances::free_balance(&1), 1418);

			assert_ok!(LiquidStaking::unstake_dot(Origin::signed(1), 300u32.into(),));

			assert_noop!(
				LiquidStaking::unstake_dot(Origin::signed(1), 800u32.into(),),
				Error::<Test>::NoRewardsAvailable
			);
			assert_eq!(Balances::free_balance(&1), 2150);
		});
}
#[test]
fn stake_price_drop_works() {
	ExtBuilder::default()
		.with_balances(vec![(1, 1000)])
		.build()
		.execute_with(|| {
			// Insert contributors
			roll_to(4);
			assert_ok!(LiquidStaking::stake_dot(
				Origin::signed(1),
				500u32.into(),
				0
			));

			assert_eq!(Balances::free_balance(&1), 500);
			assert_eq!(Balances::free_balance(&main_account.into_account()), 500);

			// Let's say now we have 1500 tokens in the sovereign account. The ratio we should get is 2:!
			assert_ok!(LiquidStaking::set_ratio(Origin::root(), 1500u32.into()));

			assert_ok!(LiquidStaking::stake_dot(
				Origin::signed(1),
				300u32.into(),
				0
			));

			println!("{:?}", LiquidStaking::staked_map(&1));
			// Let's say now we have 600 tokens in the sovereign account. The ratio we should get is 1:2!
			assert_ok!(LiquidStaking::set_ratio(Origin::root(), 525u32.into()));

			assert_eq!(LiquidStaking::current_ratio(), U64F64::from_num(0.5));
			// If I unstake now I shouls get half the money
			assert_ok!(LiquidStaking::unstake_dot(Origin::signed(1), 500u32.into(),));
			assert_eq!(Balances::free_balance(&1), 403);

			assert_ok!(LiquidStaking::unstake_dot(Origin::signed(1), 300u32.into(),));

			assert_noop!(
				LiquidStaking::unstake_dot(Origin::signed(1), 800u32.into(),),
				Error::<Test>::NoRewardsAvailable
			);
			assert_eq!(Balances::free_balance(&1), 525);
			assert_eq!(Balances::free_balance(&main_account.into_account()), 0);
		});
}
