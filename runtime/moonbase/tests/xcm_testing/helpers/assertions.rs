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

//! Assertion helpers for XCM tests

use crate::xcm_mock::{parachain, Assets, ParaA, Treasury};
use xcm_simulator::TestExt;

// Balance assertion helpers - only keep the ones that are actually used

pub fn assert_asset_balance(account: &[u8; 20], asset_id: parachain::AssetId, expected: u128) {
	ParaA::execute_with(|| {
		let account_id = parachain::AccountId::from(*account);
		assert_eq!(Assets::balance(asset_id, &account_id), expected);
	});
}

pub fn assert_asset_balance_para_b(
	account: &[u8; 20],
	asset_id: parachain::AssetId,
	expected: u128,
) {
	use crate::xcm_mock::ParaB;
	ParaB::execute_with(|| {
		let account_id = parachain::AccountId::from(*account);
		assert_eq!(Assets::balance(asset_id, &account_id), expected);
	});
}

pub fn assert_treasury_asset_balance(asset_id: parachain::AssetId, expected: u128) {
	ParaA::execute_with(|| {
		assert_eq!(Assets::balance(asset_id, &Treasury::account_id()), expected);
	});
}

// Balance change assertion helpers - only keep the one that's used

pub fn assert_native_balance_decreased_by(
	account: &[u8; 20],
	initial_balance: u128,
	decrease: u128,
) {
	ParaA::execute_with(|| {
		let account_id = parachain::AccountId::from(*account);
		let current_balance = parachain::Balances::free_balance(&account_id);
		assert_eq!(current_balance, initial_balance - decrease);
	});
}
