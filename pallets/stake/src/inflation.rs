//! Helper methods for computing issuance based on inflation
use crate::{BalanceOf, Config};
use frame_support::traits::{Currency, Get};
use parity_scale_codec::{Decode, Encode};
use sp_runtime::{Perbill, RuntimeDebug};

const SECONDS_PER_YEAR: u32 = 31557600;
const SECONDS_PER_BLOCK: u32 = 6;
const BLOCKS_PER_YEAR: u32 = SECONDS_PER_YEAR / SECONDS_PER_BLOCK;

#[derive(Eq, PartialEq, Clone, Encode, Decode, Default, RuntimeDebug)]
pub struct InflationSchedule<T: Ord> {
	pub min: T,
	pub ideal: T,
	pub max: T,
}

impl<T: Ord> InflationSchedule<T> {
	pub fn valid(&self) -> bool {
		self.max >= self.ideal && self.ideal >= self.min
	}
}

fn rounds_per_year<T: Config>() -> u32 {
	BLOCKS_PER_YEAR / T::BlocksPerRound::get()
}

/// Converts annual inflation schedule to round issuance settings
pub fn per_round<T: Config>(
	schedule: InflationSchedule<Perbill>,
) -> InflationSchedule<BalanceOf<T>> {
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
	InflationSchedule { min, ideal, max }
}

#[cfg(test)]
mod tests {
	use super::*;
	fn mock_periodic_issuance(
		// Annual inflation schedule
		schedule: InflationSchedule<Perbill>,
		// Total current issuance at time of minting
		total_issuance: u128,
		// Total number of periods
		periods: u128,
	) -> InflationSchedule<u128> {
		let ideal_issuance = schedule.ideal * total_issuance;
		let max_issuance = schedule.max * total_issuance;
		let min_issuance = schedule.min * total_issuance;
		let (min, ideal, max): (u128, u128, u128) = (
			min_issuance / periods,
			ideal_issuance / periods,
			max_issuance / periods,
		);
		InflationSchedule { min, ideal, max }
	}
	#[test]
	fn periodic_issuance_conversion() {
		// 5% inflation for 10_000_0000 = 500,000 minted over the year
		// let's assume there are 10 periods in a year for the sake of simplicity
		// => mint 500_000 over 10 periods => 50_000 minted per period
		let expected_round_schedule: InflationSchedule<u128> = InflationSchedule {
			min: 50_000,
			ideal: 50_000,
			max: 50_000,
		};
		let schedule = InflationSchedule {
			min: Perbill::from_percent(5),
			ideal: Perbill::from_percent(5),
			max: Perbill::from_percent(5),
		};
		assert_eq!(
			expected_round_schedule,
			mock_periodic_issuance(schedule, 10_000_000, 10)
		);
	}
}
