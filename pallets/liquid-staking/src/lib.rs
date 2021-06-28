// Copyright 2019-2021 PureStake Inc.
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

//! # Liquid Staking Module
//!
//! ## Overview
//!
//! Module to provide interaction with Relay Chain Tokens directly
//! This module allows to
//! - Token transfer from parachain to relay chain.
//! - Token transfer from relay to parachain
//! - Exposure to staking functions

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet;

pub use pallet::*;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[pallet]
pub mod pallet {

	use frame_support::{
		pallet_prelude::*,
		storage::{with_transaction, TransactionOutcome},
		traits::{Currency, Get, ReservableCurrency, fungibles},
		Parameter, PalletId,
	};
	use frame_support::traits::fungibles::Mutate;
	use frame_system::{ensure_signed, pallet_prelude::*};
	use sp_runtime::{
		traits::{AtLeast32BitUnsigned, Convert, MaybeSerializeDeserialize, Member, Zero},
		DispatchError,
	};
	use sp_std::prelude::*;

	use xcm::v0::prelude::*;
	use xcm_executor::traits::WeightBounds;


	type BalanceOf<T> = <<T as Config>::Assets as frame_support::traits::fungibles::Inspect<
		<T as frame_system::Config>::AccountId,
	>>::Balance;

	type AssetIdOf<T> = <<T as Config>::Assets as frame_support::traits::fungibles::Inspect
	<<T as frame_system::Config>::AccountId>>::AssetId;


	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	/// Configuration trait of this pallet. We tightly couple to Parachain Staking in order to
	/// ensure that only staked accounts can create registrations in the first place. This could be
	/// generalized.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// The units in which we record balances.
		type Assets: fungibles::Mutate<Self::AccountId> + fungibles::Transfer<Self::AccountId>;

		type AssetId: Get<<Self::Assets as frame_support::traits::fungibles::Inspect
		<Self::AccountId>>::AssetId>;
		/// Convert `T::AccountId` to `MultiLocation`.
		type AccountIdToMultiLocation: Convert<Self::AccountId, MultiLocation>;

		type PalletId: Get<PalletId>;

		/// XCM executor.
		type XcmExecutor: ExecuteXcm<Self::Call>;

		/// Means of measuring the weight consumed by an XCM message locally.
		type Weigher: WeightBounds<Self::Call>;
	}

	/// An error that can occur while executing the mapping pallet's logic.
	#[pallet::error]
	pub enum Error<T> {
		MyError,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		Staked(<T as frame_system::Config>::AccountId, BalanceOf<T>),
		Unstaked(<T as frame_system::Config>::AccountId, BalanceOf<T>),
		RatioSet(u32, BalanceOf<T>),
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		pub fn stake_dot(
			origin: OriginFor<T>,
			amount: BalanceOf<T>,
			dest_weight: Weight,
		) -> DispatchResult {

			let who = ensure_signed(origin)?;
			T::Assets::burn_from(T::AssetId::get().into(), &who, amount.into())
				.map_err(|e| XcmError::FailedToTransactAsset(e.into())).unwrap();
			Self::deposit_event(Event::<T>::Staked(who.clone(), amount.clone()));
			Ok(())
		}

		#[pallet::weight(0)]
		pub fn unstake_dot(
			origin: OriginFor<T>,
			amount: BalanceOf<T>,
			dest_weight: Weight,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::deposit_event(Event::<T>::Unstaked(who, amount));
			Ok(())
		}

		#[pallet::weight(0)]
		pub fn set_ratio(origin: OriginFor<T>, dot: u32, v_dot: BalanceOf<T>) -> DispatchResult {
			ensure_root(origin)?;
			Self::deposit_event(Event::<T>::RatioSet(dot, v_dot));

			Ok(())
		}
	}
}
