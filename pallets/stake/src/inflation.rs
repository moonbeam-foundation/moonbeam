//! Helper methods for computing issuance based on inflation
use crate::{BalanceOf, Config};
use frame_support::traits::{Currency, Get};
use parity_scale_codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::{Perbill, RuntimeDebug};

const SECONDS_PER_YEAR: u32 = 31557600;
const SECONDS_PER_BLOCK: u32 = 6;
const BLOCKS_PER_YEAR: u32 = SECONDS_PER_YEAR / SECONDS_PER_BLOCK;

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Eq, PartialEq, Clone, Encode, Decode, Default, RuntimeDebug)]
pub struct Range<T: Ord> {
	pub min: T,
	pub ideal: T,
	pub max: T,
}

impl<T: Ord> Range<T> {
	pub fn is_valid(&self) -> bool {
		self.max >= self.ideal && self.ideal >= self.min
	}
	pub fn is_one(&self) -> bool {
		self.max == self.ideal && self.ideal == self.min
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

fn rounds_per_year<T: Config>() -> u32 {
	BLOCKS_PER_YEAR / T::BlocksPerRound::get()
}

/// Converts annual inflation schedule to round issuance settings
pub fn per_round<T: Config>(schedule: Range<Perbill>) -> Range<BalanceOf<T>> {
	let rounds_per_year = rounds_per_year::<T>();
	let total_issuance = T::Currency::total_issuance();
	let ideal_annual_issuance = schedule.ideal * total_issuance;
	let max_annual_issuance = schedule.max * total_issuance;
	let min_annual_issuance = schedule.min * total_issuance;
	let (min, ideal, max): (BalanceOf<T>, BalanceOf<T>, BalanceOf<T>) = (
		min_annual_issuance / rounds_per_year.into(),
		ideal_annual_issuance / rounds_per_year.into(),
		max_annual_issuance / rounds_per_year.into(),
	);
	Range { min, ideal, max }
}

#[cfg(test)]
mod tests {
	use super::*;
	fn mock_periodic_issuance(
		// Annual inflation schedule
		schedule: Range<Perbill>,
		// Total circulating before minting
		pre_issuance_circulating: u128,
		// Total number of periods
		periods: u128,
	) -> Range<u128> {
		let ideal_issuance = schedule.ideal * pre_issuance_circulating;
		let max_issuance = schedule.max * pre_issuance_circulating;
		let min_issuance = schedule.min * pre_issuance_circulating;
		let (min, ideal, max): (u128, u128, u128) = (
			min_issuance / periods,
			ideal_issuance / periods,
			max_issuance / periods,
		);
		Range { min, ideal, max }
	}
	#[test]
	fn simple_issuance_conversion() {
		// 5% inflation for 10_000_0000 = 500,000 minted over the year
		// let's assume there are 10 periods in a year
		// => mint 500_000 over 10 periods => 50_000 minted per period
		let expected_round_schedule: Range<u128> = Range {
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
			expected_round_schedule,
			mock_periodic_issuance(schedule, 10_000_000, 10)
		);
	}
	#[test]
	fn range_issuance_conversion() {
		// 3-5% inflation for 10_000_0000 = 300_000-500,000 minted over the year
		// let's assume there are 10 periods in a year
		// => mint 300_000-500_000 over 10 periods => 30_000-50_000 minted per period
		let expected_round_schedule: Range<u128> = Range {
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
			expected_round_schedule,
			mock_periodic_issuance(schedule, 10_000_000, 10)
		);
	}
	#[test]
	fn current_parameterization() {
		let expected_round_schedule: Range<u128> = Range {
			min: 1,
			ideal: 1,
			max: 2,
		};
		let schedule = Range {
			min: Perbill::from_percent(4),
			ideal: Perbill::from_percent(5),
			max: Perbill::from_percent(6),
		};
		assert_eq!(
			expected_round_schedule,
			mock_periodic_issuance(schedule, 10_000_000, 262980)
		);
	}
	// this shows us that we need to increase the `BlocksPerRound` in order for the `stake` pallet to work
	// what is the minimum `BlocksPerRound`?
	#[test]
	fn proposed_parameterization() {
		// 4-6% annual inflation
		// 10_000_000 total circulating pre issuance
		// RoundsPerYear = BLOCKS_PER_YEAR / BLOCKS_PER_ROUND = (31557600 / 6) / X = 10000
		// solve for X = 525.96 ~= 526 BLOCKS_PER_ROUND => ROUND is 52.596 hours = 2.17 days
		// 400_000-600_000 minted
		let expected_round_schedule: Range<u128> = Range {
			min: 40,
			ideal: 50,
			max: 60,
		};
		let schedule = Range {
			min: Perbill::from_percent(4),
			ideal: Perbill::from_percent(5),
			max: Perbill::from_percent(6),
		};
		assert_eq!(
			expected_round_schedule,
			mock_periodic_issuance(schedule, 10_000_000, 10000)
		);
	}
}
