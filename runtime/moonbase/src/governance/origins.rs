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
	use frame_support::{pallet_prelude::*, traits::Currency};

	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Currency type to limit spends per spend origin
		type Currency: Currency<Self::AccountId>;
		/// Maximum amount able to be spent by SmallSpender origin from treasury at once
		#[pallet::constant]
		type MaxSmallSpenderSpend: Get<BalanceOf<Self>>;
		/// Maximum amount able to be spent by MediumSpender origin from treasury at once
		#[pallet::constant]
		type MaxMediumSpenderSpend: Get<BalanceOf<Self>>;
		/// Maximum amount able to be spent by BigSpender origin from treasury at once
		#[pallet::constant]
		type MaxBigSpenderSpend: Get<BalanceOf<Self>>;
		/// Maximum amount able to be spent by Treasurer origin from treasury at once
		#[pallet::constant]
		type MaxTreasurerSpend: Get<BalanceOf<Self>>;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[derive(PartialEq, Eq, Clone, MaxEncodedLen, Encode, Decode, TypeInfo, RuntimeDebug)]
	#[pallet::origin]
	pub enum Origin {
		/// Origin able to dispatch a whitelisted call.
		WhitelistedCaller,
		/// Origin able to spend up to MaxTreasurerSpend from the treasury at once.
		Treasurer,
		/// Origin able to cancel referenda.
		ReferendumCanceller,
		/// Origin able to kill referenda.
		ReferendumKiller,
		/// Origin able to spend up to MaxSmallSpenderSpend from the treasury at once.
		SmallSpender,
		/// Origin able to spend up to MaxMediumSpenderSpend from the treasury at once.
		MediumSpender,
		/// Origin able to spend up to MaxBigSpenderSpend from the treasury at once.
		BigSpender,
	}

	impl TryFrom<u8> for Origin {
		type Error = ();
		/// TrackId => Origin
		fn try_from(value: u8) -> Result<Origin, ()> {
			match value {
				1 => Ok(Origin::WhitelistedCaller),
				10 => Ok(Origin::Treasurer),
				11 => Ok(Origin::ReferendumCanceller),
				12 => Ok(Origin::ReferendumKiller),
				13 => Ok(Origin::SmallSpender),
				14 => Ok(Origin::MediumSpender),
				15 => Ok(Origin::BigSpender),
				_ => Err(()),
			}
		}
	}

	impl Into<u16> for Origin {
		/// Origin => TrackId
		fn into(self) -> u16 {
			match self {
				Origin::WhitelistedCaller => 1,
				Origin::Treasurer => 10,
				Origin::ReferendumCanceller => 11,
				Origin::ReferendumKiller => 12,
				Origin::SmallSpender => 13,
				Origin::MediumSpender => 14,
				Origin::BigSpender => 15,
			}
		}
	}

	#[test]
	fn origin_track_conversion_is_consistent() {
		macro_rules! has_consistent_conversions {
			( $o:expr ) => {
				let origin_as_u16 = <Origin as Into<u16>>::into($o);
				let u16_as_u8: u8 = origin_as_u16.try_into().unwrap();
				let u8_as_origin: Origin = u16_as_u8.try_into().unwrap();
				assert_eq!($o, u8_as_origin);
			};
		}
		has_consistent_conversions!(Origin::WhitelistedCaller);
		has_consistent_conversions!(Origin::Treasurer);
		has_consistent_conversions!(Origin::ReferendumCanceller);
		has_consistent_conversions!(Origin::ReferendumKiller);
		has_consistent_conversions!(Origin::SmallSpender);
		has_consistent_conversions!(Origin::MediumSpender);
		has_consistent_conversions!(Origin::BigSpender);
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
	decl_unit_ensures!(ReferendumCanceller, ReferendumKiller, WhitelistedCaller,);

	macro_rules! decl_ensure {
		(
			$vis:vis type $name:ident: EnsureOrigin<Success = $success_type:ty> {
				$( $item:ident = $success:expr, )*
			}
		) => {
			$vis struct $name<T>(sp_std::marker::PhantomData<T>);
			impl<T: Config, O: Into<Result<Origin, O>> + From<Origin>>
				EnsureOrigin<O> for $name<T>
			{
				type Success = $success_type;
				fn try_origin(o: O) -> Result<Self::Success, O> {
					o.into().and_then(|o| match o {
						$(
							Origin::$item => Ok($success),
						)*
						r => Err(O::from(r)),
					})
				}
				#[cfg(feature = "runtime-benchmarks")]
				fn try_successful_origin() -> Result<O, ()> {
					// By convention the more privileged origins go later, so for greatest chance
					// of success, we want the last one.
					let _result: Result<O, ()> = Err(());
					$(
						let _result: Result<O, ()> = Ok(O::from(Origin::$item));
					)*
					_result
				}
			}
		}
	}

	// Origins able to spend $AMOUNT from treasury at once
	decl_ensure! {
		pub type Spender: EnsureOrigin<Success = BalanceOf<T>> {
			SmallSpender = T::MaxSmallSpenderSpend::get(),
			MediumSpender = T::MaxMediumSpenderSpend::get(),
			BigSpender = T::MaxBigSpenderSpend::get(),
			Treasurer = T::MaxTreasurerSpend::get(),
		}
	}
}
