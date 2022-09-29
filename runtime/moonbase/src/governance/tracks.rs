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
use crate::currency::{KILOUNIT, UNIT};

const fn percent(x: i32) -> sp_arithmetic::FixedI64 {
	sp_arithmetic::FixedI64::from_rational(x as u128, 100)
}
use pallet_referenda::Curve;
const APP_ROOT: Curve = Curve::make_reciprocal(4, 28, percent(80), percent(50), percent(100));
const SUP_ROOT: Curve = Curve::make_linear(28, 28, percent(0), percent(50));
const APP_TREASURER: Curve = Curve::make_reciprocal(4, 28, percent(80), percent(50), percent(100));
const SUP_TREASURER: Curve = Curve::make_linear(28, 28, percent(0), percent(50));
const APP_GENERAL_ADMIN: Curve =
	Curve::make_reciprocal(4, 28, percent(80), percent(50), percent(100));
const SUP_GENERAL_ADMIN: Curve =
	Curve::make_reciprocal(7, 28, percent(10), percent(0), percent(50));
const APP_REFERENDUM_CANCELLER: Curve = Curve::make_linear(17, 28, percent(50), percent(100));
const SUP_REFERENDUM_CANCELLER: Curve =
	Curve::make_reciprocal(12, 28, percent(1), percent(0), percent(50));
const APP_REFERENDUM_KILLER: Curve = Curve::make_linear(17, 28, percent(50), percent(100));
const SUP_REFERENDUM_KILLER: Curve =
	Curve::make_reciprocal(12, 28, percent(1), percent(0), percent(50));
const APP_SMALL_TIPPER: Curve = Curve::make_linear(10, 28, percent(50), percent(100));
const SUP_SMALL_TIPPER: Curve = Curve::make_reciprocal(1, 28, percent(4), percent(0), percent(50));
const APP_BIG_TIPPER: Curve = Curve::make_linear(10, 28, percent(50), percent(100));
const SUP_BIG_TIPPER: Curve = Curve::make_reciprocal(8, 28, percent(1), percent(0), percent(50));
const APP_SMALL_SPENDER: Curve = Curve::make_linear(17, 28, percent(50), percent(100));
const SUP_SMALL_SPENDER: Curve =
	Curve::make_reciprocal(12, 28, percent(1), percent(0), percent(50));
const APP_MEDIUM_SPENDER: Curve = Curve::make_linear(23, 28, percent(50), percent(100));
const SUP_MEDIUM_SPENDER: Curve =
	Curve::make_reciprocal(16, 28, percent(1), percent(0), percent(50));
const APP_BIG_SPENDER: Curve = Curve::make_linear(28, 28, percent(50), percent(100));
const SUP_BIG_SPENDER: Curve = Curve::make_reciprocal(20, 28, percent(1), percent(0), percent(50));
const APP_WHITELISTED_CALLER: Curve =
	Curve::make_reciprocal(16, 28 * 24, percent(96), percent(50), percent(100));
const SUP_WHITELISTED_CALLER: Curve =
	Curve::make_reciprocal(1, 28, percent(20), percent(10), percent(50));

const TRACKS_DATA: [(u16, pallet_referenda::TrackInfo<Balance, BlockNumber>); 11] = [
	(
		0,
		pallet_referenda::TrackInfo {
			name: "root",
			max_deciding: 1,
			decision_deposit: 1_000 * KILOUNIT,
			prepare_period: 3 * HOURS,
			decision_period: 28 * DAYS,
			confirm_period: 3 * HOURS,
			min_enactment_period: 3 * HOURS,
			min_approval: APP_ROOT,
			min_support: SUP_ROOT,
		},
	),
	// Fastrack
	(
		1,
		pallet_referenda::TrackInfo {
			name: "whitelisted_caller",
			max_deciding: 10,
			decision_deposit: 10_000 * KILOUNIT,
			prepare_period: 3 * HOURS,
			decision_period: 28 * DAYS,
			// TODO: review
			confirm_period: 10 * MINUTES,
			// TODO: review
			min_enactment_period: 30 * MINUTES,
			min_approval: APP_WHITELISTED_CALLER,
			min_support: SUP_WHITELISTED_CALLER,
		},
	),
	(
		10,
		pallet_referenda::TrackInfo {
			name: "treasurer",
			max_deciding: 10,
			decision_deposit: 5 * KILOUNIT,
			prepare_period: 4,
			decision_period: 28 * DAYS,
			confirm_period: 3 * HOURS,
			min_enactment_period: 2 * DAYS,
			min_approval: APP_TREASURER,
			min_support: SUP_TREASURER,
		},
	),
	(
		11,
		pallet_referenda::TrackInfo {
			name: "general_admin",
			max_deciding: 10,
			decision_deposit: 5 * KILOUNIT,
			prepare_period: 4,
			decision_period: 28 * DAYS,
			confirm_period: 3 * HOURS,
			min_enactment_period: 2 * DAYS,
			min_approval: APP_GENERAL_ADMIN,
			min_support: SUP_GENERAL_ADMIN,
		},
	),
	(
		12,
		pallet_referenda::TrackInfo {
			name: "referendum_canceller",
			max_deciding: 1_000,
			decision_deposit: 50 * KILOUNIT,
			prepare_period: 4,
			decision_period: 28 * DAYS,
			confirm_period: 3 * HOURS,
			min_enactment_period: 10 * MINUTES,
			min_approval: APP_REFERENDUM_CANCELLER,
			min_support: SUP_REFERENDUM_CANCELLER,
		},
	),
	(
		13,
		pallet_referenda::TrackInfo {
			name: "referendum_killer",
			max_deciding: 1_000,
			decision_deposit: 50 * KILOUNIT,
			prepare_period: 4,
			decision_period: 28 * DAYS,
			confirm_period: 3 * HOURS,
			min_enactment_period: 10 * MINUTES,
			min_approval: APP_REFERENDUM_KILLER,
			min_support: SUP_REFERENDUM_KILLER,
		},
	),
	(
		14,
		pallet_referenda::TrackInfo {
			name: "small_tipper",
			max_deciding: 200,
			decision_deposit: 5 * 100 * UNIT,
			prepare_period: 4,
			decision_period: 28 * DAYS,
			confirm_period: 3 * HOURS,
			min_enactment_period: 28 * DAYS,
			min_approval: APP_SMALL_TIPPER,
			min_support: SUP_SMALL_TIPPER,
		},
	),
	(
		15,
		pallet_referenda::TrackInfo {
			name: "big_tipper",
			max_deciding: 100,
			decision_deposit: 50 * 100 * UNIT,
			prepare_period: 4,
			decision_period: 28 * DAYS,
			confirm_period: 6 * HOURS,
			min_enactment_period: 28 * DAYS,
			min_approval: APP_BIG_TIPPER,
			min_support: SUP_BIG_TIPPER,
		},
	),
	(
		16,
		pallet_referenda::TrackInfo {
			name: "small_spender",
			max_deciding: 50,
			decision_deposit: 500 * 100 * UNIT,
			prepare_period: 4,
			decision_period: 28 * DAYS,
			confirm_period: 12 * HOURS,
			min_enactment_period: 28 * DAYS,
			min_approval: APP_SMALL_SPENDER,
			min_support: SUP_SMALL_SPENDER,
		},
	),
	(
		17,
		pallet_referenda::TrackInfo {
			name: "medium_spender",
			max_deciding: 20,
			decision_deposit: 1_500 * 100 * UNIT,
			prepare_period: 4,
			decision_period: 28 * DAYS,
			confirm_period: 24 * HOURS,
			min_enactment_period: 28 * DAYS,
			min_approval: APP_MEDIUM_SPENDER,
			min_support: SUP_MEDIUM_SPENDER,
		},
	),
	(
		18,
		pallet_referenda::TrackInfo {
			name: "big_spender",
			max_deciding: 10,
			decision_deposit: 5 * KILOUNIT,
			prepare_period: 4,
			decision_period: 28 * DAYS,
			confirm_period: 48 * HOURS,
			min_enactment_period: 28 * DAYS,
			min_approval: APP_BIG_SPENDER,
			min_support: SUP_BIG_SPENDER,
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
				// General admin
				origins::Origin::Treasurer => Ok(10),
				origins::Origin::GeneralAdmin => Ok(11),
				// Referendum admins
				origins::Origin::ReferendumCanceller => Ok(12),
				origins::Origin::ReferendumKiller => Ok(13),
				// Limited treasury spenders
				origins::Origin::SmallTipper => Ok(14),
				origins::Origin::BigTipper => Ok(15),
				origins::Origin::SmallSpender => Ok(16),
				origins::Origin::MediumSpender => Ok(17),
				origins::Origin::BigSpender => Ok(18),
			}
		} else {
			Err(())
		}
	}
}
