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

//! Helper methods for computing block issuance based on annual inflation
use crate::pallet::{BalanceOf, Config};
use frame_support::traits::Currency;
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::PerThing;
use sp_runtime::{Perbill, RuntimeDebug};
use substrate_fixed::transcendental::pow as floatpow;
use substrate_fixed::types::{I32F32, I64F64};

const SECONDS_PER_YEAR: u32 = 31557600;
const SECONDS_PER_BLOCK: u32 = 12;
const BLOCKS_PER_YEAR: u32 = SECONDS_PER_YEAR / SECONDS_PER_BLOCK;

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Eq, PartialEq, Clone, Copy, Encode, Decode, Default, RuntimeDebug, TypeInfo)]
pub struct Range<T> {
	pub min: T,
	pub ideal: T,
	pub max: T,
}

impl<T: Ord> Range<T> {
	pub fn is_valid(&self) -> bool {
		self.max >= self.ideal && self.ideal >= self.min
	}
}

impl<T: Ord + Copy> From<T> for Range<T> {
	fn from(other: T) -> Range<T> {
		Range {
			min: other,
			ideal: other,
			max: other,
		}
	}
}
/// Convert an annual inflation to a block inflation
/// block = 1 - (1+annual)^(1/blocks_per_year)
fn perbill_annual_to_perbill_block(annual: Range<Perbill>) -> Range<Perbill> {
	let exponent = I32F32::from_num(1) / I32F32::from_num(BLOCKS_PER_YEAR);
	let year_to_block = |annual: Perbill| -> Perbill {
		let x = I32F32::from_num(annual.deconstruct()) / I32F32::from_num(Perbill::ACCURACY);
		let y: I64F64 = floatpow(I32F32::from_num(1) + x, exponent)
			.expect("Cannot overflow since blocks_per_year < u32::MAX");
		Perbill::from_parts(
			((y - I64F64::from_num(1)) * I64F64::from_num(Perbill::ACCURACY))
				.ceil()
				.to_num::<u32>(),
		)
	};
	Range {
		min: year_to_block(annual.min),
		ideal: year_to_block(annual.ideal),
		max: year_to_block(annual.max),
	}
}
/// Convert annual inflation rate range to block inflation rate
pub fn annual_to_block<T: Config>(annual: Range<Perbill>) -> Range<Perbill> {
	perbill_annual_to_perbill_block(annual)
}

/// Compute block issuance range from block inflation range and current total issuance
pub fn block_issuance_range<T: Config>(block: Range<Perbill>) -> Range<BalanceOf<T>> {
	let circulating = T::Currency::total_issuance();
	Range {
		min: block.min * circulating,
		ideal: block.ideal * circulating,
		max: block.max * circulating,
	}
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Eq, PartialEq, Clone, Encode, Decode, Default, RuntimeDebug, TypeInfo)]
/// DEPRECATED
pub struct InflationInfo<Balance> {
	/// Staking expectations
	pub expect: Range<Balance>,
	/// Annual inflation range
	pub annual: Range<Perbill>,
	/// Round inflation range
	pub round: Range<Perbill>,
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Eq, PartialEq, Clone, Encode, Decode, Default, RuntimeDebug, TypeInfo)]
pub struct InflationInformation<Balance> {
	/// Staking expectations
	pub expect: Range<Balance>,
	/// Annual inflation range
	pub annual: Range<Perbill>,
	/// Block inflation range
	pub block: Range<Perbill>,
}

impl<Balance> InflationInformation<Balance> {
	pub fn new<T: Config>(
		annual: Range<Perbill>,
		expect: Range<Balance>,
	) -> InflationInfo<Balance> {
		InflationInfo {
			expect,
			annual,
			round: annual_to_block::<T>(annual),
		}
	}
	/// Set block inflation range according to input annual inflation range
	pub fn set_block_from_annual<T: Config>(&mut self, new: Range<Perbill>) {
		self.block = annual_to_block::<T>(new);
	}
	/// Reset block inflation rate based on changes to blocks per year
	pub fn reset_block(&mut self) {
		self.block = perbill_annual_to_perbill_block(self.annual);
	}
	/// Set staking expectations
	pub fn set_expectations(&mut self, expect: Range<Balance>) {
		self.expect = expect;
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	fn mock_annual_to_block(annual: Range<Perbill>) -> Range<Perbill> {
		perbill_annual_to_perbill_block(annual)
	}
	fn mock_block_issuance_range(
		// Total circulating before minting
		circulating: u128,
		// Block inflation range
		block: Range<Perbill>,
	) -> Range<u128> {
		Range {
			min: block.min * circulating,
			ideal: block.ideal * circulating,
			max: block.max * circulating,
		}
	}
	#[test]
	fn simple_issuance_conversion() {
		// 5% inflation for 10_000_0000 = 500,000 minted over the year
		// let's assume there are 10 periods in a year
		// => mint 500_000 over 10 periods => 50_000 minted per period
		let expected_round_issuance_range: Range<u128> = Range {
			min: 48_909,
			ideal: 48_909,
			max: 48_909,
		};
		let schedule = Range {
			min: Perbill::from_percent(5),
			ideal: Perbill::from_percent(5),
			max: Perbill::from_percent(5),
		};
		assert_eq!(
			expected_round_issuance_range,
			mock_round_issuance_range(10_000_000, mock_annual_to_block(schedule))
		);
	}
	#[test]
	fn range_issuance_conversion() {
		// 3-5% inflation for 10_000_0000 = 300_000-500,000 minted over the year
		// let's assume there are 10 periods in a year
		// => mint 300_000-500_000 over 10 periods => 30_000-50_000 minted per period
		let expected_round_issuance_range: Range<u128> = Range {
			min: 29_603,
			ideal: 39298,
			max: 48_909,
		};
		let schedule = Range {
			min: Perbill::from_percent(3),
			ideal: Perbill::from_percent(4),
			max: Perbill::from_percent(5),
		};
		assert_eq!(
			expected_round_issuance_range,
			mock_round_issuance_range(10_000_000, mock_annual_to_round(schedule, 10))
		);
	}
	#[test]
	fn expected_parameterization() {
		let expected_round_schedule: Range<u128> = Range {
			min: 45,
			ideal: 56,
			max: 56,
		};
		let schedule = Range {
			min: Perbill::from_percent(4),
			ideal: Perbill::from_percent(5),
			max: Perbill::from_percent(5),
		};
		assert_eq!(
			expected_round_schedule,
			mock_round_issuance_range(10_000_000, mock_annual_to_round(schedule, 8766))
		);
	}
}
