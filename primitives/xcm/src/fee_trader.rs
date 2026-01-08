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

//! Trait for computing XCM fees and managing asset pricing.

use frame_support::weights::Weight;
use sp_runtime::DispatchError;
use xcm::latest::Location;

/// Trait for computing XCM fees and managing asset pricing.
/// This allows pallets to delegate fee calculation to an external system
/// (e.g., pallet-xcm-weight-trader) instead of maintaining their own storage.
pub trait XcmFeeTrader {
	/// Compute the fee amount for a given weight and asset location.
	///
	/// The fee should be calculated based on the weight and asset pricing configured
	/// for the given asset location.
	fn compute_fee(weight: Weight, asset_location: &Location) -> Result<u128, DispatchError>;

	/// Get the current price/fee-per-second for an asset, if configured.
	/// Returns None if the asset is not configured.
	fn get_asset_price(asset_location: &Location) -> Option<u128>;

	/// Set the price/configuration for an asset.

	fn set_asset_price(_asset_location: Location, _value: u128) -> Result<(), DispatchError>;

	/// Remove the price/configuration for an asset.
	fn remove_asset(_asset_location: Location) -> Result<(), DispatchError>;
}
