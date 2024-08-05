use frame_support::dynamic_params::{dynamic_pallet_params, dynamic_params};
use crate::{Treasury, Runtime, Balance};
use crate::currency::{UNIT, SUPPLY_FACTOR};

#[dynamic_params(RuntimeParameters, pallet_parameters::Parameters::<Runtime>)]
pub mod dynamic_params {
	use super::*;

    #[dynamic_pallet_params]
	#[codec(index = 0)]
	pub mod pallet_referenda {

		#[codec(index = 0)]
		pub static SubmissionDeposit: Balance = 10 * UNIT * SUPPLY_FACTOR;
	}

}

#[cfg(feature = "runtime-benchmarks")]
impl Default for RuntimeParameters {
    fn default() -> Self {
        RuntimeParameters::PalletReferenda(
			runtime_params::dynamic_params::pallet_referenda::Parameters::SubmissionDeposit(
				runtime_params::dynamic_params::pallet_referenda::SubmissionDeposit,
				Some(10 * UNIT * SUPPLY_FACTOR),
			)
		)
    }
}