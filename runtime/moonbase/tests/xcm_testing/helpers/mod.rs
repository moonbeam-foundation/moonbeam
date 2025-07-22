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
pub mod core;
pub mod statemint;
pub mod transactor;
pub mod transfers;
pub mod weights;

// Re-export functions from specific modules

pub use assertions::{
	assert_asset_balance, assert_asset_balance_para_b, assert_native_balance_decreased_by,
	assert_treasury_asset_balance,
};
pub use assets::{
	register_relay_asset, register_relay_asset_in_para_b, register_relay_asset_non_sufficient,
	register_relay_asset_with_units_per_second, setup_relay_asset_for_statemint,
};

pub use core::{account_key20_location, fund_account_native, parachain_location};
pub use statemint::{
	create_statemint_asset_location, execute_statemint_to_para_dot_transfer,
	execute_statemint_to_para_transfer_with_balance_check, register_statemint_asset_on_para,
	setup_multi_asset_statemint_test, setup_statemint_asset, setup_statemint_test_environment,
};
pub use transactor::{encode_relay_balance_transfer_call, setup_relay_transactor_config};
pub use transfers::{execute_relay_to_statemint_transfer, execute_transfer_to_para};
pub use weights::{medium_transfer_weight, standard_heavy_weight, standard_transfer_weight};
