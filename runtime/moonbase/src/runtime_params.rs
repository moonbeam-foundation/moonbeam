// Copyright 2019-2022 PureStake Inc.
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
use crate::currency::{SUPPLY_FACTOR, UNIT};
use crate::{Balance, Runtime};
use account::AccountId20;
use frame_support::dynamic_params::{dynamic_pallet_params, dynamic_params};

#[dynamic_params(RuntimeParameters, pallet_parameters::Parameters::<Runtime>)]
pub mod dynamic_params {
	use super::*;

	#[dynamic_pallet_params]
	#[codec(index = 0)]
	pub mod pallet_referenda {

		#[codec(index = 0)]
		pub static SubmissionDeposit: Balance = 10 * UNIT * SUPPLY_FACTOR;
	}

	#[dynamic_pallet_params]
	#[codec(index = 1)]
	pub mod xcm_executor {

		#[codec(index = 0)]
		/// Xcm fees will go to the treasury account
		pub static XcmFeesAccount: AccountId20 = crate::Treasury::account_id();
	}
}

#[cfg(feature = "runtime-benchmarks")]
impl Default for RuntimeParameters {
	fn default() -> Self {
		RuntimeParameters::PalletReferenda(
			dynamic_params::pallet_referenda::Parameters::SubmissionDeposit(
				dynamic_params::pallet_referenda::SubmissionDeposit,
				Some(10 * UNIT * SUPPLY_FACTOR),
			),
		)
	}
}
