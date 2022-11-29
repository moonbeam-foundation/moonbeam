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

use moonbase_runtime::{governance::TracksInfo as MoonbaseTracks, Balance, BlockNumber, DAYS};
use pallet_referenda::{Curve, TracksInfo};
use plotters::prelude::*;
use sp_runtime::Perbill;

pub mod curve;
use curve::*;

fn plot_curve(curve: &Curve, curve_name: String, days: u32) {
	let file_name = format!("plots/{}.png", curve_name);
	let grid = BitMapBackend::new(&file_name, (600, 400)).into_drawing_area();

	grid.fill(&WHITE).unwrap();

	let mut plot = ChartBuilder::on(&grid)
		.caption(&curve_name, ("Arial", 30))
		// Y axis size is 40 px
		.set_label_area_size(LabelAreaPosition::Left, 40)
		// X axis size is 40 px
		.set_label_area_size(LabelAreaPosition::Bottom, 40)
		.build_cartesian_2d(0..days, 0..100)
		.unwrap();

	plot.draw_series(LineSeries::new(
		(0..days).map(|x| {
			(
				x,
				perbill_to_percent_coordinate(threshold(curve, Perbill::from_rational(x, days))),
			)
		}),
		&GREEN,
	))
	.unwrap();

	plot.configure_mesh().draw().unwrap();
}

fn print_moonbase_track_info() {
	for (track_id, track) in <MoonbaseTracks as TracksInfo<Balance, BlockNumber>>::tracks() {
		println!("{} TRACK, ID # {}", track.name, track_id);
		let decision_period_days = track.decision_period / DAYS;
		println!(
			"{} DECISION PERIOD: {} days",
			track.name, decision_period_days
		);
		let approval_curve_title = format!("{} APPROVAL REQUIREMENT", track.name);
		println!("{}:", approval_curve_title);
		plot_curve(
			&track.min_approval,
			approval_curve_title,
			decision_period_days,
		);
		track.min_approval.info(decision_period_days, track.name);
		let support_curve_title = format!("{} SUPPORT REQUIREMENT", track.name);
		println!("{}:", support_curve_title);
		plot_curve(
			&track.min_support,
			support_curve_title,
			decision_period_days,
		);
		track.min_support.info(decision_period_days, track.name);
	}
}

fn main() {
	print_moonbase_track_info();
}
