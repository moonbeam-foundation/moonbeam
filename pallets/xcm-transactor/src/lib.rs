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

// You should have received a copy of the GNU General Public License
// along with Moonbeam.  If not, see <http://www.gnu.org/licenses/>.

//! # Xcm Transactor Module
//!
//! ## Overview
//!
//! Module to provide transact capabilities on other chains
//!
//! For the transaction to successfully be dispatched in the destination chain, pallet-utility
//! needs to be installed and at least paid xcm message execution should be allowed (and
//! WithdrawAsset,BuyExecution and Transact messages allowed) in the destination chain
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
//! This derivative account can be funded by external users to
//! ensure it has enough funds to make the calls

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet;

pub use pallet::*;

#[cfg(any(test, feature = "runtime-benchmarks"))]
mod benchmarks;

#[cfg(test)]
pub(crate) mod mock;
#[cfg(test)]
mod tests;

pub mod migrations;
pub mod weights;
#[pallet]
pub mod pallet {

	use crate::weights::WeightInfo;
	use frame_support::{pallet_prelude::*, weights::constants::WEIGHT_PER_SECOND};
	use frame_system::{ensure_signed, pallet_prelude::*};
	use orml_traits::location::{Parse, Reserve};
	use sp_runtime::traits::{AtLeast32BitUnsigned, Convert};
	use sp_std::borrow::ToOwned;
	use sp_std::boxed::Box;
	use sp_std::convert::TryFrom;
	use sp_std::prelude::*;
	use xcm::{latest::prelude::*, VersionedMultiLocation};
	use xcm_executor::traits::{InvertLocation, TransactAsset, WeightBounds};
	use xcm_primitives::{UtilityAvailableCalls, UtilityEncodeCall, XcmTransact};

	#[pallet::pallet]
	#[pallet::without_storage_info]
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
		type CurrencyIdToMultiLocation: Convert<Self::CurrencyId, Option<MultiLocation>>;

		// XcmTransact needs to be implemented. This type needs to implement
		// utility call encoding and multilocation gathering
		type Transactor: Parameter + Member + Clone + XcmTransact;

		/// AssetTransactor allows us to withdraw asset without being trapped
		/// This should change in xcm v3, which allows us to burn assets
		type AssetTransactor: TransactAsset;

		// The origin that is allowed to register derivative address indices
		type DerivativeAddressRegistrationOrigin: EnsureOrigin<Self::Origin>;

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

		/// The way to retrieve the reserve of a MultiAsset. This can be
		/// configured to accept absolute or relative paths for self tokens
		type ReserveProvider: Reserve;

		type WeightInfo: WeightInfo;
	}

	/// Stores the information to be able to issue a transact operation in another chain use an
	/// asset as fee payer.
	#[derive(Default, Clone, Encode, Decode, RuntimeDebug, PartialEq, scale_info::TypeInfo)]
	pub struct RemoteTransactInfoWithMaxWeight {
		/// Extra weight that transacting a call in a destination chain adds
		/// Extra weight involved when transacting without DescendOrigin
		/// This should always be possible in a destination chain, since
		/// it involves going through the sovereign account
		pub transact_extra_weight: Weight,
		/// Max destination weight
		pub max_weight: Weight,
		/// Whether we allow transacting through signed origins in another chain, and
		/// how much extra cost implies
		/// Extra weight involved when transacting with DescendOrigin
		/// The reason for it being an option is because the destination chain
		/// might not support constructing origins based on generic MultiLocations
		pub transact_extra_weight_signed: Option<Weight>,
	}

	/// Since we are using pallet-utility for account derivation (through AsDerivative),
	/// we need to provide an index for the account derivation. This storage item stores the index
	/// assigned for a given local account. These indices are usable as derivative in the relay chain
	#[pallet::storage]
	#[pallet::getter(fn index_to_account)]
	pub type IndexToAccount<T: Config> = StorageMap<_, Blake2_128Concat, u16, T::AccountId>;

	/// Stores the transact info of a MultiLocation. This defines how much extra weight we need to
	/// add when we want to transact in the destination chain and maximum amount of weight allowed
	/// by the destination chain
	#[pallet::storage]
	#[pallet::getter(fn transact_info)]
	pub type TransactInfoWithWeightLimit<T: Config> =
		StorageMap<_, Blake2_128Concat, MultiLocation, RemoteTransactInfoWithMaxWeight>;

	/// Stores the fee per second for an asset in its reserve chain. This allows us to convert
	/// from weight to fee
	#[pallet::storage]
	#[pallet::getter(fn dest_asset_fee_per_second)]
	pub type DestinationAssetFeePerSecond<T: Config> =
		StorageMap<_, Twox64Concat, MultiLocation, u128>;

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
		BadVersion,
		MaxWeightTransactReached,
		UnableToWithdrawAsset,
		FeePerSecondNotSet,
		SignedTransactNotAllowedForDestination,
		FailedMultiLocationToJunction,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Transacted the inner call through a derivative account in a destination chain.
		TransactedDerivative {
			account_id: T::AccountId,
			dest: MultiLocation,
			call: Vec<u8>,
			index: u16,
		},
		/// Transacted the call through the sovereign account in a destination chain.
		TransactedSovereign {
			fee_payer: T::AccountId,
			dest: MultiLocation,
			call: Vec<u8>,
		},
		/// Transacted the call through a signed account in a destination chain.
		TransactedSigned {
			fee_payer: T::AccountId,
			dest: MultiLocation,
			call: Vec<u8>,
		},
		/// Registered a derivative index for an account id.
		RegisteredDerivative {
			account_id: T::AccountId,
			index: u16,
		},
		DeRegisteredDerivative {
			index: u16,
		},
		/// Transact failed
		TransactFailed {
			error: XcmError,
		},
		/// Changed the transact info of a location
		TransactInfoChanged {
			location: MultiLocation,
			remote_info: RemoteTransactInfoWithMaxWeight,
		},
		/// Removed the transact info of a location
		TransactInfoRemoved {
			location: MultiLocation,
		},
		/// Set dest fee per second
		DestFeePerSecondChanged {
			location: MultiLocation,
			fee_per_second: u128,
		},
		/// Remove dest fee per second
		DestFeePerSecondRemoved {
			location: MultiLocation,
		},
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(T::WeightInfo::register())]
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
			Self::deposit_event(Event::<T>::RegisteredDerivative {
				account_id: who,
				index: index,
			});

			Ok(())
		}

		#[pallet::weight(T::WeightInfo::deregister())]
		/// De-Register a derivative index. This prevents an account to use a derivative address
		/// (represented by an index) from our of our sovereign accounts anymore
		pub fn deregister(origin: OriginFor<T>, index: u16) -> DispatchResult {
			T::DerivativeAddressRegistrationOrigin::ensure_origin(origin)?;

			// Remove index
			IndexToAccount::<T>::remove(&index);

			// Deposit event
			Self::deposit_event(Event::<T>::DeRegisteredDerivative { index });

			Ok(())
		}

		/// Transact the inner call through a derivative account in a destination chain,
		/// using 'fee_location' to pay for the fees. This fee_location is given as a multilocation
		///
		/// The caller needs to have the index registered in this pallet. The fee multiasset needs
		/// to be a reserve asset for the destination transactor::multilocation.
		#[pallet::weight(
			Pallet::<T>::weight_of_transact_through_derivative_multilocation(
				&fee_location,
				index,
				&dest,
				dest_weight,
				inner_call
			)
		)]
		pub fn transact_through_derivative_multilocation(
			origin: OriginFor<T>,
			dest: T::Transactor,
			index: u16,
			fee_location: Box<VersionedMultiLocation>,
			dest_weight: Weight,
			inner_call: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let fee_location =
				MultiLocation::try_from(*fee_location).map_err(|()| Error::<T>::BadVersion)?;
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
			let dest = dest.destination();

			Self::transact_in_dest_chain_asset_non_signed(
				dest.clone(),
				who.clone(),
				fee_location,
				dest_weight,
				call_bytes.clone(),
				OriginKind::SovereignAccount,
			)?;

			// Deposit event
			Self::deposit_event(Event::<T>::TransactedDerivative {
				account_id: who,
				dest: dest,
				call: call_bytes,
				index: index,
			});

			Ok(())
		}

		/// Transact the inner call through a derivative account in a destination chain,
		/// using 'currency_id' to pay for the fees.
		///
		/// The caller needs to have the index registered in this pallet. The fee multiasset needs
		/// to be a reserve asset for the destination transactor::multilocation.
		#[pallet::weight(
			Pallet::<T>::weight_of_transact_through_derivative(
				&currency_id,
				index,
				&dest,
				dest_weight,
				inner_call
			)
		)]
		pub fn transact_through_derivative(
			origin: OriginFor<T>,
			dest: T::Transactor,
			index: u16,
			currency_id: T::CurrencyId,
			dest_weight: Weight,
			inner_call: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let fee_location: MultiLocation = T::CurrencyIdToMultiLocation::convert(currency_id)
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
			let dest = dest.destination();

			Self::transact_in_dest_chain_asset_non_signed(
				dest.clone(),
				who.clone(),
				fee_location,
				dest_weight,
				call_bytes.clone(),
				OriginKind::SovereignAccount,
			)?;
			// Deposit event
			Self::deposit_event(Event::<T>::TransactedDerivative {
				account_id: who,
				dest: dest,
				call: call_bytes,
				index: index,
			});

			Ok(())
		}

		/// Transact the call through the sovereign account in a destination chain,
		/// 'fee_payer' pays for the fee
		///
		/// SovereignAccountDispatcherOrigin callable only
		#[pallet::weight(
			Pallet::<T>::weight_of_transact_through_sovereign(
				&fee_location,
				&dest,
				dest_weight,
				call,
				*origin_kind
			)
		)]
		pub fn transact_through_sovereign(
			origin: OriginFor<T>,
			dest: Box<VersionedMultiLocation>,
			fee_payer: T::AccountId,
			fee_location: Box<VersionedMultiLocation>,
			dest_weight: Weight,
			call: Vec<u8>,
			origin_kind: OriginKind,
		) -> DispatchResult {
			T::SovereignAccountDispatcherOrigin::ensure_origin(origin)?;

			let fee_location =
				MultiLocation::try_from(*fee_location).map_err(|()| Error::<T>::BadVersion)?;

			let dest = MultiLocation::try_from(*dest).map_err(|()| Error::<T>::BadVersion)?;
			// Grab the destination
			Self::transact_in_dest_chain_asset_non_signed(
				dest.clone(),
				fee_payer.clone(),
				fee_location,
				dest_weight,
				call.clone(),
				origin_kind,
			)?;

			// Deposit event
			Self::deposit_event(Event::<T>::TransactedSovereign {
				fee_payer,
				dest,
				call,
			});

			Ok(())
		}

		/// Change the transact info of a location
		#[pallet::weight(T::WeightInfo::set_transact_info())]
		pub fn set_transact_info(
			origin: OriginFor<T>,
			location: Box<VersionedMultiLocation>,
			transact_extra_weight: Weight,
			max_weight: u64,
			transact_extra_weight_signed: Option<Weight>,
		) -> DispatchResult {
			T::DerivativeAddressRegistrationOrigin::ensure_origin(origin)?;
			let location =
				MultiLocation::try_from(*location).map_err(|()| Error::<T>::BadVersion)?;
			let remote_info = RemoteTransactInfoWithMaxWeight {
				transact_extra_weight,
				max_weight,
				transact_extra_weight_signed,
			};

			TransactInfoWithWeightLimit::<T>::insert(&location, &remote_info);

			Self::deposit_event(Event::TransactInfoChanged {
				location,
				remote_info,
			});
			Ok(())
		}

		/// Remove the transact info of a location
		#[pallet::weight(T::WeightInfo::remove_transact_info())]
		pub fn remove_transact_info(
			origin: OriginFor<T>,
			location: Box<VersionedMultiLocation>,
		) -> DispatchResult {
			T::DerivativeAddressRegistrationOrigin::ensure_origin(origin)?;
			let location =
				MultiLocation::try_from(*location).map_err(|()| Error::<T>::BadVersion)?;

			// Remove transact info
			TransactInfoWithWeightLimit::<T>::remove(&location);

			Self::deposit_event(Event::TransactInfoRemoved { location });
			Ok(())
		}

		/// Transact the call through the a signed origin in this chain
		/// that should be converted to a transaction dispatch account in the destination chain
		/// by any method implemented in the destination chains runtime
		///
		/// This time we are giving the currency as a currencyId instead of multilocation
		#[pallet::weight(T::WeightInfo::transact_through_signed_multilocation())]
		pub fn transact_through_signed(
			origin: OriginFor<T>,
			dest: Box<VersionedMultiLocation>,
			fee_currency_id: T::CurrencyId,
			dest_weight: Weight,
			call: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let dest = MultiLocation::try_from(*dest).map_err(|()| Error::<T>::BadVersion)?;

			let fee_location: MultiLocation =
				T::CurrencyIdToMultiLocation::convert(fee_currency_id)
					.ok_or(Error::<T>::NotCrossChainTransferableCurrency)?;

			// Grab the destination
			Self::transact_in_dest_chain_asset_signed(
				dest.clone(),
				who.clone(),
				fee_location,
				dest_weight,
				call.clone(),
				OriginKind::SovereignAccount,
			)?;

			// Deposit event
			Self::deposit_event(Event::<T>::TransactedSigned {
				fee_payer: who,
				dest,
				call,
			});

			Ok(())
		}

		/// Transact the call through the a signed origin in this chain
		/// that should be converted to a transaction dispatch account in the destination chain
		/// by any method implemented in the destination chains runtime
		///
		/// This time we are giving the currency as a multilocation instead of currencyId
		#[pallet::weight(T::WeightInfo::transact_through_signed_multilocation())]
		pub fn transact_through_signed_multilocation(
			origin: OriginFor<T>,
			dest: Box<VersionedMultiLocation>,
			fee_location: Box<VersionedMultiLocation>,
			dest_weight: Weight,
			call: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let dest = MultiLocation::try_from(*dest).map_err(|()| Error::<T>::BadVersion)?;
			let fee_location =
				MultiLocation::try_from(*fee_location).map_err(|()| Error::<T>::BadVersion)?;

			// Grab the destination
			Self::transact_in_dest_chain_asset_signed(
				dest.clone(),
				who.clone(),
				fee_location,
				dest_weight,
				call.clone(),
				OriginKind::SovereignAccount,
			)?;

			// Deposit event
			Self::deposit_event(Event::<T>::TransactedSigned {
				fee_payer: who,
				dest,
				call,
			});

			Ok(())
		}

		/// Set the fee per second of an asset on its reserve chain
		#[pallet::weight(T::WeightInfo::set_fee_per_second())]
		pub fn set_fee_per_second(
			origin: OriginFor<T>,
			asset_location: Box<VersionedMultiLocation>,
			fee_per_second: u128,
		) -> DispatchResult {
			T::DerivativeAddressRegistrationOrigin::ensure_origin(origin)?;
			let asset_location =
				MultiLocation::try_from(*asset_location).map_err(|()| Error::<T>::BadVersion)?;

			DestinationAssetFeePerSecond::<T>::insert(&asset_location, &fee_per_second);

			Self::deposit_event(Event::DestFeePerSecondChanged {
				location: asset_location,
				fee_per_second,
			});
			Ok(())
		}

		/// Remove the fee per second of an asset on its reserve chain
		#[pallet::weight(T::WeightInfo::set_fee_per_second())]
		pub fn remove_fee_per_second(
			origin: OriginFor<T>,
			asset_location: Box<VersionedMultiLocation>,
		) -> DispatchResult {
			T::DerivativeAddressRegistrationOrigin::ensure_origin(origin)?;
			let asset_location =
				MultiLocation::try_from(*asset_location).map_err(|()| Error::<T>::BadVersion)?;

			DestinationAssetFeePerSecond::<T>::remove(&asset_location);

			Self::deposit_event(Event::DestFeePerSecondRemoved {
				location: asset_location,
			});
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn transact_in_dest_chain_asset_non_signed(
			dest: MultiLocation,
			fee_payer: T::AccountId,
			fee_location: MultiLocation,
			dest_weight: Weight,
			call: Vec<u8>,
			origin_kind: OriginKind,
		) -> DispatchResult {
			// Grab transact info for the fee loation provided
			let transactor_info = TransactInfoWithWeightLimit::<T>::get(&dest)
				.ok_or(Error::<T>::TransactorInfoNotSet)?;

			// Calculate the total weight that the xcm message is going to spend in the
			// destination chain
			let total_weight = dest_weight
				.checked_add(transactor_info.transact_extra_weight)
				.ok_or(Error::<T>::WeightOverflow)?;

			ensure!(
				total_weight < transactor_info.max_weight,
				Error::<T>::MaxWeightTransactReached
			);

			// Calculate fee based on FeePerSecond and total_weight
			let fee = Self::calculate_fee(fee_location, total_weight)?;

			// Ensure the asset is a reserve
			Self::transfer_allowed(&fee, &dest)?;

			// Convert origin to multilocation
			let origin_as_mult = T::AccountIdToMultiLocation::convert(fee_payer);

			// Construct the local withdraw message with the previous calculated amount
			// This message deducts and burns "amount" from the caller when executed
			T::AssetTransactor::withdraw_asset(&fee.clone().into(), &origin_as_mult)
				.map_err(|_| Error::<T>::UnableToWithdrawAsset)?;

			// Construct the transact message. This is composed of WithdrawAsset||BuyExecution||
			// Transact.
			// WithdrawAsset: Withdraws "amount" from the sovereign account. These tokens will be
			// used to pay fees
			// BuyExecution: Buys "execution power" in the destination chain
			// Transact: Issues the transaction
			let transact_message: Xcm<()> = Self::transact_message(
				dest.clone(),
				fee,
				total_weight,
				call,
				dest_weight,
				origin_kind,
			)?;

			// Send to sovereign
			T::XcmSender::send_xcm(dest, transact_message).map_err(|_| Error::<T>::ErrorSending)?;

			Ok(())
		}

		fn transact_in_dest_chain_asset_signed(
			dest: MultiLocation,
			fee_payer: T::AccountId,
			fee_location: MultiLocation,
			dest_weight: Weight,
			call: Vec<u8>,
			origin_kind: OriginKind,
		) -> DispatchResult {
			// Grab transact info for the fee loation provided
			let transactor_info = TransactInfoWithWeightLimit::<T>::get(&dest)
				.ok_or(Error::<T>::TransactorInfoNotSet)?;

			// If this storage item is not set, it means that the destination chain
			// does not support this kind of transact message
			let transact_in_dest_as_signed_weight = transactor_info
				.transact_extra_weight_signed
				.ok_or(Error::<T>::SignedTransactNotAllowedForDestination)?;

			// Calculate the total weight that the xcm message is going to spend in the
			// destination chain
			let total_weight = dest_weight
				.checked_add(transact_in_dest_as_signed_weight)
				.ok_or(Error::<T>::WeightOverflow)?;

			ensure!(
				total_weight < transactor_info.max_weight,
				Error::<T>::MaxWeightTransactReached
			);

			// Calculate fee based on FeePerSecond and total_weight
			let fee = Self::calculate_fee(fee_location, total_weight)?;

			// Convert origin to multilocation
			let origin_as_mult = T::AccountIdToMultiLocation::convert(fee_payer);

			// Construct the transact message. This is composed of WithdrawAsset||BuyExecution||
			// Transact.
			// WithdrawAsset: Withdraws "amount" from the sovereign account. These tokens will be
			// used to pay fees
			// BuyExecution: Buys "execution power" in the destination chain
			// Transact: Issues the transaction
			let mut transact_message: Xcm<()> = Self::transact_message(
				dest.clone(),
				fee,
				total_weight,
				call,
				dest_weight,
				origin_kind,
			)?;

			// We append DescendOrigin as the first instruction in the message
			// The new message looks like DescendOrigin||WithdrawAsset||BuyExecution||
			// Transact.
			let interior: Junctions = origin_as_mult
				.clone()
				.try_into()
				.map_err(|_| Error::<T>::FailedMultiLocationToJunction)?;
			transact_message.0.insert(0, DescendOrigin(interior));

			// Send to destination chain
			T::XcmSender::send_xcm(dest, transact_message).map_err(|_| Error::<T>::ErrorSending)?;

			Ok(())
		}

		/// Calculate the amount of fee based on the multilocation of the fee asset and
		/// the total weight to be spent
		fn calculate_fee(
			fee_location: MultiLocation,
			total_weight: Weight,
		) -> Result<MultiAsset, DispatchError> {
			// Grab how much fee per second the destination chain charges in the fee asset
			// location
			let fee_per_second = DestinationAssetFeePerSecond::<T>::get(&fee_location)
				.ok_or(Error::<T>::FeePerSecondNotSet)?;

			// Multiply weight*destination_units_per_second to see how much we should charge for
			// this weight execution
			let amount = Self::calculate_fee_per_second(total_weight, fee_per_second);

			// Construct MultiAsset
			Ok(MultiAsset {
				id: Concrete(fee_location),
				fun: Fungible(amount),
			})
		}

		/// Construct the transact xcm message with the provided parameters
		fn transact_message(
			dest: MultiLocation,
			asset: MultiAsset,
			dest_weight: Weight,
			call: Vec<u8>,
			dispatch_weight: Weight,
			origin_kind: OriginKind,
		) -> Result<Xcm<()>, DispatchError> {
			Ok(Xcm(vec![
				Self::sovereign_withdraw(asset.clone(), &dest)?,
				Self::buy_execution(asset, &dest, dest_weight)?,
				Transact {
					origin_type: origin_kind,
					require_weight_at_most: dispatch_weight,
					call: call.into(),
				},
			]))
		}

		/// Construct a buy execution xcm order with the provided parameters
		fn buy_execution(
			asset: MultiAsset,
			at: &MultiLocation,
			weight: u64,
		) -> Result<Instruction<()>, DispatchError> {
			let ancestry = T::LocationInverter::ancestry();
			let fees = asset
				.reanchored(at, &ancestry)
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
			let ancestry = T::LocationInverter::ancestry();
			let fees = asset
				.reanchored(at, &ancestry)
				.map_err(|_| Error::<T>::CannotReanchor)?;

			Ok(WithdrawAsset(fees.into()))
		}

		/// Ensure `dest` has chain part and none recipient part.
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

			let reserve =
				T::ReserveProvider::reserve(asset).ok_or(Error::<T>::AssetHasNoReserve)?;

			// We only allow to transact using a reserve asset as fee
			ensure!(reserve == dest, Error::<T>::AssetIsNotReserveInDestination);

			Ok(dest)
		}

		/// Returns weight of `transact_through_derivative` call.
		fn weight_of_transact_through_derivative_multilocation(
			asset: &VersionedMultiLocation,
			index: &u16,
			dest: &T::Transactor,
			weight: &u64,
			call: &[u8],
		) -> Weight {
			// If bad version, return 0
			let asset = if let Ok(asset) = MultiLocation::try_from(asset.clone()) {
				asset
			} else {
				return 0;
			};

			let call_bytes: Vec<u8> =
				dest.clone()
					.encode_call(UtilityAvailableCalls::AsDerivative(
						index.to_owned(),
						call.to_owned(),
					));

			Self::weight_of_transact(
				&asset,
				&dest.clone().destination(),
				weight,
				call_bytes,
				OriginKind::SovereignAccount,
			)
		}

		/// Returns weight of `transact_through_derivative` call.
		fn weight_of_transact_through_derivative(
			currency_id: &T::CurrencyId,
			index: &u16,
			dest: &T::Transactor,
			weight: &u64,
			call: &Vec<u8>,
		) -> Weight {
			if let Some(id) = T::CurrencyIdToMultiLocation::convert(currency_id.clone()) {
				Self::weight_of_transact_through_derivative_multilocation(
					&VersionedMultiLocation::V1(id),
					&index,
					&dest,
					&weight,
					call,
				)
			} else {
				0
			}
		}

		/// Returns weight of `transact_through_sovereign call.
		fn weight_of_transact_through_sovereign(
			asset: &VersionedMultiLocation,
			dest: &VersionedMultiLocation,
			weight: &u64,
			call: &Vec<u8>,
			origin_kind: OriginKind,
		) -> Weight {
			// If asset or dest give errors, return 0
			let (asset, dest) = match (
				MultiLocation::try_from(asset.clone()),
				MultiLocation::try_from(dest.clone()),
			) {
				(Ok(asset), Ok(dest)) => (asset, dest),
				_ => return 0,
			};

			Self::weight_of_transact(&asset, &dest, weight, call.clone(), origin_kind)
		}

		/// Returns weight of transact message.
		fn weight_of_transact(
			asset: &MultiLocation,
			dest: &MultiLocation,
			weight: &u64,
			call: Vec<u8>,
			origin_kind: OriginKind,
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
				origin_kind,
			) {
				T::Weigher::weight(&mut msg.into()).map_or(Weight::max_value(), |w| {
					T::BaseXcmWeight::get().saturating_add(w)
				})
			} else {
				0
			}
		}

		/// Returns the fee for a given set of parameters
		/// We always round up in case of fractional division
		pub fn calculate_fee_per_second(weight: Weight, fee_per_second: u128) -> u128 {
			// grab WEIGHT_PER_SECOND as u128
			let weight_per_second_u128 = WEIGHT_PER_SECOND as u128;

			// we add WEIGHT_PER_SECOND -1 after multiplication to make sure that
			// if there is a fractional part we round up the result
			let fee_mul_rounded_up = (fee_per_second.saturating_mul(weight as u128))
				.saturating_add(weight_per_second_u128 - 1);

			fee_mul_rounded_up / weight_per_second_u128
		}
	}
}
