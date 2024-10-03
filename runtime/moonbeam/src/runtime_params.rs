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

//! Dynamic runtime parameters for moonbeam.

use moonbeam_runtime_common::gen_runtime_params;

gen_runtime_params!(
	UNIT: GLMR,
	RuntimeConfig_TreasuryProportion: Perbill::from_percent(20),
	PalletRandomness_Deposit:
		BoundedU128::const_new::<{ 1 * currency::GLMR * currency::SUPPLY_FACTOR }>(),
);
