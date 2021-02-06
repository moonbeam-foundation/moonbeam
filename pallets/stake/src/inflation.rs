//! Helper methods for computing issuance based on inflation
use crate::Config;
use frame_support::traits::Get;
use parity_scale_codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::{traits::AtLeast32BitUnsigned, Perbill, RuntimeDebug};
use sp_std::ops::{Div, Mul};

const SECONDS_PER_YEAR: u32 = 31557600;
const SECONDS_PER_BLOCK: u32 = 6;
const BLOCKS_PER_YEAR: u32 = SECONDS_PER_YEAR / SECONDS_PER_BLOCK;

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Eq, PartialEq, Clone, Encode, Decode, Default, RuntimeDebug)]
pub struct InflationSchedule<Balance> {
	/// Pre-issuance circulating supply
	pub base: Balance,
	/// Annual inflation rate
	pub annual: Range<Perbill>,
	/// Staking expectations
	pub expect: Range<Balance>,
	/// Round issuance
	pub round: Range<Balance>,
}

impl<Balance: Copy + Mul<Output = Balance> + Div<Output = Balance> + AtLeast32BitUnsigned>
	InflationSchedule<Balance>
{
	pub fn new<T: Config>(
		base: Balance,
		annual: Range<Perbill>,
		expect: Range<Balance>,
	) -> InflationSchedule<Balance> {
		let rounds_per_year = rounds_per_year::<T>();
		let ideal_annual_issuance = annual.ideal * base;
		let max_annual_issuance = annual.max * base;
		let min_annual_issuance = annual.min * base;
		let round = Range {
			min: min_annual_issuance / rounds_per_year.into(),
			ideal: ideal_annual_issuance / rounds_per_year.into(),
			max: max_annual_issuance / rounds_per_year.into(),
		};
		InflationSchedule {
			base,
			annual,
			expect,
			round,
		}
	}
	/// Set annual inflation rate without changing the base amount
	pub fn set_rate<T: Config>(&mut self, new: Range<Perbill>) {
		let rounds_per_year = rounds_per_year::<T>();
		let ideal_annual_issuance = new.ideal * self.base;
		let max_annual_issuance = new.max * self.base;
		let min_annual_issuance = new.min * self.base;
		let round = Range {
			min: min_annual_issuance / rounds_per_year.into(),
			ideal: ideal_annual_issuance / rounds_per_year.into(),
			max: max_annual_issuance / rounds_per_year.into(),
		};
		self.round = round;
		self.annual = new;
	}
	/// Set staking expectations
	pub fn set_expectations(&mut self, expect: Range<Balance>) {
		self.expect = expect;
	}
	/// Set base using T::Currency::total_issuance upon a new year; updates round issuance as well
	pub fn set_base<T: Config>(&mut self, circulating: Balance) {
		let rounds_per_year = rounds_per_year::<T>();
		let ideal_annual_issuance = self.annual.ideal * circulating;
		let max_annual_issuance = self.annual.max * circulating;
		let min_annual_issuance = self.annual.min * circulating;
		let round = Range {
			min: min_annual_issuance / rounds_per_year.into(),
			ideal: ideal_annual_issuance / rounds_per_year.into(),
			max: max_annual_issuance / rounds_per_year.into(),
		};
		self.base = circulating;
		self.round = round;
	}
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
