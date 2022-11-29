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
use std::fs::File;
use std::io::Write;

#[derive(Clone, Copy)]
pub enum CurveType {
	/// Approval is defined as the share of approval vote-weight (i.e. after adjustment for
	/// conviction) against the total number of vote-weight (for both approval and rejection).
	Approval,
	/// Support is the total number of votes in approval (i.e. ignoring any adjustment for
	/// conviction) compared to the total possible amount of votes that could be made in the system.
	Support,
}

pub(crate) fn plot_curve(ty: CurveType, name: String, id: u16, curve: &Curve, days: u32) {
	let (plot_file, points_file) = match ty {
		CurveType::Approval => (
			format!("plots/{} Approval.png", name),
			format!("points/{} Approval.csv", name),
		),
		CurveType::Support => (
			format!("plots/{} Support.png", name),
			format!("points/{} Support.csv", name),
		),
	};
	let grid = BitMapBackend::new(&plot_file, (600, 400)).into_drawing_area();
	grid.fill(&WHITE).unwrap();
	let hours = 24 * days;
	let title = match ty {
		CurveType::Approval => format!("{} Approval Requirement, TrackID #{}", name, id),
		CurveType::Support => format!("{} Support Requirement, TrackID #{}", name, id),
	};
	let mut plot = ChartBuilder::on(&grid)
		.caption(&title, ("sans-serif", 30))
		.margin(5)
		.set_left_and_bottom_label_area_size(40)
		.build_cartesian_2d(0..hours + 1, 0..100)
		.unwrap();
	let y_axis_label = match ty {
		CurveType::Approval => "% of Votes in Favor / All Votes in This Referendum",
		CurveType::Support => "% of Votes in This Referendum / Total Possible Turnout",
	};
	let x_axis_label = format!("Hours into {}-Day Decision Period", days);
	plot.configure_mesh()
		.y_desc(y_axis_label)
		.x_desc(x_axis_label)
		.axis_desc_style(("sans-serif", 15))
		.draw()
		.unwrap();
	let curve_points = |crv, pts| {
		(0..=pts).map(move |x| {
			(
				x,
				perbill_to_percent_coordinate(threshold(crv, Perbill::from_rational(x, pts))),
			)
		})
	};
	plot.draw_series(LineSeries::new(curve_points(curve, hours), &RED))
		.unwrap();
	write_curve_points_csv(points_file, curve_points(curve, hours).collect());
}

/// Write curve points to file
fn write_curve_points_csv(file: String, points: Vec<(u32, i32)>) {
	let mut file = File::create(file).unwrap();
	for (x, y) in points {
		file.write_all(format!("{}, {}\n", x, y).as_bytes())
			.unwrap();
	}
}

/// Input Perbill, output i32 between 0 and 100
fn perbill_to_percent_coordinate(input: Perbill) -> i32 {
	(input.deconstruct() / (Perbill::one().deconstruct() / 100))
		.try_into()
		.unwrap()
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
