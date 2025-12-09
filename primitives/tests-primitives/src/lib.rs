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

//! Test primitives for Moonbeam, including mock implementations for testing.

use frame_support::weights::{constants::WEIGHT_REF_TIME_PER_SECOND, Weight};
use sp_runtime::DispatchError;
use sp_std::cell::RefCell;
use sp_std::collections::btree_map::BTreeMap;
use xcm::latest::Location;
use xcm_primitives::XcmFeeTrader;

thread_local! {
	/// Thread-local storage for fee-per-second values keyed by asset location.
	static FEE_PER_SECOND: RefCell<BTreeMap<Location, u128>> = RefCell::new(BTreeMap::new());
}

/// Memory-based fee trader for tests that stores fee-per-second values in memory.
///
/// This implementation calculates fees based on weight and a fee-per-second rate,
/// similar to how the real weight-trader pallet works. It's useful for testing
/// XCM fee payment scenarios without requiring a full runtime setup.
pub struct MemoryFeeTrader;

impl XcmFeeTrader for MemoryFeeTrader {
	fn compute_fee(
		weight: Weight,
		asset_location: &Location,
		explicit_amount: Option<u128>,
	) -> Result<u128, DispatchError> {
		// If explicit amount is provided, use it directly
		if let Some(amount) = explicit_amount {
			return Ok(amount);
		}

		// Get fee-per-second from storage
		let fee_per_second = FEE_PER_SECOND
			.with(|map| map.borrow().get(asset_location).copied())
			.ok_or(DispatchError::Other("Fee per second not set"))?;

		// Calculate fee using the same formula as the weight-trader pallet
		// fee = (fee_per_second * weight.ref_time() + weight_per_second - 1) / weight_per_second
		let weight_per_second_u128 = WEIGHT_REF_TIME_PER_SECOND as u128;
		let fee_mul_rounded_up = (fee_per_second.saturating_mul(weight.ref_time() as u128))
			.saturating_add(weight_per_second_u128 - 1);
		Ok(fee_mul_rounded_up / weight_per_second_u128)
	}

	fn get_asset_price(asset_location: &Location) -> Option<u128> {
		FEE_PER_SECOND.with(|map| map.borrow().get(asset_location).copied())
	}

	fn set_asset_price(asset_location: Location, value: u128) -> Result<(), DispatchError> {
		FEE_PER_SECOND.with(|map| {
			map.borrow_mut().insert(asset_location, value);
		});
		Ok(())
	}

	fn remove_asset(asset_location: Location) -> Result<(), DispatchError> {
		FEE_PER_SECOND.with(|map| {
			map.borrow_mut().remove(&asset_location);
		});
		Ok(())
	}
}
