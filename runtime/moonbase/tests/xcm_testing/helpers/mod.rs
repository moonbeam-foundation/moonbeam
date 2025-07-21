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

//! Helper modules for XCM tests

pub mod assertions;
pub mod assets;
pub mod setup;

// Re-export only the functions that are actually used
pub use assertions::{
	assert_asset_balance, assert_asset_balance_para_b, assert_native_balance_decreased_by,
	assert_treasury_asset_balance,
};
pub use assets::{
	register_relay_asset, register_relay_asset_in_para_b, register_relay_asset_non_sufficient,
	register_relay_asset_with_units_per_second, setup_relay_asset_for_statemint,
};
pub use setup::{
	account_key20_location, encode_relay_balance_transfer_call, execute_transfer_to_para,
	fund_account_native, medium_transfer_weight, parachain_location, reset_test_environment,
	setup_relay_transactor_config, standard_heavy_weight, standard_transfer_weight,
};
