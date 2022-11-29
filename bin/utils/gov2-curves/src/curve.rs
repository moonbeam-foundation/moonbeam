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

use pallet_referenda::Curve;
use plotters::prelude::*;
use sp_arithmetic::{Rounding::*, SignedRounding::*};
use sp_runtime::{FixedI64, Perbill};

pub fn plot_curve(curve: &Curve, curve_name: String, days: u32) {
	let file_name = format!("plots/{}.png", curve_name);
	let grid = BitMapBackend::new(&file_name, (600, 400)).into_drawing_area();

	grid.fill(&WHITE).unwrap();

	let mut plot = ChartBuilder::on(&grid)
		.caption(&curve_name, ("sans-serif", 30))
		.margin(5)
		.set_left_and_bottom_label_area_size(40)
		.build_cartesian_2d(0..days + 1, 0..100)
		.unwrap();

	plot.configure_mesh()
		.y_desc("Percent %")
		.x_desc("Days into Decision Period")
		.axis_desc_style(("sans-serif", 15))
		.draw()
		.unwrap();

	plot.draw_series(LineSeries::new(
		(0..=days).map(|x| {
			(
				x,
				perbill_to_percent_coordinate(threshold(curve, Perbill::from_rational(x, days))),
			)
		}),
		&GREEN,
	))
	.unwrap();
}

/// Input Perbill, output i32 between 0 and 100
fn perbill_to_percent_coordinate(input: Perbill) -> i32 {
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
fn threshold(curve: &Curve, x: Perbill) -> Perbill {
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
