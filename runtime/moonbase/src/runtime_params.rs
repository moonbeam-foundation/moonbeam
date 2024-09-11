// Copyright 2024 Moonbeam Foundation.
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

//! Dynamic runtime parametes.
use frame_support::dynamic_params::{dynamic_pallet_params, dynamic_params};
use sp_runtime::Perbill;
use crate::{Runtime, currency, Balance};

#[dynamic_params(RuntimeParameters, pallet_parameters::Parameters::<Runtime>)]
pub mod dynamic_params {
	use super::*;
	#[dynamic_pallet_params]
	#[codec(index = 0)]
	pub mod runtime_config {
		// for fees, 80% are burned, 20% to the treasury
		#[codec(index = 0)]
		pub static FeesTreasuryProportion: Perbill = Perbill::from_percent(20);
	}

	#[dynamic_pallet_params]
	#[codec(index = 1)]
	pub mod pallet_randomness {
		#[codec(index = 0)]
		pub static Deposit: Balance = 1 * currency::UNIT * currency::SUPPLY_FACTOR;
	}
}

#[cfg(feature = "runtime-benchmarks")]
impl Default for RuntimeParameters {
	fn default() -> Self {
		RuntimeParameters::RuntimeConfig(
			dynamic_params::runtime_config::Parameters::FeesTreasuryProportion(
				dynamic_params::runtime_config::FeesTreasuryProportion,
				Some(Perbill::from_percent(20)),
			),
		)
	}
}
