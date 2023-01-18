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

//! Custom origins for governance interventions.
#![cfg_attr(not(feature = "std"), no_std)]

pub use custom_origins::*;

#[frame_support::pallet]
pub mod custom_origins {
	use frame_support::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[derive(PartialEq, Eq, Clone, MaxEncodedLen, Encode, Decode, TypeInfo, RuntimeDebug)]
	#[pallet::origin]
	pub enum Origin {
		/// Origin able to dispatch a whitelisted call.
		WhitelistedCaller,
		/// General admin
		GeneralAdmin,
		/// Origin able to cancel referenda.
		ReferendumCanceller,
		/// Origin able to kill referenda.
		ReferendumKiller,
	}

	impl TryFrom<u16> for Origin {
		type Error = ();
		/// TrackId => Origin
		fn try_from(value: u16) -> Result<Origin, ()> {
			match value {
				1 => Ok(Origin::WhitelistedCaller),
				2 => Ok(Origin::GeneralAdmin),
				3 => Ok(Origin::ReferendumCanceller),
				4 => Ok(Origin::ReferendumKiller),
				_ => Err(()),
			}
		}
	}

	impl Into<u16> for Origin {
		/// Origin => TrackId
		fn into(self) -> u16 {
			match self {
				Origin::WhitelistedCaller => 1,
				Origin::GeneralAdmin => 2,
				Origin::ReferendumCanceller => 3,
				Origin::ReferendumKiller => 4,
			}
		}
	}

	#[test]
	fn origin_track_conversion_is_consistent() {
		macro_rules! has_consistent_conversions {
			( $o:expr ) => {
				let origin_as_u16 = <Origin as Into<u16>>::into($o);
				let u16_as_origin: Origin = origin_as_u16.try_into().unwrap();
				assert_eq!($o, u16_as_origin);
			};
		}
		has_consistent_conversions!(Origin::WhitelistedCaller);
		has_consistent_conversions!(Origin::GeneralAdmin);
		has_consistent_conversions!(Origin::ReferendumCanceller);
		has_consistent_conversions!(Origin::ReferendumKiller);
	}

	macro_rules! decl_unit_ensures {
		( $name:ident: $success_type:ty = $success:expr ) => {
			pub struct $name;
			impl<O: Into<Result<Origin, O>> + From<Origin>>
				EnsureOrigin<O> for $name
			{
				type Success = $success_type;
				fn try_origin(o: O) -> Result<Self::Success, O> {
					o.into().and_then(|o| match o {
						Origin::$name => Ok($success),
						r => Err(O::from(r)),
					})
				}
				#[cfg(feature = "runtime-benchmarks")]
				fn try_successful_origin() -> Result<O, ()> {
					Ok(O::from(Origin::$name))
				}
			}
		};
		( $name:ident ) => { decl_unit_ensures! { $name : () = () } };
		( $name:ident: $success_type:ty = $success:expr, $( $rest:tt )* ) => {
			decl_unit_ensures! { $name: $success_type = $success }
			decl_unit_ensures! { $( $rest )* }
		};
		( $name:ident, $( $rest:tt )* ) => {
			decl_unit_ensures! { $name }
			decl_unit_ensures! { $( $rest )* }
		};
		() => {}
	}
	decl_unit_ensures!(
		ReferendumCanceller,
		ReferendumKiller,
		WhitelistedCaller,
		GeneralAdmin
	);
}
