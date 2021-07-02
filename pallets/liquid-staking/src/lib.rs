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

#[pallet]
pub mod pallet {

	use std::thread::current;

	use cumulus_primitives_core::relay_chain;
	use frame_support::{
		pallet_prelude::*,
		traits::{Currency, ExistenceRequirement::AllowDeath, ReservableCurrency},
		PalletId,
	};
	use frame_system::{ensure_signed, pallet_prelude::*};
	use sp_runtime::traits::AccountIdConversion;
	use sp_runtime::traits::Convert;
	use sp_runtime::traits::{CheckedAdd, Saturating};
	use sp_runtime::SaturatedConversion;
	use sp_std::prelude::*;

	use substrate_fixed::types::{U32F32, U64F64};
	use xcm::v0::prelude::*;
	use xcm_executor::traits::WeightBounds;

	type BalanceOf<T> =
		<<T as Config>::RelayCurrency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	/// Stores info about how many DOTS someone has staked and the relation with the ratio
	#[derive(Default, Clone, Encode, Decode, RuntimeDebug)]
	pub struct StakeInfo<T: Config> {
		pub staked_without_ratio: BalanceOf<T>,
		pub staked_with_ratio: BalanceOf<T>,
	}
	/// All possible messages that may be delivered to generic Substrate chain.
	///
	/// Note this enum may be used in the context of both Source (as part of `encode-call`)
	/// and Target chain (as part of `encode-message/send-message`).
	#[derive(Debug, PartialEq, Eq)]
	pub enum AvailableCalls {
		Reserve,
	}

	pub trait EncodeCall {
		/// Encode call from the relay.
		fn encode_call(call: AvailableCalls) -> Vec<u8>;
	}

	/// Configuration trait of this pallet. We tightly couple to Parachain Staking in order to
	/// ensure that only staked accounts can create registrations in the first place. This could be
	/// generalized.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// The currency type for Relay balances
		type RelayCurrency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

		/// The Pallets PalletId
		type PalletId: Get<PalletId>;

		/// Convert `T::AccountId` to `MultiLocation`.
		type AccountIdToMultiLocation: Convert<Self::AccountId, MultiLocation>;

		/// XCM executor.
		type CallEncoder: EncodeCall;

		/// XCM executor.
		type XcmExecutor: ExecuteXcm<Self::Call>;

		/// XCM sender.
		type XcmSender: SendXcm;

		/// Means of measuring the weight consumed by an XCM message locally.
		type Weigher: WeightBounds<Self::Call>;
	}

	#[pallet::storage]
	#[pallet::getter(fn current_nomination)]
	pub type Nominations<T: Config> = StorageValue<_, Vec<relay_chain::AccountId>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn current_ratio)]
	pub type Ratio<T: Config> = StorageValue<_, U64F64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn total_staked)]
	pub type TotalStaked<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn total_staked_multiplier)]
	pub type TotalStakedultiplier<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn staked_map)]
	pub type StakedMap<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, StakeInfo<T>>;

	/// An error that can occur while executing the mapping pallet's logic.
	#[pallet::error]
	pub enum Error<T> {
		MyError,
		WrongConversionU128ToBalance,
		SendFailure,
		Overflow,
		NothingStakedToSetRatio,
		NoRewardsAvailable,
		UnstakingMoreThanStaked,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		Staked(<T as frame_system::Config>::AccountId, BalanceOf<T>),
		Unstaked(<T as frame_system::Config>::AccountId, BalanceOf<T>),
		RatioSet(BalanceOf<T>, BalanceOf<T>),
		NominationsSet(Vec<relay_chain::AccountId>),
		XcmSent(MultiLocation, Xcm<()>),
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

			// Stake bytes
			let amount_as_u128 = amount.saturated_into::<u128>();

			// We "work" as if we had a "currency", which in our case is just a mapping. We track
			// how much each user should have in case of having minted a token
			let staked = StakedMap::<T>::get(who.clone());
			// These is the total staked in V-DOT, without taking into account the ratio
			let total_staked = TotalStaked::<T>::get();
			let total_staked_multiplier = TotalStakedMultiplier::<T>::get();

			// We get the current ratio
			let current_ratio = Ratio::<T>::get();

			// This is the part where we need to take into account the ratio
			// We monitor how many "L-DOTS" would someone be assigned, with respect to the current ratio
			// This are just stored in a mapping, to later re-do the conversion
			// Example:
			// 	1) I stake 6 V-DOT when the difference when the ratio is 1.2. This means that if we
			// 	have staked 10, the sovereign account (due to that staking) has 12 now. In this case
			//  we anotate "total = 6/1.2 =5".
			// 	2) I later stake another 7-V-DOT, when the ratio is 1.4. total becomes 6+(7/1.4) = 10.
			//  3) I want to withdraw at 1.6. We know total is 10, so we give away 10*1.6 = 16 V-DOT
			//  back. I won 16-13 = 3V-DOT
			//	Note that this is essentially the same as doing: 6*(1.6/1.2) + 7*(1.4/1.6)
			// A S-C can take the opportunity to mint an additional Token  here.
			let to_monitor = U64F64::from_num(amount.saturated_into::<u128>()) / current_ratio;
			let balance_to_add = to_monitor.ceil().to_num::<u128>();
			// This aims to emulate "hoy many tokens would one receive", although nothing is minted
			// However, this serves for us to make the correct change later back to V-DOT.
			let stake_to_store = if let Some(previously_staked) = staked {
				let new_total = previously_staked
					.staked_with_ratio
					.checked_add(&(balance_to_add.saturated_into::<BalanceOf<T>>()))
					.ok_or(Error::<T>::Overflow)?;
				StakeInfo {
					staked_without_ratio: previously_staked.staked_without_ratio + amount,
					staked_with_ratio: new_total,
				}
			} else {
				StakeInfo {
					staked_without_ratio: amount,
					staked_with_ratio: balance_to_add.saturated_into::<BalanceOf<T>>(),
				}
			};

			StakedMap::<T>::insert(who.clone(), stake_to_store);
			let new_total_staked = total_staked
				.checked_add(&amount)
				.ok_or(Error::<T>::Overflow)?;
			let new_total_staked_multiplier = total_staked_multiplier
				.checked_add(&stake_to_store.staked_without_ratio)
				.ok_or(Error::<T>::Overflow)?;

			TotalStaked::<T>::put(new_total_staked);
			TotalStakedMultiplier::<T>::put(new_total_staked_multiplier);

			let stake_bytes: Vec<u8> = T::CallEncoder::encode_call(AvailableCalls::Reserve);

			// Construct messages
			let message = Self::transact(amount_as_u128, dest_weight, stake_bytes);

			// Send xcm as root
			Self::send_xcm(
				MultiLocation::Null,
				MultiLocation::X1(Parent),
				message.clone(),
			)
			.map_err(|_| Error::<T>::SendFailure)?;

			// Deposit event
			Self::deposit_event(Event::<T>::XcmSent(MultiLocation::Null, message));

			// Deposit event
			Self::deposit_event(Event::<T>::Staked(who.clone(), amount.clone()));

			// Reserve balances
			T::RelayCurrency::transfer(
				&who,
				&T::PalletId::get().into_account(),
				amount,
				AllowDeath,
			)?;

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn claim_vdot_rewards(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// We get the current ratio
			let current_ratio = Ratio::<T>::get();

			// We know how much a particular account has on rewards. Its the difference
			// between whatever Let's calculate it.
			let staked = StakedMap::<T>::get(who.clone()).ok_or(Error::<T>::NoRewardsAvailable)?;

			let current_vdot =
				(U64F64::from_num(staked.staked_with_ratio.saturated_into::<u128>())
					* current_ratio)
					.ceil()
					.to_num::<u128>()
					.saturated_into::<BalanceOf<T>>();

			ensure!(
				amount > current_vdot.saturating_sub(staked.staked_without_ratio),
				Error::<T>::NoRewardsAvailable
			);

			// Mint rewards balances
			T::RelayCurrency::deposit_into_existing(&who, amount)?;

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn unstake_dot(
			origin: OriginFor<T>,
			amount: BalanceOf<T>,
			dest_weight: Weight,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// We get the current ratio
			let current_ratio = Ratio::<T>::get();

			// We know how much a particular account has on rewards. Its the difference
			// between whatever Let's calculate it.
			let staked = StakedMap::<T>::get(who.clone()).ok_or(Error::<T>::NoRewardsAvailable)?;
			
			// This is the current v_dot (staked + reward or penalties)
			let current_vdot =
				(U64F64::from_num(staked.staked_with_ratio.saturated_into::<u128>())
					* current_ratio)
					.ceil()
					.to_num::<u128>()
					.saturated_into::<BalanceOf<T>>();

			// Do we need to burn or mint?
			if current_vdot < amount {
				ensure!(
					amount > staked.staked_without_ratio,
					Error::<T>::UnstakingMoreThanStaked
				);
			
			} else {
				if amount 
					// We need to burn
					let imbalance = T::RelayCurrency::burn(amount - current_vdot);
					T::RelayCurrency::settle(
						&T::PalletId::get().into_account(),
						imbalance,
						WithdrawReasons::TRANSFER,
						KeepAlive,
					);
					if amount - current_vdot > 0u32.into() {
						T::RelayCurrency::transfer(
							&T::PalletId::get().into_account(),
							&who,
							amount - current_vdot,
							AllowDeath,
						)?;
					}
				// We need to mint
				T::RelayCurrency::deposit_into_existing(&who, current_vdot - amount)?;
				T::RelayCurrency::transfer(
					&T::PalletId::get().into_account(),
					&who,
					amount,
					AllowDeath,
				)?;
			}

			let staked = StakedMap::<T>::get(who.clone()).ok_or(Error::<T>::NoRewardsAvailable)?;

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn set_ratio(origin: OriginFor<T>, dot_in_sovereign: BalanceOf<T>) -> DispatchResult {
			ensure_root(origin)?;
			let total_staked = TotalStaked::<T>::get();

			// Division by 0
			ensure!(
				total_staked != 0u32.into(),
				Error::<T>::NothingStakedToSetRatio
			);
			let total_issuance: BalanceOf<T> = T::RelayCurrency::total_issuance();
			// The ratio is: the total amount of dots in the sovereign, minus the total issuance of
			// T::RelayCurrency. Those are essentially the dots that were sent to our sovereign but
			// that were not minted in our parachain, i.e., the rewards.
			// The ratio is that difference divided by the total staked
			let difference = dot_in_sovereign - (total_issuance);
			// We should be using the total minted (with multiplier)
			let ratio = U64F64::from_num(difference.saturated_into::<u128>())
				/ U64F64::from_num(total_staked.saturated_into::<u128>());
			Ratio::<T>::put(ratio);
			Self::deposit_event(Event::<T>::RatioSet(difference, total_staked));

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn set_nominations(
			origin: OriginFor<T>,
			nominations: Vec<relay_chain::AccountId>,
		) -> DispatchResult {
			ensure_root(origin)?;
			<Nominations<T>>::put(nominations.clone());
			Self::deposit_event(Event::<T>::NominationsSet(nominations));
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn transact(amount: u128, dest_weight: Weight, call: Vec<u8>) -> Xcm<()> {
			let buy_order = BuyExecution {
				fees: All,
				// Zero weight for additional XCM (since there are none to execute)
				weight: dest_weight,
				debt: dest_weight,
				halt_on_error: false,
				xcm: vec![Transact {
					origin_type: OriginKind::SovereignAccount,
					require_weight_at_most: dest_weight,
					call: call.into(),
				}],
			};

			// We put Null here, as this will be interpreted by the sovereign account
			WithdrawAsset {
				assets: vec![MultiAsset::ConcreteFungible {
					id: MultiLocation::Null,
					amount: amount,
				}],
				effects: vec![buy_order],
			}
		}

		fn send_xcm(
			interior: MultiLocation,
			dest: MultiLocation,
			message: Xcm<()>,
		) -> Result<(), XcmError> {
			let message = match interior {
				MultiLocation::Null => message,
				who => Xcm::<()>::RelayedFrom {
					who,
					message: Box::new(message),
				},
			};
			T::XcmSender::send_xcm(dest, message)
		}
	}
}
