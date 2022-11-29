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

use moonbase_runtime::{governance::TracksInfo as MoonbaseTracks, Balance, BlockNumber, DAYS};
use pallet_referenda::TracksInfo;

mod curve;
use curve::*;

fn write_moonbase_gov2_curves() {
	for (track_id, track) in <MoonbaseTracks as TracksInfo<Balance, BlockNumber>>::tracks() {
		let decision_period_days = track.decision_period / DAYS;
		plot_curve(
			CurveType::Approval,
			track.name.to_string(),
			*track_id,
			&track.min_approval,
			decision_period_days,
		);
		plot_curve(
			CurveType::Support,
			track.name.to_string(),
			*track_id,
			&track.min_support,
			decision_period_days,
		);
	}
}

fn main() {
	write_moonbase_gov2_curves();
}
