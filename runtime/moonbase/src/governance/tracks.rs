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

//! Track configurations for governance.

use super::*;
use crate::currency::{KILOUNIT, SUPPLY_FACTOR, UNIT};

const fn percent(x: i32) -> sp_runtime::FixedI64 {
	sp_runtime::FixedI64::from_rational(x as u128, 100)
}
use pallet_referenda::Curve;
const TRACKS_DATA: [(u16, pallet_referenda::TrackInfo<Balance, BlockNumber>); 8] = [
	(
		0,
		pallet_referenda::TrackInfo {
			// Name of this track.
			name: "root",
			// A limit for the number of referenda on this track that can be being decided at once.
			// For Root origin this should generally be just one.
			max_deciding: 1,
			// Amount that must be placed on deposit before a decision can be made.
			decision_deposit: 100 * KILOUNIT * SUPPLY_FACTOR,
			// Amount of time this must be submitted for before a decision can be made.
			prepare_period: 3 * HOURS,
			// Amount of time that a decision may take to be approved prior to cancellation.
			decision_period: 14 * DAYS,
			// Amount of time that the approval criteria must hold before it can be approved.
			confirm_period: 3 * HOURS,
			// Minimum amount of time that an approved proposal must be in the dispatch queue.
			min_enactment_period: 3 * HOURS,
			// Minimum aye votes as percentage of overall conviction-weighted votes needed for
			// approval as a function of time into decision period.
			min_approval: Curve::make_reciprocal(4, 14, percent(80), percent(50), percent(100)),
			// Minimum pre-conviction aye-votes ("support") as percentage of overall population that
			// is needed for approval as a function of time into decision period.
			min_support: Curve::make_linear(14, 14, percent(0), percent(50)),
		},
	),
	(
		1,
		pallet_referenda::TrackInfo {
			name: "whitelisted_caller",
			max_deciding: 10,
			decision_deposit: 10 * KILOUNIT * SUPPLY_FACTOR,
			prepare_period: 30 * MINUTES,
			decision_period: 14 * DAYS,
			confirm_period: 10 * MINUTES,
			min_enactment_period: 30 * MINUTES,
			min_approval: Curve::make_reciprocal(
				1,
				14 * 24,
				percent(96),
				percent(50),
				percent(100),
			),
			min_support: Curve::make_reciprocal(1, 14 * 24, percent(4), percent(2), percent(50)),
		},
	),
	(
		10,
		pallet_referenda::TrackInfo {
			name: "treasurer",
			max_deciding: 1,
			decision_deposit: 10 * KILOUNIT * SUPPLY_FACTOR,
			prepare_period: 1 * DAYS,
			decision_period: 14 * DAYS,
			confirm_period: 2 * DAYS,
			min_enactment_period: 2 * DAYS,
			min_approval: Curve::make_linear(14, 14, percent(50), percent(100)),
			min_support: Curve::make_reciprocal(10, 14, percent(10), percent(0), percent(50)),
		},
	),
	(
		11,
		pallet_referenda::TrackInfo {
			name: "referendum_canceller",
			max_deciding: 100,
			decision_deposit: 5 * KILOUNIT * SUPPLY_FACTOR,
			prepare_period: 4,
			decision_period: 14 * DAYS,
			confirm_period: 1 * DAYS,
			min_enactment_period: 10 * MINUTES,
			min_approval: Curve::make_reciprocal(1, 14, percent(96), percent(50), percent(100)),
			min_support: Curve::make_reciprocal(1, 14, percent(1), percent(0), percent(50)),
		},
	),
	(
		12,
		pallet_referenda::TrackInfo {
			name: "referendum_killer",
			max_deciding: 100,
			decision_deposit: 5 * KILOUNIT * SUPPLY_FACTOR,
			prepare_period: 4,
			decision_period: 14 * DAYS,
			confirm_period: 1 * DAYS,
			min_enactment_period: 10 * MINUTES,
			min_approval: Curve::make_reciprocal(1, 14, percent(96), percent(50), percent(100)),
			min_support: Curve::make_reciprocal(7, 14, percent(1), percent(0), percent(10)),
		},
	),
	(
		13,
		pallet_referenda::TrackInfo {
			name: "small_spender",
			max_deciding: 5,
			decision_deposit: 300 * UNIT * SUPPLY_FACTOR,
			prepare_period: 4,
			decision_period: 14 * DAYS,
			confirm_period: 12 * HOURS,
			min_enactment_period: 1 * DAYS,
			min_approval: Curve::make_linear(8, 14, percent(50), percent(100)),
			min_support: Curve::make_reciprocal(2, 14, percent(1), percent(0), percent(10)),
		},
	),
	(
		14,
		pallet_referenda::TrackInfo {
			name: "medium_spender",
			max_deciding: 5,
			decision_deposit: 3000 * UNIT * SUPPLY_FACTOR,
			prepare_period: 4,
			decision_period: 14 * DAYS,
			confirm_period: 24 * HOURS,
			min_enactment_period: 1 * DAYS,
			min_approval: Curve::make_linear(10, 14, percent(50), percent(100)),
			min_support: Curve::make_reciprocal(4, 14, percent(1), percent(0), percent(10)),
		},
	),
	(
		15,
		pallet_referenda::TrackInfo {
			name: "big_spender",
			max_deciding: 5,
			decision_deposit: 30 * KILOUNIT * SUPPLY_FACTOR,
			prepare_period: 4,
			decision_period: 14 * DAYS,
			confirm_period: 48 * HOURS,
			min_enactment_period: 1 * DAYS,
			min_approval: Curve::make_linear(14, 14, percent(50), percent(100)),
			min_support: Curve::make_reciprocal(8, 14, percent(1), percent(0), percent(10)),
		},
	),
];

pub struct TracksInfo;
impl pallet_referenda::TracksInfo<Balance, BlockNumber> for TracksInfo {
	type Id = u16;
	type Origin = <Origin as frame_support::traits::OriginTrait>::PalletsOrigin;
	fn tracks() -> &'static [(Self::Id, pallet_referenda::TrackInfo<Balance, BlockNumber>)] {
		&TRACKS_DATA[..]
	}
	fn track_for(id: &Self::Origin) -> Result<Self::Id, ()> {
		if let Ok(system_origin) = frame_system::RawOrigin::try_from(id.clone()) {
			match system_origin {
				frame_system::RawOrigin::Root => Ok(0),
				_ => Err(()),
			}
		} else if let Ok(custom_origin) = origins::Origin::try_from(id.clone()) {
			match custom_origin {
				origins::Origin::WhitelistedCaller => Ok(1),
				// Unlimited spender
				origins::Origin::Treasurer => Ok(10),
				// Referendum admins
				origins::Origin::ReferendumCanceller => Ok(11),
				origins::Origin::ReferendumKiller => Ok(12),
				// Limited spenders
				origins::Origin::SmallSpender => Ok(13),
				origins::Origin::MediumSpender => Ok(14),
				origins::Origin::BigSpender => Ok(15),
			}
		} else {
			Err(())
		}
	}
}

#[test]
#[should_panic] // comment out to see curve info for all tracks
fn print_all_approval_and_support_curves() {
	for (_, track_info) in TRACKS_DATA {
		println!("{} TRACK", track_info.name);
		let decision_period_days = track_info.decision_period / DAYS;
		println!(
			"{} DECISION PERIOD: {} days",
			track_info.name, decision_period_days
		);
		println!("{} MIN APPROVAL:", track_info.name);
		track_info
			.min_approval
			.info(decision_period_days, track_info.name);
		println!("{} MIN SUPPORT:", track_info.name);
		track_info
			.min_support
			.info(decision_period_days, track_info.name);
	}
	assert!(false);
}
