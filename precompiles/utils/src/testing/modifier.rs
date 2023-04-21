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
		solidity::codec::Writer,
		testing::{decode_revert_message, MockHandle},
	},
	fp_evm::{Context, PrecompileFailure, PrecompileSet},
	sp_core::{H160, U256},
};

pub struct PrecompilesModifierTester<P> {
	precompiles: P,
	handle: MockHandle,
}

impl<P: PrecompileSet> PrecompilesModifierTester<P> {
	pub fn new(precompiles: P, from: impl Into<H160>, to: impl Into<H160>) -> Self {
		let to = to.into();
		let mut handle = MockHandle::new(
			to.clone(),
			Context {
				address: to,
				caller: from.into(),
				apparent_value: U256::zero(),
			},
		);

		handle.gas_limit = u64::MAX;

		Self {
			precompiles,
			handle,
		}
	}

	fn is_view(&mut self, selector: u32) -> bool {
		// View: calling with static should not revert with static-related message.
		let handle = &mut self.handle;
		handle.is_static = true;
		handle.context.apparent_value = U256::zero();
		handle.input = Writer::new_with_selector(selector).build();

		let res = self.precompiles.execute(handle);

		match res {
			Some(Err(PrecompileFailure::Revert { output, .. })) => {
				let decoded = decode_revert_message(&output);

				dbg!(decoded) != b"Can't call non-static function in static context"
			}
			Some(_) => true,
			None => panic!("tried to check view modifier on unknown precompile"),
		}
	}

	fn is_payable(&mut self, selector: u32) -> bool {
		// Payable: calling with value should not revert with payable-related message.
		let handle = &mut self.handle;
		handle.is_static = false;
		handle.context.apparent_value = U256::one();
		handle.input = Writer::new_with_selector(selector).build();

		let res = self.precompiles.execute(handle);

		match res {
			Some(Err(PrecompileFailure::Revert { output, .. })) => {
				let decoded = decode_revert_message(&output);

				decoded != b"Function is not payable"
			}
			Some(_) => true,
			None => panic!("tried to check payable modifier on unknown precompile"),
		}
	}

	pub fn test_view_modifier(&mut self, selectors: &[u32]) {
		for &s in selectors {
			assert!(
				self.is_view(s),
				"Function doesn't behave like a view function."
			);
			assert!(
				!self.is_payable(s),
				"Function doesn't behave like a non-payable function."
			)
		}
	}

	pub fn test_payable_modifier(&mut self, selectors: &[u32]) {
		for &s in selectors {
			assert!(
				!self.is_view(s),
				"Function doesn't behave like a non-view function."
			);
			assert!(
				self.is_payable(s),
				"Function doesn't behave like a payable function."
			);
		}
	}

	pub fn test_default_modifier(&mut self, selectors: &[u32]) {
		for &s in selectors {
			assert!(
				!self.is_view(s),
				"Function doesn't behave like a non-view function."
			);
			assert!(
				!self.is_payable(s),
				"Function doesn't behave like a non-payable function."
			);
		}
	}
}
