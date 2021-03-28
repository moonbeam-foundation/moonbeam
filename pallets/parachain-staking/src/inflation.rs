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

//! Helper methods for computing issuance based on inflation
use crate::pallet::{BalanceOf, Config, Pallet};
use frame_support::traits::Currency;
use parity_scale_codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::{Perbill, RuntimeDebug};

const SECONDS_PER_YEAR: u32 = 31557600;
const SECONDS_PER_BLOCK: u32 = 6;
const BLOCKS_PER_YEAR: u32 = SECONDS_PER_YEAR / SECONDS_PER_BLOCK;

fn rounds_per_year<T: Config>() -> u32 {
	let blocks_per_round = <Pallet<T>>::round().length;
	BLOCKS_PER_YEAR / blocks_per_round
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Eq, PartialEq, Clone, Encode, Decode, Default, RuntimeDebug)]
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

/// Convert annual inflation rate range to round inflation range
pub fn annual_to_round<T: Config>(annual: Range<Perbill>) -> Range<Perbill> {
	let periods = rounds_per_year::<T>();
	Range {
		min: Perbill::from_parts(annual.min.deconstruct() / periods),
		ideal: Perbill::from_parts(annual.ideal.deconstruct() / periods),
		max: Perbill::from_parts(annual.max.deconstruct() / periods),
	}
}

/// Compute round issuance range from round inflation range and current total issuance
pub fn round_issuance_range<T: Config>(round: Range<Perbill>) -> Range<BalanceOf<T>> {
	let circulating = T::Currency::total_issuance();
	Range {
		min: round.min * circulating,
		ideal: round.ideal * circulating,
		max: round.max * circulating,
	}
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Eq, PartialEq, Clone, Encode, Decode, Default, RuntimeDebug)]
pub struct InflationInfo<Balance> {
	/// Staking expectations
	pub expect: Range<Balance>,
	/// Round inflation range
	pub round: Range<Perbill>,
}

impl<Balance> InflationInfo<Balance> {
	pub fn new<T: Config>(
		annual: Range<Perbill>,
		expect: Range<Balance>,
	) -> InflationInfo<Balance> {
		InflationInfo {
			expect,
			round: annual_to_round::<T>(annual),
		}
	}
	/// Set round inflation range according to input annual inflation range
	pub fn set_annual_rate<T: Config>(&mut self, new: Range<Perbill>) {
		self.round = annual_to_round::<T>(new);
	}
	/// Set staking expectations
	pub fn set_expectations(&mut self, expect: Range<Balance>) {
		self.expect = expect;
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	fn mock_annual_to_round(annual: Range<Perbill>, rounds_per_year: u32) -> Range<Perbill> {
		Range {
			min: Perbill::from_parts(annual.min.deconstruct() / rounds_per_year),
			ideal: Perbill::from_parts(annual.ideal.deconstruct() / rounds_per_year),
			max: Perbill::from_parts(annual.max.deconstruct() / rounds_per_year),
		}
	}
	fn mock_round_issuance_range(
		// Total circulating before minting
		circulating: u128,
		// Round inflation range
		round: Range<Perbill>,
	) -> Range<u128> {
		Range {
			min: round.min * circulating,
			ideal: round.ideal * circulating,
			max: round.max * circulating,
		}
	}
	#[test]
	fn simple_issuance_conversion() {
		// 5% inflation for 10_000_0000 = 500,000 minted over the year
		// let's assume there are 10 periods in a year
		// => mint 500_000 over 10 periods => 50_000 minted per period
		let expected_round_issuance_range: Range<u128> = Range {
			min: 50_000,
			ideal: 50_000,
			max: 50_000,
		};
		let schedule = Range {
			min: Perbill::from_percent(5),
			ideal: Perbill::from_percent(5),
			max: Perbill::from_percent(5),
		};
		assert_eq!(
			expected_round_issuance_range,
			mock_round_issuance_range(10_000_000, mock_annual_to_round(schedule, 10))
		);
	}
	#[test]
	fn range_issuance_conversion() {
		// 3-5% inflation for 10_000_0000 = 300_000-500,000 minted over the year
		// let's assume there are 10 periods in a year
		// => mint 300_000-500_000 over 10 periods => 30_000-50_000 minted per period
		let expected_round_issuance_range: Range<u128> = Range {
			min: 30_000,
			ideal: 40_000,
			max: 50_000,
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
			min: 46,
			ideal: 57,
			max: 57,
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
