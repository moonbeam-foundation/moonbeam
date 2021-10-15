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

	use xcm::v1::prelude::*;

	use xcm_executor::traits::{InvertLocation, WeightBounds};

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

		// Base XCM weight.
		///
		/// The actually weight for an XCM message is `T::BaseXcmWeight +
		/// T::Weigher::weight(&msg)`.
		#[pallet::constant]
		type BaseXcmWeight: Get<Weight>;
	}

	// The utility calls that need to be implemented as part of
	// this pallet
	#[derive(Debug, PartialEq, Eq)]
	pub enum UtilityAvailableCalls {
		AsDerivative(u16, Vec<u8>),
	}

	// Trait that the ensures we can encode a call with utility functions.
	// With this trait we ensure that the user cannot control entirely the call
	// to be performed in the destination chain. It only can control the call inside
	// the as_derivative extrinsic, and thus, this call can only be dispatched from the
	// derivative account
	pub trait UtilityEncodeCall {
		fn encode_call(self, call: UtilityAvailableCalls) -> Vec<u8>;
	}

	// Trait to ensure we can retrieve the destination of a given type
	// It must implement UtilityEncodeCall
	// We separate this in two traits to be able to implement UtilityEncodeCall separately
	// for different runtimes of our choice
	pub trait XcmTransact: UtilityEncodeCall {
		/// Encode call from the relay.
		fn destination(self) -> MultiLocation;
	}

	#[pallet::storage]
	#[pallet::getter(fn claimed_indices)]
	pub type ClaimedIndices<T: Config> = StorageMap<_, Blake2_128Concat, u16, T::AccountId>;

	#[pallet::storage]
	#[pallet::getter(fn account_to_index)]
	pub type AccountToIndex<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, u16>;

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
		NotAllowed,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		TransactedDerivative(T::AccountId, MultiLocation, Vec<u8>, u16),
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
				ClaimedIndices::<T>::get(&index).is_none(),
				Error::<T>::IndexAlreadyClaimed
			);

			ClaimedIndices::<T>::insert(&index, who.clone());
			AccountToIndex::<T>::insert(&who, index);

			// Deposit event
			Self::deposit_event(Event::<T>::RegisterdDerivative(who, index));

			Ok(())
		}

		/// Transact the inner call through a derivative account in a destination chain,
		/// using 'fee' to pay for the fees
		///
		/// The caller needs to have the index registered in this pallet. The fee multiasset needs
		/// to be a reserve asset for the destination transactor::multilocation.
		#[pallet::weight(Pallet::<T>::weight_of_transact_through_derivative(&fee, &dest, dest_weight, inner_call))]
		pub fn transact_through_derivative(
			origin: OriginFor<T>,
			dest: T::Transactor,
			index: u16,
			fee: MultiAsset,
			dest_weight: Weight,
			inner_call: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;

			// The index exists
			let account = ClaimedIndices::<T>::get(index).ok_or(Error::<T>::UnclaimedIndex)?;
			// The derivative index is owned by the origin
			ensure!(account == who, Error::<T>::NotOwner);

			// Gather the destination
			let destination = Self::transfer_kind(&fee, &dest.clone().destination())?;

			// Encode call bytes
			// We make sure the inner call is wrapped on a as_derivative dispatchable
			let call_bytes: Vec<u8> =
				dest.encode_call(UtilityAvailableCalls::AsDerivative(index, inner_call));

			// Convert origin to multilocation
			let origin_as_mult = T::AccountIdToMultiLocation::convert(who.clone());

			// Gather the xcm call
			let mut xcm: Xcm<T::Call> = Self::transact_fee_in_dest_chain_asset(
				destination.clone(),
				fee,
				dest_weight,
				OriginKind::SovereignAccount,
				call_bytes.clone(),
			)?;

			let weight =
				T::Weigher::weight(&mut xcm).map_err(|()| Error::<T>::UnweighableMessage)?;
			let outcome =
				T::XcmExecutor::execute_xcm_in_credit(origin_as_mult, xcm, weight, weight);

			let maybe_xcm_err: Option<XcmError> = match outcome {
				Outcome::Complete(_w) => Option::None,
				Outcome::Incomplete(_w, err) => Some(err),
				Outcome::Error(err) => Some(err),
			};
			if let Some(xcm_err) = maybe_xcm_err {
				Self::deposit_event(Event::<T>::TransactFailed(xcm_err));
			} else {
				// Deposit event
				Self::deposit_event(Event::<T>::TransactedDerivative(
					who.clone(),
					destination,
					call_bytes,
					index,
				));
			}

			Ok(())
		}

		/// Transact the call through the sovereign account in a destination chain,
		/// 'fee_payer' pays for the 'fee'
		///
		/// Root callable only
		#[pallet::weight(0)]
		pub fn transact_through_sovereign(
			origin: OriginFor<T>,
			destination: MultiLocation,
			fee_payer: T::AccountId,
			fee: MultiAsset,
			dest_weight: Weight,
			call: Vec<u8>,
		) -> DispatchResult {
			ensure_root(origin)?;

			// Convert origin to multilocation
			let origin_as_mult = T::AccountIdToMultiLocation::convert(fee_payer.clone());

			// Gather the xcm call
			let mut xcm: Xcm<T::Call> = Self::transact_fee_in_dest_chain_asset(
				destination.clone(),
				fee,
				dest_weight,
				OriginKind::SovereignAccount,
				call.clone(),
			)?;

			let weight =
				T::Weigher::weight(&mut xcm).map_err(|()| Error::<T>::UnweighableMessage)?;
			let outcome =
				T::XcmExecutor::execute_xcm_in_credit(origin_as_mult, xcm, weight, weight);

			let maybe_xcm_err: Option<XcmError> = match outcome {
				Outcome::Complete(_w) => Option::None,
				Outcome::Incomplete(_w, err) => Some(err),
				Outcome::Error(err) => Some(err),
			};
			if let Some(xcm_err) = maybe_xcm_err {
				Self::deposit_event(Event::<T>::TransactFailed(xcm_err));
			} else {
				// Deposit event
				Self::deposit_event(Event::<T>::TransactedSovereign(
					fee_payer.clone(),
					destination,
					call,
				));
			}

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Construct the transact xcm message with the provided parameters
		fn transact_fee_in_dest_chain_asset(
			dest: MultiLocation,
			asset: MultiAsset,
			dest_weight: Weight,
			origin_kind: OriginKind,
			call: Vec<u8>,
		) -> Result<Xcm<T::Call>, DispatchError> {
			let transact_instructions: Vec<Xcm<()>> = vec![Transact {
				origin_type: origin_kind,
				require_weight_at_most: dest_weight,
				call: call.into(),
			}];
			let effects: Vec<Order<()>> = vec![Self::buy_execution(
				asset.clone(),
				&dest,
				dest_weight,
				transact_instructions,
			)?];

			Ok(Xcm::WithdrawAsset {
				assets: vec![asset].into(),
				effects: vec![Order::InitiateReserveWithdraw {
					assets: All.into(),
					reserve: dest,
					effects: effects,
				}],
			})
		}

		/// Construct a buy execution xcm order with the provided parameters
		fn buy_execution(
			asset: MultiAsset,
			at: &MultiLocation,
			weight: u64,
			instructions: Vec<Xcm<()>>,
		) -> Result<Order<()>, DispatchError> {
			let inv_at = T::LocationInverter::invert_location(at);
			let fees = asset
				.reanchored(&inv_at)
				.map_err(|_| Error::<T>::CannotReanchor)?;
			Ok(BuyExecution {
				fees,
				weight: weight,
				debt: weight,
				halt_on_error: false,
				instructions: instructions,
			})
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
		/// Returns `Err` if `asset` and `dest` combination doesn't make sense,
		/// else returns `dest`, parachain or relay chain location.
		fn transfer_kind(
			asset: &MultiAsset,
			dest: &MultiLocation,
		) -> Result<MultiLocation, DispatchError> {
			let dest = Self::ensure_valid_dest(dest)?;

			let self_location = T::SelfLocation::get();
			ensure!(dest != self_location, Error::<T>::NotCrossChainTransfer);

			let reserve = asset.reserve().ok_or(Error::<T>::AssetHasNoReserve)?;
			ensure!(reserve == dest, Error::<T>::NotAllowed);

			Ok(dest)
		}

		/// Returns weight of `transact_through_derivative` call.
		fn weight_of_transact_through_derivative(
			asset: &MultiAsset,
			dest: &T::Transactor,
			weight: &u64,
			call: &Vec<u8>,
		) -> Weight {
			if let Ok(mut msg) = Self::transact_fee_in_dest_chain_asset(
				dest.clone().destination(),
				asset.clone(),
				weight.clone(),
				OriginKind::SovereignAccount,
				call.clone(),
			) {
				T::Weigher::weight(&mut msg).map_or(Weight::max_value(), |w| {
					T::BaseXcmWeight::get().saturating_add(w)
				})
			} else {
				0
			}
		}
	}
}
