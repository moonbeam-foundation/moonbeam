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

/// Convert annual inflation schedule to round issuance settings
/// - called whenever the annual inflation schedule is changed to update round issuance
pub fn per_round<T: Config>(
	schedule: InflationSchedule<Perbill>,
) -> InflationSchedule<BalanceOf<T>> {
	let rounds_per_year = rounds_per_year::<T>();
	let total_issuance = T::Currency::total_issuance();
	let ideal_annual_issuance = schedule.ideal * total_issuance;
	let max_annual_issuance = schedule.max * total_issuance;
	let min_annual_issuance = schedule.min * total_issuance;
	let (max, min, ideal): (BalanceOf<T>, BalanceOf<T>, BalanceOf<T>) = (
		max_annual_issuance / rounds_per_year.into(),
		min_annual_issuance / rounds_per_year.into(),
		ideal_annual_issuance / rounds_per_year.into(),
	);
	InflationSchedule { max, min, ideal }
}

#[cfg(test)]
mod tests {
	// TODO: write a mock function with similar logic and test the conversion
	#[test]
	fn round_issuance_conversion() {
		assert!(true);
	}
}
