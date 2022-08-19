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

pub use pallet_custom_origins::*;

#[frame_support::pallet]
pub mod pallet_custom_origins {
	use crate::{
		currency::{KILOUNIT, UNIT},
		Balance,
	};
	use frame_support::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[derive(PartialEq, Eq, Clone, MaxEncodedLen, Encode, Decode, TypeInfo, RuntimeDebug)]
	#[pallet::origin]
	pub enum Origin {
		/// Origin for cancelling slashes.
		StakingAdmin,
		/// Origin for spending (any amount of) funds.
		Treasurer,
		/// Origin for managing the registrar.
		GeneralAdmin,
		/// Origin able to cancel referenda.
		ReferendumCanceller,
		/// Origin able to kill referenda.
		ReferendumKiller,
		/// Origin able to spend up to 1 KSM from the treasury at once.
		SmallTipper,
		/// Origin able to spend up to 5 KSM from the treasury at once.
		BigTipper,
		/// Origin able to spend up to 50 KSM from the treasury at once.
		SmallSpender,
		/// Origin able to spend up to 500 KSM from the treasury at once.
		MediumSpender,
		/// Origin able to spend up to 5,000 KSM from the treasury at once.
		BigSpender,
		/// Origin able to dispatch a whitelisted call.
		WhitelistedCaller,
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
		StakingAdmin,
		GeneralAdmin,
		ReferendumCanceller,
		ReferendumKiller,
		WhitelistedCaller,
	);

	macro_rules! decl_ensure {
		(
			$vis:vis type $name:ident: EnsureOrigin<Success = $success_type:ty> {
				$( $item:ident = $success:expr, )*
			}
		) => {
			$vis struct $name;
			impl<O: Into<Result<Origin, O>> + From<Origin>>
				EnsureOrigin<O> for $name
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

	decl_ensure! {
		pub type Spender: EnsureOrigin<Success = Balance> {
			SmallTipper = 250 * 100 * UNIT,
			BigTipper = 1 * KILOUNIT,
			SmallSpender = 10 * KILOUNIT,
			MediumSpender = 100 * KILOUNIT,
			BigSpender = 1_000 * KILOUNIT,
		}
	}
}
