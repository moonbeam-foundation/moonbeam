// Copyright 2025 Moonbeam foundation
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

use frame_support::weights::Weight;
use sp_runtime::DispatchError;
use sp_std::cell::RefCell;
use sp_std::collections::btree_map::BTreeMap;
use xcm::latest::Location;
use xcm_primitives::XcmFeeTrader;

/// Must match `pallet-xcm-weight-trader`'s `RELATIVE_PRICE_DECIMALS`.
pub const RELATIVE_PRICE_DECIMALS: u32 = 18;

thread_local! {
	/// Thread-local storage for relative prices keyed by asset location.
	///
	/// Semantics match `pallet-xcm-weight-trader`:
	/// - Price is relative to the chain native asset.
	/// - It uses 18 decimals of precision.
	/// - Larger values mean the asset is *more valuable* and therefore requires *less* of it
	///   to pay for the same amount of weight.
	static RELATIVE_PRICE: RefCell<BTreeMap<Location, u128>> = RefCell::new(BTreeMap::new());
}

/// Memory-based fee trader for tests that stores relative prices in memory.
///
/// This is intentionally aligned with `pallet-xcm-weight-trader`'s behavior to make tests and
/// benchmarks share the same pricing interpretation:
/// - `set_asset_price(location, value)` sets a **relative price** (18 decimals), not a fee-per-second.
/// - `compute_fee(weight, asset)` converts `weight` into a "native fee amount" and then converts that
///   native amount into the target asset using the stored relative price:
///
/// \[
/// \text{asset\_amount} = \left\lceil \frac{\text{native\_amount} \cdot 10^{18}}{\text{relative\_price}} \right\rceil
/// \]
///
/// For simplicity (and determinism), the "native amount" is derived from `weight.ref_time()`.
pub struct MemoryFeeTrader;

impl XcmFeeTrader for MemoryFeeTrader {
	fn compute_fee(weight: Weight, asset_location: &Location) -> Result<u128, DispatchError> {
		let relative_price = RELATIVE_PRICE
			.with(|map| map.borrow().get(asset_location).copied())
			.ok_or(DispatchError::Other("Asset relative price not set"))?;

		// Stand-in for the runtime's native `WeightToFee`: use ref_time directly.
		let native_amount: u128 = weight.ref_time() as u128;

		let scale: u128 = 10u128.pow(RELATIVE_PRICE_DECIMALS);
		let numerator = native_amount
			.checked_mul(scale)
			.ok_or(DispatchError::Other("Overflow computing fee"))?;

		// Round up (match weight-trader behavior).
		let amount = numerator
			.checked_add(relative_price.saturating_sub(1))
			.ok_or(DispatchError::Other("Overflow computing fee"))?
			.checked_div(relative_price)
			.ok_or(DispatchError::Other("Division by zero"))?;

		Ok(amount)
	}

	fn get_asset_price(asset_location: &Location) -> Option<u128> {
		RELATIVE_PRICE.with(|map| map.borrow().get(asset_location).copied())
	}

	fn set_asset_price(asset_location: Location, value: u128) -> Result<(), DispatchError> {
		if value == 0 {
			return Err(DispatchError::Other("Relative price cannot be zero"));
		}

		RELATIVE_PRICE.with(|map| map.borrow_mut().insert(asset_location, value));
		Ok(())
	}

	fn remove_asset(asset_location: Location) -> Result<(), DispatchError> {
		RELATIVE_PRICE.with(|map| map.borrow_mut().remove(&asset_location));
		Ok(())
	}
}
