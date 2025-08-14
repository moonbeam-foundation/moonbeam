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

use crate::mock::*;
use fp_evm::{ExitRevert, PrecompileFailure};
use precompile_utils::{solidity::revert::revert_as_bytes, testing::*};

fn precompiles() -> Precompiles<Runtime> {
	PrecompilesValue::get()
}

#[test]
fn contract_disabling_default_value_is_false() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 100_000)])
		.build()
		.execute_with(|| {
			// default should be false
			assert_eq!(crate::storage::PrecompileEnabled::get(), None);
			assert_eq!(crate::is_enabled(), false);
			assert_eq!(
				crate::ensure_enabled(),
				Err(PrecompileFailure::Revert {
					exit_status: ExitRevert::Reverted,
					output: revert_as_bytes("GMP Precompile is not enabled"),
				})
			);

			precompiles()
				.prepare_test(
					CryptoAlith,
					Precompile1,
					PCall::wormhole_transfer_erc20 {
						wormhole_vaa: Vec::new().into(),
					},
				)
				.execute_reverts(|output| output == b"GMP Precompile is not enabled");
		})
}

#[test]
fn contract_enabling_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 100_000)])
		.build()
		.execute_with(|| {
			crate::storage::PrecompileEnabled::set(Some(true));
			assert_eq!(crate::storage::PrecompileEnabled::get(), Some(true));
			assert_eq!(crate::is_enabled(), true);
			assert_eq!(crate::ensure_enabled(), Ok(()));

			// should fail at a later point since contract addresses are not set
			precompiles()
				.prepare_test(
					CryptoAlith,
					Precompile1,
					PCall::wormhole_transfer_erc20 {
						wormhole_vaa: Vec::new().into(),
					},
				)
				.execute_reverts(|output| output == b"invalid wormhole core address");
		})
}

#[test]
fn contract_disabling_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 100_000)])
		.build()
		.execute_with(|| {
			crate::storage::PrecompileEnabled::set(Some(false));
			assert_eq!(crate::storage::PrecompileEnabled::get(), Some(false));
			assert_eq!(crate::is_enabled(), false);
			assert_eq!(
				crate::ensure_enabled(),
				Err(PrecompileFailure::Revert {
					exit_status: ExitRevert::Reverted,
					output: revert_as_bytes("GMP Precompile is not enabled"),
				})
			);

			precompiles()
				.prepare_test(
					CryptoAlith,
					Precompile1,
					PCall::wormhole_transfer_erc20 {
						wormhole_vaa: Vec::new().into(),
					},
				)
				.execute_reverts(|output| output == b"GMP Precompile is not enabled");
		})
}

#[test]
fn test_solidity_interface_has_all_function_selectors_documented_and_implemented() {
	check_precompile_implements_solidity_interfaces(&["Gmp.sol"], PCall::supports_selector)
}
