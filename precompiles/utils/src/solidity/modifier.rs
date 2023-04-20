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

//! Provide checks related to function modifiers (view/payable).

use {
	crate::solidity::revert::{MayRevert, RevertReason},
	fp_evm::Context,
	sp_core::U256,
};

/// Represents modifiers a Solidity function can be annotated with.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum FunctionModifier {
	/// Function that doesn't modify the state.
	View,
	/// Function that modifies the state but refuse receiving funds.
	/// Correspond to a Solidity function with no modifiers.
	NonPayable,
	/// Function that modifies the state and accept funds.
	Payable,
}

#[must_use]
/// Check that a function call is compatible with the context it is
/// called into.
pub fn check_function_modifier(
	context: &Context,
	is_static: bool,
	modifier: FunctionModifier,
) -> MayRevert {
	if is_static && modifier != FunctionModifier::View {
		return Err(
			RevertReason::custom("Can't call non-static function in static context").into(),
		);
	}

	if modifier != FunctionModifier::Payable && context.apparent_value > U256::zero() {
		return Err(RevertReason::custom("Function is not payable").into());
	}

	Ok(())
}
