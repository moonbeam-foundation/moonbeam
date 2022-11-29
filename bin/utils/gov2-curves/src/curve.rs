// Copyright 2022 Parity Technologies (UK) Ltd.
// This file is part of Polkadot.

// Polkadot is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Polkadot is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Polkadot. If not, see <http://www.gnu.org/licenses/>.

//! Temporary curve threshold until substrate patch is made and pulled in

use pallet_referenda::Curve;
use sp_arithmetic::{Rounding::*, SignedRounding::*};
use sp_runtime::{FixedI64, Perbill};

/// Input Perbill, output i32 between 0 and 100
pub fn perbill_to_percent_coordinate(input: Perbill) -> i32 {
	(input.deconstruct() / (Perbill::one().deconstruct() / 100))
		.try_into()
		.expect("failed conversion")
}

#[test]
fn perbill_to_i32_percent_conversion() {
	for i in 0..100 {
		let j: i32 = i.into();
		assert_eq!(perbill_to_percent_coordinate(Perbill::from_percent(i)), j);
	}
}

// TODO: expose in substrate
/// Determine the `y` value for the given `x` value.
pub fn threshold(curve: &Curve, x: Perbill) -> Perbill {
	match curve {
		Curve::LinearDecreasing {
			length,
			floor,
			ceil,
		} => *ceil - (x.min(*length).saturating_div(*length, Down) * (*ceil - *floor)),
		Curve::SteppedDecreasing {
			begin,
			end,
			step,
			period,
		} => (*begin - (step.int_mul(x.int_div(*period))).min(*begin)).max(*end),
		Curve::Reciprocal {
			factor,
			x_offset,
			y_offset,
		} => factor
			.checked_rounding_div(FixedI64::from(x) + *x_offset, Low)
			.map(|yp| (yp + *y_offset).into_clamped_perthing())
			.unwrap_or_else(Perbill::one),
	}
}
