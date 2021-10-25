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
//! Module to provide transact capabilities on other chains
//!
//! In this pallet we will make distinctions between sovereign
//! and derivative accounts. The first is the account the parachain controls
//! in the destination chain, while the latter is an account derived from the
//! sovereign account itself, e.g., by hashing it with an index. Such distinction
//! is important since we want to keep the integrity of the sovereign account
//!
//! The transactions are dispatched from a derivative account
//! of the sovereign account
//! This pallet only stores the index of the derivative account used, but
//! not the derivative account itself. The only assumption this pallet makes
//! is the existence of the pallet_utility pallet in the destination chain
//! through the XcmTransact trait.
//!
//! All calls will be wrapped around utility::as_derivative. This makes sure
//! the inner call is executed from the derivative account and not the sovereign
//! account itself.
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

	use frame_support::weights::constants::WEIGHT_PER_SECOND;

	use frame_support::pallet_prelude::*;
	use frame_system::{ensure_signed, pallet_prelude::*};
	use orml_traits::location::{Parse, Reserve};
	use sp_runtime::traits::{AtLeast32BitUnsigned, Convert};
	use sp_std::prelude::*;

	use xcm::latest::prelude::*;

	use xcm_executor::traits::{InvertLocation, WeightBounds};
	use xcm_primitives::{UtilityAvailableCalls, UtilityEncodeCall, XcmTransact};

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

		/// Currency Id.
		type CurrencyId: Parameter + Member + Clone;

		/// Convert `T::CurrencyId` to `MultiLocation`.
		type CurrencyIdConvert: Convert<Self::CurrencyId, Option<MultiLocation>>;

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

		// The origin that is allowed to dispatch calls from the sovereign account directly
		type SovereignAccountDispatcherOrigin: EnsureOrigin<Self::Origin>;

		/// XCM sender.
		type XcmSender: SendXcm;

		// Base XCM weight.
		///
		/// The actual weight for an XCM message is `T::BaseXcmWeight +
		/// T::Weigher::weight(&msg)`.
		#[pallet::constant]
		type BaseXcmWeight: Get<Weight>;
	}

	/// Stores the information to be able to issue a transact operation in another chain use an
	/// asset as fee payer.
	#[derive(Default, Clone, Encode, Decode, RuntimeDebug, PartialEq, scale_info::TypeInfo)]
	pub struct RemoteTransactInfo {
		/// Extra weight that transacting a call in a destination chain adds
		pub transact_extra_weight: Weight,
		/// Upper bound of units per second that  the destination chain is going to charge for execution
		pub destination_units_per_second: u128,
	}

	// Since we are using pallet-utility for account derivation (through AsDerivative),
	// we need to provide an index for the account derivation. This storage item stores the index
	// assigned for a given local account. These indices are usable as derivative in the relay chain
	#[pallet::storage]
	#[pallet::getter(fn index_to_account)]
	pub type IndexToAccount<T: Config> = StorageMap<_, Blake2_128Concat, u16, T::AccountId>;

	// Stores the transact info of a MULTIlOCAITON. This defines how much extra weight we need to
	// add when we want to transact in the destination chain and how we convert weight to units
	// in the destination chain
	#[pallet::storage]
	#[pallet::getter(fn transact_info)]
	pub type TransactInfo<T: Config> =
		StorageMap<_, Blake2_128Concat, MultiLocation, RemoteTransactInfo>;

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
		WeightOverflow,
		AmountOverflow,
		TransactorInfoNotSet,
		NotCrossChainTransferableCurrency,
		XcmExecuteError,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		TransactedDerivative(T::AccountId, MultiLocation, Vec<u8>, u16),
		TransactedSovereign(T::AccountId, MultiLocation, Vec<u8>),
		RegisterdDerivative(T::AccountId, u16),
		TransactFailed(XcmError),
		TransactInfoChanged(MultiLocation, RemoteTransactInfo),
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
		/// using 'fee_location' to pay for the fees. This fee_location is given as a multilocation
		///
		/// The caller needs to have the index registered in this pallet. The fee multiasset needs
		/// to be a reserve asset for the destination transactor::multilocation.
		#[pallet::weight(Pallet::<T>::weight_of_transact_through_derivative_multilocation(&fee_location, index, &dest, dest_weight, inner_call))]
		pub fn transact_through_derivative_multilocation(
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

			// Grab the destination
			let dest = dest.clone().destination();

			Self::transact_in_dest_chain_asset(
				dest.clone(),
				who.clone(),
				fee_location,
				dest_weight,
				call_bytes.clone(),
			)?;

			// Deposit event
			Self::deposit_event(Event::<T>::TransactedDerivative(
				who, dest, call_bytes, index,
			));

			Ok(())
		}

		/// Transact the inner call through a derivative account in a destination chain,
		/// using 'currency_id' to pay for the fees.
		///
		/// The caller needs to have the index registered in this pallet. The fee multiasset needs
		/// to be a reserve asset for the destination transactor::multilocation.
		#[pallet::weight(Pallet::<T>::weight_of_transact_through_derivative(&currency_id, index, &dest, dest_weight, inner_call))]
		pub fn transact_through_derivative(
			origin: OriginFor<T>,
			dest: T::Transactor,
			index: u16,
			currency_id: T::CurrencyId,
			dest_weight: Weight,
			inner_call: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;

			let fee_location: MultiLocation = T::CurrencyIdConvert::convert(currency_id.clone())
				.ok_or(Error::<T>::NotCrossChainTransferableCurrency)?;

			// The index exists
			let account = IndexToAccount::<T>::get(index).ok_or(Error::<T>::UnclaimedIndex)?;
			// The derivative index is owned by the origin
			ensure!(account == who, Error::<T>::NotOwner);

			// Encode call bytes
			// We make sure the inner call is wrapped on a as_derivative dispatchable
			let call_bytes: Vec<u8> = dest
				.clone()
				.encode_call(UtilityAvailableCalls::AsDerivative(index, inner_call));

			// Grab the destination
			let dest = dest.clone().destination();

			// Grab the destination
			Self::transact_in_dest_chain_asset(
				dest.clone(),
				who.clone(),
				fee_location,
				dest_weight,
				call_bytes.clone(),
			)?;
			// Deposit event
			Self::deposit_event(Event::<T>::TransactedDerivative(
				who, dest, call_bytes, index,
			));

			Ok(())
		}

		/// Transact the call through the sovereign account in a destination chain,
		/// 'fee_payer' pays for the fee
		///
		/// SovereignAccountDispatcherOrigin callable only
		#[pallet::weight(Pallet::<T>::weight_of_transact_through_sovereign(&fee_location, &dest, dest_weight, call))]
		pub fn transact_through_sovereign(
			origin: OriginFor<T>,
			dest: MultiLocation,
			fee_payer: T::AccountId,
			fee_location: MultiLocation,
			dest_weight: Weight,
			call: Vec<u8>,
		) -> DispatchResult {
			T::SovereignAccountDispatcherOrigin::ensure_origin(origin)?;

			// Grab the destination
			Self::transact_in_dest_chain_asset(
				dest.clone(),
				fee_payer.clone(),
				fee_location,
				dest_weight,
				call.clone(),
			)?;

			// Deposit event
			Self::deposit_event(Event::<T>::TransactedSovereign(fee_payer, dest, call));

			Ok(())
		}

		/// Change the transact info of a location
		#[pallet::weight(0)]
		pub fn set_transact_info(
			origin: OriginFor<T>,
			location: MultiLocation,
			transact_extra_weight: Weight,
			destination_units_per_second: u128,
		) -> DispatchResult {
			T::DerivativeAddressRegistrationOrigin::ensure_origin(origin)?;

			let remote_info = RemoteTransactInfo {
				transact_extra_weight,
				destination_units_per_second,
			};

			TransactInfo::<T>::insert(&location, &remote_info);

			Self::deposit_event(Event::TransactInfoChanged(location, remote_info));
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn transact_in_dest_chain_asset(
			dest: MultiLocation,
			fee_payer: T::AccountId,
			fee_location: MultiLocation,
			dest_weight: Weight,
			call: Vec<u8>,
		) -> DispatchResult {
			// Grab transact info for the fee loation provided
			let transactor_info =
				TransactInfo::<T>::get(&fee_location).ok_or(Error::<T>::TransactorInfoNotSet)?;

			// Calculate the total weight that we the xcm message is going to spend in the
			// destination chain
			let total_weight = dest_weight
				.checked_add(transactor_info.transact_extra_weight)
				.ok_or(Error::<T>::WeightOverflow)?;

			// Multiply weight*destination_units_per_second to see how much we should charge for
			// this weight execution
			let amount = transactor_info
				.destination_units_per_second
				.checked_mul(total_weight as u128)
				.ok_or(Error::<T>::AmountOverflow)?
				/ (WEIGHT_PER_SECOND as u128);

			// Construct MultiAsset
			let fee = MultiAsset {
				id: Concrete(fee_location),
				fun: Fungible(amount),
			};

			// Ensure the asset is a reserve
			Self::transfer_allowed(&fee, &dest)?;

			// Convert origin to multilocation
			let origin_as_mult = T::AccountIdToMultiLocation::convert(fee_payer.clone());

			// Construct the local withdraw message with the previous calculated amount
			// This message deducts and burns "amount" from the caller when executed
			let mut withdraw_message = Xcm(vec![WithdrawAsset(fee.clone().into())]);

			// Calculate weight of message
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

			ensure!(maybe_xcm_err.is_none(), Error::<T>::XcmExecuteError);

			// Construct the transact message. This is composed of WithdrawAsset||BuyExecution||
			// Transact.
			// WithdrawAsset: Withdraws "amount" from the sovereign account. This tokens will be
			// used to pay fees
			// BuyExecution: Buys "execution power" in the destination chain
			// Transact: Issues the transaction
			let transact_message: Xcm<()> = Self::transact_message(
				dest.clone(),
				fee.clone(),
				total_weight,
				call.clone(),
				dest_weight,
			)?;

			// Send to sovereign
			T::XcmSender::send_xcm(dest.clone(), transact_message)
				.map_err(|_| Error::<T>::ErrorSending)?;

			Ok(())
		}
		/// Construct the transact xcm message with the provided parameters
		fn transact_message(
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

		/// Construct a withdraw instruction for the sovereign account
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

		/// Check whether the transfer is allowed.
		///
		/// Returns `Err` if `asset` is not a reserved asset of `dest`,
		/// else returns `dest`, parachain or relay chain location.
		fn transfer_allowed(
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
		fn weight_of_transact_through_derivative_multilocation(
			asset: &MultiLocation,
			index: &u16,
			dest: &T::Transactor,
			weight: &u64,
			call: &Vec<u8>,
		) -> Weight {
			let call_bytes: Vec<u8> =
				dest.clone()
					.encode_call(UtilityAvailableCalls::AsDerivative(
						index.clone(),
						call.clone(),
					));
			// Construct MultiAsset
			let fee = MultiAsset {
				id: Concrete(asset.clone()),
				fun: Fungible(0),
			};
			if let Ok(msg) = Self::transact_message(
				dest.clone().destination(),
				fee.clone(),
				weight.clone(),
				call_bytes.clone(),
				weight.clone(),
			) {
				T::Weigher::weight(&mut msg.into()).map_or(Weight::max_value(), |w| {
					T::BaseXcmWeight::get().saturating_add(w)
				})
			} else {
				0
			}
		}

		/// Returns weight of `transact_through_derivative` call.
		fn weight_of_transact_through_derivative(
			currency_id: &T::CurrencyId,
			index: &u16,
			dest: &T::Transactor,
			weight: &u64,
			call: &Vec<u8>,
		) -> Weight {
			if let Some(id) = T::CurrencyIdConvert::convert(currency_id.clone()) {
				Self::weight_of_transact_through_derivative_multilocation(
					&id, &index, &dest, &weight, call,
				)
			} else {
				0
			}
		}

		/// Returns weight of `transact_through_sovereign call.
		fn weight_of_transact_through_sovereign(
			asset: &MultiLocation,
			dest: &MultiLocation,
			weight: &u64,
			call: &Vec<u8>,
		) -> Weight {
			// Construct MultiAsset
			let fee = MultiAsset {
				id: Concrete(asset.clone()),
				fun: Fungible(0),
			};

			if let Ok(msg) = Self::transact_message(
				dest.clone(),
				fee.clone(),
				weight.clone(),
				call.clone(),
				weight.clone(),
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
