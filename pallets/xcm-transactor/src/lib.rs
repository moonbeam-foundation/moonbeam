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

//! # Xcm Transactor Module
//!
//! ## Overview
//!
//! Module to provide transact capabilitise in other chains
//!
//! This module the transactions are dispatched from a derivative account
//! of the sovereign account
//! This module only stores the index of the derivative account used, but
//! not the derivative account itself. The only assumption this trait makes
//! is the existence of the pallet_utility pallet in the destination chain
//! through the XcmTransact trait.
//!
//! All calls will be wrapped around utility::as_derivative. This makes sure
//! the inner call is executed from the derivative account and not the sovereign
//! account itself. This derivative account can be funded by external users to
//! ensure it has enough funds to make the calls
//!
//! Index registration happens through DerivativeAddressRegistrationOrigin.
//! the inner call is executed from the derivative account and not the sovereign
//! account itself. This derivative account can be funded by external users to
//! ensure it has enough funds to make the calls

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet;

pub use pallet::*;
#[cfg(test)]
pub(crate) mod mock;
#[cfg(test)]
mod tests;

#[pallet]
pub mod pallet {

	use frame_support::dispatch::fmt::Debug;

	use frame_support::pallet_prelude::*;
	use frame_system::{ensure_signed, pallet_prelude::*};
	use orml_traits::location::{Parse, Reserve};
	use sp_runtime::traits::{AtLeast32BitUnsigned, Convert};
	use sp_std::prelude::*;

	use xcm::latest::prelude::*;

	use xcm_executor::traits::{InvertLocation, WeightBounds};
	use xcm_primitives::{TransactInfo, UtilityAvailableCalls, UtilityEncodeCall, XcmTransact};

	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// The balance type.
		type Balance: Parameter
			+ Member
			+ AtLeast32BitUnsigned
			+ Default
			+ Copy
			+ MaybeSerializeDeserialize
			+ Into<u128>;

		// XcmTransact needs to be implemented. This type needs to implement
		// utility call encoding and multilocation gathering
		type Transactor: Parameter + Member + Clone + XcmTransact;

		// XcmTransactorInfo
		type XcmTransactorInfo: TransactInfo<MultiLocation>;

		// The origin that is allowed to register derivative address indices
		type DerivativeAddressRegistrationOrigin: EnsureOrigin<Self::Origin>;

		/// XCM executor.
		type XcmExecutor: ExecuteXcm<Self::Call>;

		/// Convert `T::AccountId` to `MultiLocation`.
		type AccountIdToMultiLocation: Convert<Self::AccountId, MultiLocation>;

		/// Means of measuring the weight consumed by an XCM message locally.
		type Weigher: WeightBounds<Self::Call>;

		/// Means of inverting a location.
		type LocationInverter: InvertLocation;

		/// Self chain location.
		#[pallet::constant]
		type SelfLocation: Get<MultiLocation>;

		// The origin that is allowed to dispatch calls from the sovereign account directly
		type SovereignAccountDispatcherOrigin: EnsureOrigin<Self::Origin>;

		/// XCM sender.
		type XcmSender: SendXcm;

		// Base XCM weight.
		///
		/// The actually weight for an XCM message is `T::BaseXcmWeight +
		/// T::Weigher::weight(&msg)`.
		#[pallet::constant]
		type BaseXcmWeight: Get<Weight>;
	}

	// Stores the index to account mapping. These indices are usable as derivative
	// in the relay chain
	#[pallet::storage]
	#[pallet::getter(fn index_to_account)]
	pub type IndexToAccount<T: Config> = StorageMap<_, Blake2_128Concat, u16, T::AccountId>;

	/// An error that can occur while executing the mapping pallet's logic.
	#[pallet::error]
	pub enum Error<T> {
		IndexAlreadyClaimed,
		UnclaimedIndex,
		NotOwner,
		UnweighableMessage,
		CannotReanchor,
		AssetHasNoReserve,
		InvalidDest,
		NotCrossChainTransfer,
		AssetIsNotReserveInDestination,
		DestinationNotInvertible,
		ErrorSending,
		DispatchWeightBiggerThanTotalWeight,
		Overflow,
		TransactorInfoNotSet,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		TransactedDerivative(T::AccountId, MultiLocation, Vec<u8>, u16, MultiAsset),
		TransactedSovereign(T::AccountId, MultiLocation, Vec<u8>),
		RegisterdDerivative(T::AccountId, u16),
		TransactFailed(XcmError),
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		/// Register a derivative index for an account id. Dispatchable by
		/// DerivativeAddressRegistrationOrigin
		///
		/// We do not store the derivative address, but only the index. We do not need to store
		/// the derivative address to issue calls, only the index is enough
		///
		/// For now an index is registered for all possible destinations and not per-destination.
		/// We can change this in the future although it would just make things more complicated
		pub fn register(origin: OriginFor<T>, who: T::AccountId, index: u16) -> DispatchResult {
			T::DerivativeAddressRegistrationOrigin::ensure_origin(origin)?;

			ensure!(
				IndexToAccount::<T>::get(&index).is_none(),
				Error::<T>::IndexAlreadyClaimed
			);

			IndexToAccount::<T>::insert(&index, who.clone());

			// Deposit event
			Self::deposit_event(Event::<T>::RegisterdDerivative(who, index));

			Ok(())
		}

		/// Transact the inner call through a derivative account in a destination chain,
		/// using 'fee' to pay for the fees
		///
		/// The caller needs to have the index registered in this pallet. The fee multiasset needs
		/// to be a reserve asset for the destination transactor::multilocation.
		#[pallet::weight(0)]
		pub fn transact_through_derivative(
			origin: OriginFor<T>,
			dest: T::Transactor,
			index: u16,
			fee_location: MultiLocation,
			dest_weight: Weight,
			inner_call: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;

			// The index exists
			let account = IndexToAccount::<T>::get(index).ok_or(Error::<T>::UnclaimedIndex)?;
			// The derivative index is owned by the origin
			ensure!(account == who, Error::<T>::NotOwner);

			// Encode call bytes
			// We make sure the inner call is wrapped on a as_derivative dispatchable
			let call_bytes: Vec<u8> = dest
				.clone()
				.encode_call(UtilityAvailableCalls::AsDerivative(index, inner_call));

			let dest = dest.clone().destination();

			let transactor_info = T::XcmTransactorInfo::transactor_info(fee_location.clone())
				.ok_or(Error::<T>::TransactorInfoNotSet)?;

			let total_weight = dest_weight
				.checked_add(transactor_info.transact_extra_weight)
				.ok_or(Error::<T>::Overflow)?;

			let amount = transactor_info
				.destination_units_per_second
				.checked_mul(total_weight as u128)
				.ok_or(Error::<T>::Overflow)?;

			let fee = MultiAsset {
				id: Concrete(fee_location),
				fun: Fungible(amount),
			};

			// Ensure the asset is a reserve
			Self::transfer_kind(&fee, &dest)?;

			// Convert origin to multilocation
			let origin_as_mult = T::AccountIdToMultiLocation::convert(who.clone());

			// does not trigger anything outside moonbeam
			let mut withdraw_message = Xcm(vec![WithdrawAsset(fee.clone().into())]);

			let weight = T::Weigher::weight(&mut withdraw_message)
				.map_err(|()| Error::<T>::UnweighableMessage)?;

			// This execution ensures we withdraw assets from the calling account
			let outcome = T::XcmExecutor::execute_xcm_in_credit(
				origin_as_mult,
				withdraw_message,
				weight,
				weight,
			);

			// Let's check if the execution was succesful
			let maybe_xcm_err: Option<XcmError> = match outcome {
				Outcome::Complete(_w) => Option::None,
				Outcome::Incomplete(_w, err) => Some(err),
				Outcome::Error(err) => Some(err),
			};
			if let Some(xcm_err) = maybe_xcm_err {
				Self::deposit_event(Event::<T>::TransactFailed(xcm_err));
			}

			let transact_message: Xcm<()> = Self::transact_in_dest_chain_asset(
				dest.clone(),
				fee.clone(),
				dest_weight
					.checked_add(transactor_info.transact_extra_weight)
					.ok_or(Error::<T>::Overflow)?,
				call_bytes.clone(),
				dest_weight,
			)?;

			// Send to sovereign
			T::XcmSender::send_xcm(dest.clone(), transact_message)
				.map_err(|_| Error::<T>::ErrorSending)?;

			// Deposit event
			Self::deposit_event(Event::<T>::TransactedDerivative(
				who.clone(),
				dest,
				call_bytes,
				index,
				fee,
			));

			Ok(())
		}

		/// Transact the call through the sovereign account in a destination chain,
		/// 'fee_payer' pays for the 'fee'
		///
		/// SovereignAccountDispatcherOrigin callable only
		#[pallet::weight(
			Pallet::<T>::weight_of_transact_through_sovereign(
				&fee,
				&dest,
				dest_weight,
				call,
				dispatch_weight.clone()
			)
		)]
		pub fn transact_through_sovereign(
			origin: OriginFor<T>,
			dest: MultiLocation,
			fee_payer: T::AccountId,
			fee: MultiAsset,
			dest_weight: Weight,
			call: Vec<u8>,
			dispatch_weight: Weight,
		) -> DispatchResult {
			T::SovereignAccountDispatcherOrigin::ensure_origin(origin)?;

			// dest_weight accounts for all the weight that needs to be purchased in the dest chain
			// dispatch weight accounts just for the weight of the transact operation itself
			// dest_weight should include dispatch_weight
			ensure!(
				dispatch_weight < dest_weight,
				Error::<T>::DispatchWeightBiggerThanTotalWeight
			);

			// Convert origin to multilocation
			let origin_as_mult = T::AccountIdToMultiLocation::convert(fee_payer.clone());

			// does not trigger anything outside moonbeam
			let mut withdraw_message = Xcm(vec![WithdrawAsset(fee.clone().into())]);

			let weight = T::Weigher::weight(&mut withdraw_message)
				.map_err(|()| Error::<T>::UnweighableMessage)?;

			// This execution ensures we withdraw assets from the calling account
			let outcome = T::XcmExecutor::execute_xcm_in_credit(
				origin_as_mult,
				withdraw_message,
				weight,
				weight,
			);

			// Let's check if the execution was succesful
			let maybe_xcm_err: Option<XcmError> = match outcome {
				Outcome::Complete(_w) => Option::None,
				Outcome::Incomplete(_w, err) => Some(err),
				Outcome::Error(err) => Some(err),
			};
			if let Some(xcm_err) = maybe_xcm_err {
				Self::deposit_event(Event::<T>::TransactFailed(xcm_err));
			}

			// Gather the xcm call
			let transact_message: Xcm<()> = Self::transact_in_dest_chain_asset(
				dest.clone(),
				fee,
				dest_weight,
				call.clone(),
				dispatch_weight,
			)?;

			// Send to sovereign
			T::XcmSender::send_xcm(dest.clone(), transact_message)
				.map_err(|_| Error::<T>::ErrorSending)?;

			// Deposit event
			Self::deposit_event(Event::<T>::TransactedSovereign(
				fee_payer.clone(),
				dest,
				call,
			));

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Construct the transact xcm message with the provided parameters
		fn transact_in_dest_chain_asset(
			dest: MultiLocation,
			asset: MultiAsset,
			dest_weight: Weight,
			call: Vec<u8>,
			dispatch_weight: Weight,
		) -> Result<Xcm<()>, DispatchError> {
			Ok(Xcm(vec![
				Self::sovereign_withdraw(asset.clone(), &dest)?,
				Self::buy_execution(asset.clone(), &dest, dest_weight)?,
				Transact {
					origin_type: OriginKind::SovereignAccount,
					require_weight_at_most: dispatch_weight,
					call: call.clone().into(),
				},
			]))
		}

		/// Construct a buy execution xcm order with the provided parameters
		fn buy_execution(
			asset: MultiAsset,
			at: &MultiLocation,
			weight: u64,
		) -> Result<Instruction<()>, DispatchError> {
			let inv_at = T::LocationInverter::invert_location(at)
				.map_err(|()| Error::<T>::DestinationNotInvertible)?;
			let fees = asset
				.reanchored(&inv_at)
				.map_err(|_| Error::<T>::CannotReanchor)?;

			Ok(BuyExecution {
				fees,
				weight_limit: WeightLimit::Limited(weight),
			})
		}

		/// Construct a buy execution xcm order with the provided parameters
		fn sovereign_withdraw(
			asset: MultiAsset,
			at: &MultiLocation,
		) -> Result<Instruction<()>, DispatchError> {
			let inv_at = T::LocationInverter::invert_location(at)
				.map_err(|()| Error::<T>::DestinationNotInvertible)?;
			let fees = asset
				.reanchored(&inv_at)
				.map_err(|_| Error::<T>::CannotReanchor)?;

			Ok(WithdrawAsset(fees.into()))
		}

		/// Ensure has the `dest` has chain part and none recipient part.
		fn ensure_valid_dest(dest: &MultiLocation) -> Result<MultiLocation, DispatchError> {
			if let (Some(dest), None) = (dest.chain_part(), dest.non_chain_part()) {
				Ok(dest)
			} else {
				Err(Error::<T>::InvalidDest.into())
			}
		}

		/// Get the transfer kind.
		///
		/// Returns `Err` if `asset` is not a reserved asset of `dest`,
		/// else returns `dest`, parachain or relay chain location.
		fn transfer_kind(
			asset: &MultiAsset,
			dest: &MultiLocation,
		) -> Result<MultiLocation, DispatchError> {
			let dest = Self::ensure_valid_dest(dest)?;

			let self_location = T::SelfLocation::get();
			ensure!(dest != self_location, Error::<T>::NotCrossChainTransfer);

			let reserve = asset.reserve().ok_or(Error::<T>::AssetHasNoReserve)?;

			// We only allow to transact using a reserve asset as fee
			ensure!(reserve == dest, Error::<T>::AssetIsNotReserveInDestination);

			Ok(dest)
		}

		/// Returns weight of `transact_through_derivative` call.
		fn weight_of_transact_through_derivative(
			asset: &MultiAsset,
			index: &u16,
			dest: &T::Transactor,
			weight: &u64,
			call: &Vec<u8>,
			transact_weight: Weight,
		) -> Weight {
			let call_bytes: Vec<u8> =
				dest.clone()
					.encode_call(UtilityAvailableCalls::AsDerivative(
						index.clone(),
						call.clone(),
					));
			if let Ok(msg) = Self::transact_in_dest_chain_asset(
				dest.clone().destination(),
				asset.clone(),
				weight.clone(),
				call_bytes.clone(),
				transact_weight,
			) {
				T::Weigher::weight(&mut msg.into()).map_or(Weight::max_value(), |w| {
					T::BaseXcmWeight::get().saturating_add(w)
				})
			} else {
				0
			}
		}

		/// Returns weight of `transact_through_sovereign call.
		fn weight_of_transact_through_sovereign(
			asset: &MultiAsset,
			dest: &MultiLocation,
			weight: &u64,
			call: &Vec<u8>,
			dispatch_weight: Weight,
		) -> Weight {
			if let Ok(msg) = Self::transact_in_dest_chain_asset(
				dest.clone(),
				asset.clone(),
				weight.clone(),
				call.clone(),
				dispatch_weight,
			) {
				T::Weigher::weight(&mut msg.into()).map_or(Weight::max_value(), |w| {
					T::BaseXcmWeight::get().saturating_add(w)
				})
			} else {
				0
			}
		}
	}
}
