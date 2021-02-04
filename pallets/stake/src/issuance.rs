//! Helper methods for computing issuance based on inflation
use crate::{BalanceOf, Config, Inflation};
use frame_support::traits::{Currency, Get};
use inflation::InflationSchedule;
use sp_runtime::Perbill;

const SECONDS_PER_YEAR: u32 = 31557600;
const SECONDS_PER_BLOCK: u32 = 6;
const BLOCKS_PER_YEAR: u32 = SECONDS_PER_YEAR / SECONDS_PER_BLOCK;

fn rounds_per_year<T: Config>() -> u32 {
	BLOCKS_PER_YEAR / T::BlocksPerRound::get()
}

/// Convert annual inflation schedule to round issuance settings
/// - called whenever the annual inflation schedule is changed to update round issuance
pub fn per_round<T: Config + Inflation>(
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
