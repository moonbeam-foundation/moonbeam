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
//! In this pallet we will make distinctions between sovereign, derivative accounts and
//! multilocation-based derived accounts. The first is the account the parachain controls
//! in the destination chain, the second is an account derived from the
//! sovereign account itself, e.g., by hashing it with an index, while the third is an account
//! derived from the multilocation of a use in this chain (tipically, hashing the ML).
//! Such distinction is important since we want to keep the integrity of the sovereign account
//!
//! This pallet provides three ways of sending Transact operations to another chain
//!
//! - transact_through_derivative: Transact through an address derived from this chains sovereign
//! 	account in the destination chain. For the transaction to successfully be dispatched in the
//! 	destination chain, pallet-utility needs to be installed and at least paid xcm message
//! 	execution should be allowed (and WithdrawAsset,BuyExecution and Transact messages allowed)
//! 	in the destination chain
//!
//!
//!
//! 	The transactions are dispatched from a derivative account
//! 	of the sovereign account
//! 	This pallet only stores the index of the derivative account used, but
//! 	not the derivative account itself. The only assumption this pallet makes
//! 	is the existence of the pallet_utility pallet in the destination chain
//! 	through the XcmTransact trait.
//!
//! 	All calls will be wrapped around utility::as_derivative. This makes sure
//! 	the inner call is executed from the derivative account and not the sovereign
//! 	account itself.
//!
//! 	Index registration happens through DerivativeAddressRegistrationOrigin.
//! 	This derivative account can be funded by external users to
//! 	ensure it has enough funds to make the calls
//!
//! - transact_through_sovereign: Transact through the sovereign account representing this chain.
//! 	For the transaction to successfully be dispatched in the destination chain, at least paid
//! 	xcm message execution should be allowed (and WithdrawAsset,BuyExecution and Transact
//! 	messages allowed) in the destination chain. Only callable by Root
//!
//! - transact_through_signed: Transact through an account derived from the multilocation
//! 	representing the signed user making the call. We ensure this by prepending DescendOrigin as
//! 	the first instruction of the XCM message. For the transaction to successfully be dispatched
//! 	in the destination chain, at least descended paid xcm message execution should be allowed
//! 	(and DescendOrigin + WithdrawAsset + BuyExecution + Transact messages allowed) in the
//! 	destination chain. Additionally, a ML-based derivation mechanism needs to be implemented
//! 	in the destination chain.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet;

pub use pallet::*;

#[cfg(any(test, feature = "runtime-benchmarks"))]
mod benchmarks;

#[cfg(test)]
pub(crate) mod mock;
#[cfg(test)]
mod tests;

pub mod encode;
pub mod migrations;
pub mod relay_indices;
pub mod weights;
pub use crate::weights::WeightInfo;

type CurrencyIdOf<T> = <T as Config>::CurrencyId;

#[pallet]
pub mod pallet {

	use super::*;
	use crate::relay_indices::RelayChainIndices;
	use crate::weights::WeightInfo;
	use crate::CurrencyIdOf;
	use cumulus_primitives_core::{relay_chain::HrmpChannelId, ParaId};
	use frame_support::{pallet_prelude::*, weights::constants::WEIGHT_REF_TIME_PER_SECOND};
	use frame_system::{ensure_signed, pallet_prelude::*};
	use orml_traits::location::{Parse, Reserve};
	use sp_runtime::traits::{AtLeast32BitUnsigned, Bounded, Convert};
	use sp_std::boxed::Box;
	use sp_std::convert::TryFrom;
	use sp_std::prelude::*;
	use sp_std::vec::Vec;
	use xcm::{latest::prelude::*, VersionedLocation};
	use xcm_executor::traits::{TransactAsset, WeightBounds};
	use xcm_primitives::{
		FilterMaxAssetFee, HrmpAvailableCalls, HrmpEncodeCall, UtilityAvailableCalls,
		UtilityEncodeCall, XcmTransact,
	};

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(pub PhantomData<T>);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
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

		/// Convert `T::CurrencyId` to `Location`.
		type CurrencyIdToLocation: Convert<Self::CurrencyId, Option<Location>>;

		// XcmTransact needs to be implemented. This type needs to implement
		// utility call encoding and multilocation gathering
		type Transactor: Parameter + Member + Clone + XcmTransact;

		/// AssetTransactor allows us to withdraw asset without being trapped
		/// This should change in xcm v3, which allows us to burn assets
		type AssetTransactor: TransactAsset;

		// The origin that is allowed to register derivative address indices
		type DerivativeAddressRegistrationOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		// The origin that is allowed to register derivative address indices
		type HrmpManipulatorOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// Convert `T::AccountId` to `Location`.
		type AccountIdToLocation: Convert<Self::AccountId, Location>;

		/// Means of measuring the weight consumed by an XCM message locally.
		type Weigher: WeightBounds<Self::RuntimeCall>;

		/// This chain's Universal Location.
		type UniversalLocation: Get<InteriorLocation>;

		/// Self chain location.
		#[pallet::constant]
		type SelfLocation: Get<Location>;

		// The origin that is allowed to dispatch calls from the sovereign account directly
		type SovereignAccountDispatcherOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// XCM sender.
		type XcmSender: SendXcm;

		// Base XCM weight.
		///
		/// The actual weight for an XCM message is `T::BaseXcmWeight +
		/// T::Weigher::weight(&msg)`.
		#[pallet::constant]
		type BaseXcmWeight: Get<Weight>;

		/// The way to retrieve the reserve of a Asset. This can be
		/// configured to accept absolute or relative paths for self tokens
		type ReserveProvider: Reserve;

		/// The way to filter the max fee to use for HRMP management operations
		type MaxHrmpFee: FilterMaxAssetFee;

		type WeightInfo: WeightInfo;
	}

	/// Stores the information to be able to issue a transact operation in another chain use an
	/// asset as fee payer.
	#[derive(
		Default,
		Clone,
		Encode,
		Decode,
		MaxEncodedLen,
		RuntimeDebug,
		Eq,
		PartialEq,
		scale_info::TypeInfo,
	)]
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

	/// Enum defining the way to express a Currency.
	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, scale_info::TypeInfo)]
	pub enum Currency<CurrencyId> {
		// Express the Currency as a CurrencyId
		AsCurrencyId(CurrencyId),
		// Express the Currency as its MultiLOcation
		AsMultiLocation(Box<VersionedLocation>),
	}

	impl<T> Default for Currency<T> {
		fn default() -> Currency<T> {
			Currency::<T>::AsMultiLocation(Box::new(Location::default().into()))
		}
	}

	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, scale_info::TypeInfo)]
	pub struct HrmpInitParams {
		pub para_id: ParaId,
		pub proposed_max_capacity: u32,
		pub proposed_max_message_size: u32,
	}

	/// Enum defining the way to express a Currency.
	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, scale_info::TypeInfo)]
	pub enum HrmpOperation {
		InitOpen(HrmpInitParams),
		Accept {
			para_id: ParaId,
		},
		Close(HrmpChannelId),
		Cancel {
			channel_id: HrmpChannelId,
			open_requests: u32,
		},
	}

	#[derive(
		Default,
		Clone,
		Encode,
		Decode,
		Eq,
		PartialEq,
		RuntimeDebug,
		MaxEncodedLen,
		scale_info::TypeInfo,
	)]

	/// Struct that defines how to express the payment in a particular currency
	/// currency is defined by the Currency enum, which can be expressed as:
	/// - CurrencyId
	/// - Location
	///
	/// The fee_amount is an option. In case of None, the fee will be tried to
	/// be calculated from storage. If the storage item for the currency is not
	/// populated, then it fails
	pub struct CurrencyPayment<CurrencyId> {
		// the currency in which we want to express our payment
		pub currency: Currency<CurrencyId>,
		// indicates whether we want to specify the fee amount to be used
		pub fee_amount: Option<u128>,
	}

	#[derive(Default, Clone, Encode, Decode, RuntimeDebug, PartialEq, scale_info::TypeInfo)]
	/// Struct tindicating information about transact weights
	/// It allows to specify:
	/// - transact_required_weight_at_most: the amount of weight the Transact instruction
	///   should consume at most
	/// - overall_weight: the overall weight to be used for the whole XCM message execution.
	///   If None, then this amount will be tried to be derived from storage.  If the storage item
	pub struct TransactWeights {
		// the amount of weight the Transact instruction should consume at most
		pub transact_required_weight_at_most: Weight,
		// the overall weight to be used for the whole XCM message execution. If None,
		// then this amount will be tried to be derived from storage.  If the storage item
		// for the chain is not populated, then it fails
		pub overall_weight: Option<WeightLimit>,
	}

	/// The amount of ref_time and proof_size to use for fee calculation if
	/// we are dealing with an Unlimited variant inside 'overall_weight' field
	/// of 'TransactWeights' struct.
	pub const MAX_WEIGHT: Weight = Weight::from_parts(100_000_000_000, 100_000);

	/// Since we are using pallet-utility for account derivation (through AsDerivative),
	/// we need to provide an index for the account derivation. This storage item stores the index
	/// assigned for a given local account. These indices are usable as derivative in the relay chain
	#[pallet::storage]
	#[pallet::getter(fn index_to_account)]
	pub type IndexToAccount<T: Config> = StorageMap<_, Blake2_128Concat, u16, T::AccountId>;

	/// Stores the transact info of a Location. This defines how much extra weight we need to
	/// add when we want to transact in the destination chain and maximum amount of weight allowed
	/// by the destination chain
	#[pallet::storage]
	#[pallet::getter(fn transact_info)]
	pub type TransactInfoWithWeightLimit<T: Config> =
		StorageMap<_, Blake2_128Concat, Location, RemoteTransactInfoWithMaxWeight>;

	/// Stores the fee per second for an asset in its reserve chain. This allows us to convert
	/// from weight to fee
	#[pallet::storage]
	#[pallet::getter(fn dest_asset_fee_per_second)]
	pub type DestinationAssetFeePerSecond<T: Config> = StorageMap<_, Twox64Concat, Location, u128>;

	/// Stores the indices of relay chain pallets
	#[pallet::storage]
	#[pallet::getter(fn relay_indices)]
	pub type RelayIndices<T: Config> = StorageValue<_, RelayChainIndices, ValueQuery>;

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
		ErrorDelivering,
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
		HrmpHandlerNotImplemented,
		TooMuchFeeUsed,
		ErrorValidating,
		RefundNotSupportedWithTransactInfo,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Transacted the inner call through a derivative account in a destination chain.
		TransactedDerivative {
			account_id: T::AccountId,
			dest: Location,
			call: Vec<u8>,
			index: u16,
		},
		/// Transacted the call through the sovereign account in a destination chain.
		TransactedSovereign {
			fee_payer: Option<T::AccountId>,
			dest: Location,
			call: Vec<u8>,
		},
		/// Transacted the call through a signed account in a destination chain.
		TransactedSigned {
			fee_payer: T::AccountId,
			dest: Location,
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
			location: Location,
			remote_info: RemoteTransactInfoWithMaxWeight,
		},
		/// Removed the transact info of a location
		TransactInfoRemoved {
			location: Location,
		},
		/// Set dest fee per second
		DestFeePerSecondChanged {
			location: Location,
			fee_per_second: u128,
		},
		/// Remove dest fee per second
		DestFeePerSecondRemoved {
			location: Location,
		},
		/// HRMP manage action succesfully sent
		HrmpManagementSent {
			action: HrmpOperation,
		},
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T> {
		pub relay_indices: RelayChainIndices,
		pub _phantom: PhantomData<T>,
	}

	impl<T> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self {
				relay_indices: RelayChainIndices::default(),
				_phantom: Default::default(),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			<RelayIndices<T>>::put(self.relay_indices);
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Register a derivative index for an account id. Dispatchable by
		/// DerivativeAddressRegistrationOrigin
		///
		/// We do not store the derivative address, but only the index. We do not need to store
		/// the derivative address to issue calls, only the index is enough
		///
		/// For now an index is registered for all possible destinations and not per-destination.
		/// We can change this in the future although it would just make things more complicated
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::register())]
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

		/// De-Register a derivative index. This prevents an account to use a derivative address
		/// (represented by an index) from our of our sovereign accounts anymore
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::deregister())]
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
		#[pallet::call_index(2)]
		#[pallet::weight(
			Pallet::<T>::weight_of_initiate_reserve_withdraw()
			.saturating_add(T::WeightInfo::transact_through_derivative())
		)]
		pub fn transact_through_derivative(
			origin: OriginFor<T>,
			// destination to which the message should be sent
			dest: T::Transactor,
			// derivative index to be used
			index: u16,
			// fee to be used
			fee: CurrencyPayment<CurrencyIdOf<T>>,
			// inner call to be executed in destination. This wiol
			// be wrapped into utility.as_derivative
			inner_call: Vec<u8>,
			// weight information to be used
			weight_info: TransactWeights,
			// add RefundSurplus and DepositAsset appendix
			refund: bool,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let fee_location = Self::currency_to_multilocation(fee.currency)
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

			// Calculate the total weight that the xcm message is going to spend in the
			// destination chain
			let total_weight = weight_info.overall_weight.map_or_else(
				|| -> Result<_, DispatchError> {
					let weight_info = Self::take_weight_from_transact_info(
						dest.clone(),
						weight_info.transact_required_weight_at_most,
						refund,
					)?;
					Ok(WeightLimit::from(Some(weight_info)))
				},
				|v| Ok(v),
			)?;

			let total_weight_fee_calculation = match total_weight {
				Unlimited => MAX_WEIGHT,
				Limited(x) => x,
			};

			// Calculate fee based on FeePerSecond
			let fee = Self::calculate_fee(
				fee_location,
				fee.fee_amount,
				dest.clone(),
				total_weight_fee_calculation,
			)?;

			// If refund is true, the appendix instruction will be a deposit back to the sovereign
			let appendix = refund
				.then(|| -> Result<_, DispatchError> {
					Ok(vec![
						RefundSurplus,
						Self::deposit_instruction(T::SelfLocation::get(), &dest, 1u32)?,
					])
				})
				.transpose()?;

			Self::transact_in_dest_chain_asset_non_signed(
				dest.clone(),
				Some(who.clone()),
				fee,
				call_bytes.clone(),
				OriginKind::SovereignAccount,
				total_weight,
				weight_info.transact_required_weight_at_most,
				appendix,
			)?;

			// Deposit event
			Self::deposit_event(Event::<T>::TransactedDerivative {
				account_id: who,
				dest,
				call: call_bytes,
				index,
			});

			Ok(())
		}

		/// Transact the call through the sovereign account in a destination chain,
		/// 'fee_payer' pays for the fee
		///
		/// SovereignAccountDispatcherOrigin callable only
		#[pallet::call_index(3)]
		#[pallet::weight(
			Pallet::<T>::weight_of_initiate_reserve_withdraw()
			.saturating_add(T::WeightInfo::transact_through_sovereign())
		)]
		pub fn transact_through_sovereign(
			origin: OriginFor<T>,
			// destination to which the message should be sent
			dest: Box<VersionedLocation>,
			// account paying for fees
			fee_payer: Option<T::AccountId>,
			// fee to be used
			fee: CurrencyPayment<CurrencyIdOf<T>>,
			// call to be executed in destination
			call: Vec<u8>,
			// origin kind to be used
			origin_kind: OriginKind,
			// weight information to be used
			weight_info: TransactWeights,
			// add RefundSurplus and DepositAsset appendix
			refund: bool,
		) -> DispatchResult {
			T::SovereignAccountDispatcherOrigin::ensure_origin(origin)?;

			let fee_location = Self::currency_to_multilocation(fee.currency)
				.ok_or(Error::<T>::NotCrossChainTransferableCurrency)?;

			let dest = Location::try_from(*dest).map_err(|()| Error::<T>::BadVersion)?;

			// Calculate the total weight that the xcm message is going to spend in the
			// destination chain
			let total_weight = weight_info.overall_weight.map_or_else(
				|| -> Result<_, DispatchError> {
					let weight_info = Self::take_weight_from_transact_info(
						dest.clone(),
						weight_info.transact_required_weight_at_most,
						refund,
					)?;
					Ok(WeightLimit::from(Some(weight_info)))
				},
				|v| Ok(v),
			)?;

			let total_weight_fee_calculation = match total_weight {
				Unlimited => MAX_WEIGHT,
				Limited(x) => x,
			};

			// Calculate fee based on FeePerSecond and total_weight
			let fee = Self::calculate_fee(
				fee_location,
				fee.fee_amount,
				dest.clone(),
				total_weight_fee_calculation,
			)?;

			// If refund is true, the appendix instruction will be a deposit back to the sovereign
			let appendix = refund
				.then(|| -> Result<_, DispatchError> {
					Ok(vec![
						RefundSurplus,
						Self::deposit_instruction(T::SelfLocation::get(), &dest, 1u32)?,
					])
				})
				.transpose()?;

			// Grab the destination
			Self::transact_in_dest_chain_asset_non_signed(
				dest.clone(),
				fee_payer.clone(),
				fee,
				call.clone(),
				origin_kind,
				total_weight,
				weight_info.transact_required_weight_at_most,
				appendix,
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
		#[pallet::call_index(4)]
		#[pallet::weight(T::WeightInfo::set_transact_info())]
		pub fn set_transact_info(
			origin: OriginFor<T>,
			location: Box<VersionedLocation>,
			transact_extra_weight: Weight,
			max_weight: Weight,
			transact_extra_weight_signed: Option<Weight>,
		) -> DispatchResult {
			T::DerivativeAddressRegistrationOrigin::ensure_origin(origin)?;
			let location = Location::try_from(*location).map_err(|()| Error::<T>::BadVersion)?;
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
		#[pallet::call_index(5)]
		#[pallet::weight(T::WeightInfo::remove_transact_info())]
		pub fn remove_transact_info(
			origin: OriginFor<T>,
			location: Box<VersionedLocation>,
		) -> DispatchResult {
			T::DerivativeAddressRegistrationOrigin::ensure_origin(origin)?;
			let location = Location::try_from(*location).map_err(|()| Error::<T>::BadVersion)?;

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
		#[pallet::call_index(6)]
		#[pallet::weight(T::WeightInfo::transact_through_signed())]
		pub fn transact_through_signed(
			origin: OriginFor<T>,
			// destination to which the message should be sent
			dest: Box<VersionedLocation>,
			// fee to be used
			fee: CurrencyPayment<CurrencyIdOf<T>>,
			// call to be executed in destination
			call: Vec<u8>,
			// weight information to be used
			weight_info: TransactWeights,
			// add RefundSurplus and DepositAsset appendix
			refund: bool,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let dest = Location::try_from(*dest).map_err(|()| Error::<T>::BadVersion)?;

			let fee_location = Self::currency_to_multilocation(fee.currency)
				.ok_or(Error::<T>::NotCrossChainTransferableCurrency)?;

			// Calculate the total weight that the xcm message is going to spend in the
			// destination chain
			let total_weight = weight_info.overall_weight.map_or_else(
				|| -> Result<_, DispatchError> {
					let weight_info = Self::take_weight_from_transact_info_signed(
						dest.clone(),
						weight_info.transact_required_weight_at_most,
						refund,
					)?;
					Ok(WeightLimit::from(Some(weight_info)))
				},
				|v| Ok(v),
			)?;

			let total_weight_fee_calculation = match total_weight {
				Unlimited => MAX_WEIGHT,
				Limited(x) => x,
			};

			// Fee to be paid
			let fee = Self::calculate_fee(
				fee_location,
				fee.fee_amount,
				dest.clone(),
				total_weight_fee_calculation,
			)?;

			// If refund is true, the appendix instruction will be a deposit back to the sender
			let appendix = refund
				.then(|| -> Result<_, DispatchError> {
					let sender = T::AccountIdToLocation::convert(who.clone());
					Ok(vec![
						RefundSurplus,
						Self::deposit_instruction(sender, &dest, 1u32)?,
					])
				})
				.transpose()?;

			// Grab the destination
			Self::transact_in_dest_chain_asset_signed(
				dest.clone(),
				who.clone(),
				fee,
				call.clone(),
				OriginKind::SovereignAccount,
				total_weight,
				weight_info.transact_required_weight_at_most,
				appendix,
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
		#[pallet::call_index(7)]
		#[pallet::weight(T::WeightInfo::set_fee_per_second())]
		pub fn set_fee_per_second(
			origin: OriginFor<T>,
			asset_location: Box<VersionedLocation>,
			fee_per_second: u128,
		) -> DispatchResult {
			T::DerivativeAddressRegistrationOrigin::ensure_origin(origin)?;
			let asset_location =
				Location::try_from(*asset_location).map_err(|()| Error::<T>::BadVersion)?;

			DestinationAssetFeePerSecond::<T>::insert(&asset_location, &fee_per_second);

			Self::deposit_event(Event::DestFeePerSecondChanged {
				location: asset_location,
				fee_per_second,
			});
			Ok(())
		}

		/// Remove the fee per second of an asset on its reserve chain
		#[pallet::call_index(8)]
		#[pallet::weight(T::WeightInfo::set_fee_per_second())]
		pub fn remove_fee_per_second(
			origin: OriginFor<T>,
			asset_location: Box<VersionedLocation>,
		) -> DispatchResult {
			T::DerivativeAddressRegistrationOrigin::ensure_origin(origin)?;
			let asset_location =
				Location::try_from(*asset_location).map_err(|()| Error::<T>::BadVersion)?;

			DestinationAssetFeePerSecond::<T>::remove(&asset_location);

			Self::deposit_event(Event::DestFeePerSecondRemoved {
				location: asset_location,
			});
			Ok(())
		}

		/// Manage HRMP operations
		#[pallet::call_index(9)]
		#[pallet::weight(T::WeightInfo::hrmp_manage())]
		pub fn hrmp_manage(
			origin: OriginFor<T>,
			action: HrmpOperation,
			// fee to be used
			fee: CurrencyPayment<CurrencyIdOf<T>>,
			// weight information to be used
			weight_info: TransactWeights,
		) -> DispatchResult {
			// WithdrawAsset
			// BuyExecution
			// SetAppendix(RefundSurplus, DepositAsset(sov account))
			// Transact
			T::HrmpManipulatorOrigin::ensure_origin(origin)?;
			let call_bytes = match action.clone() {
				HrmpOperation::InitOpen(params) => {
					Self::hrmp_encode_call(HrmpAvailableCalls::InitOpenChannel(
						params.para_id,
						params.proposed_max_capacity,
						params.proposed_max_message_size,
					))
				}
				HrmpOperation::Accept { para_id } => {
					Self::hrmp_encode_call(HrmpAvailableCalls::AcceptOpenChannel(para_id))
				}
				HrmpOperation::Close(close_params) => {
					Self::hrmp_encode_call(HrmpAvailableCalls::CloseChannel(close_params))
				}
				HrmpOperation::Cancel {
					channel_id,
					open_requests,
				} => Self::hrmp_encode_call(HrmpAvailableCalls::CancelOpenRequest(
					channel_id,
					open_requests,
				)),
			}
			.map_err(|_| Error::<T>::HrmpHandlerNotImplemented)?;

			let fee_location = Self::currency_to_multilocation(fee.currency)
				.ok_or(Error::<T>::NotCrossChainTransferableCurrency)?;

			// Grab the destination
			// For hrmp, it is always parent
			let destination = Location::parent();

			// Calculate the total weight that the xcm message is going to spend in the
			// destination chain
			let total_weight = weight_info.overall_weight.map_or_else(
				|| -> Result<_, DispatchError> {
					let weight_info = Self::take_weight_from_transact_info(
						destination.clone(),
						weight_info.transact_required_weight_at_most,
						false,
					)?;
					Ok(WeightLimit::from(Some(weight_info)))
				},
				|v| Ok(v),
			)?;

			let total_weight_fee_calculation = match total_weight {
				Unlimited => MAX_WEIGHT,
				Limited(x) => x,
			};

			let fee = Self::calculate_fee(
				fee_location,
				fee.fee_amount,
				destination.clone(),
				total_weight_fee_calculation,
			)?;

			ensure!(
				T::MaxHrmpFee::filter_max_asset_fee(&fee),
				Error::<T>::TooMuchFeeUsed
			);

			// The appendix instruction will be a deposit back to a self location
			let deposit_appendix =
				Self::deposit_instruction(T::SelfLocation::get(), &destination, 1u32)?;

			Self::transact_in_dest_chain_asset_non_signed(
				destination,
				None,
				fee,
				call_bytes.clone(),
				OriginKind::Native,
				total_weight,
				weight_info.transact_required_weight_at_most,
				Some(vec![RefundSurplus, deposit_appendix]),
			)?;

			Self::deposit_event(Event::HrmpManagementSent { action });

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn transact_in_dest_chain_asset_non_signed(
			dest: Location,
			fee_payer: Option<T::AccountId>,
			fee: Asset,
			call: Vec<u8>,
			origin_kind: OriginKind,
			total_weight: WeightLimit,
			transact_required_weight_at_most: Weight,
			with_appendix: Option<Vec<Instruction<()>>>,
		) -> DispatchResult {
			if let Some(fee_payer) = fee_payer {
				// Convert origin to multilocation
				let origin_as_mult = T::AccountIdToLocation::convert(fee_payer);

				// Construct the local withdraw message with the previous calculated amount
				// This message deducts and burns "amount" from the caller when executed
				T::AssetTransactor::withdraw_asset(&fee.clone().into(), &origin_as_mult, None)
					.map_err(|_| Error::<T>::UnableToWithdrawAsset)?;
			}

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
				transact_required_weight_at_most,
				origin_kind,
				with_appendix,
			)?;

			// Send to sovereign
			let (ticket, _price) =
				T::XcmSender::validate(&mut Some(dest), &mut Some(transact_message))
					.map_err(|_| Error::<T>::ErrorValidating)?;
			T::XcmSender::deliver(ticket).map_err(|_| Error::<T>::ErrorDelivering)?;

			Ok(())
		}

		fn transact_in_dest_chain_asset_signed(
			dest: Location,
			fee_payer: T::AccountId,
			fee: Asset,
			call: Vec<u8>,
			origin_kind: OriginKind,
			total_weight: WeightLimit,
			transact_required_weight_at_most: Weight,
			with_appendix: Option<Vec<Instruction<()>>>,
		) -> DispatchResult {
			// Convert origin to multilocation
			let origin_as_mult = T::AccountIdToLocation::convert(fee_payer);

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
				transact_required_weight_at_most,
				origin_kind,
				with_appendix,
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
			let (ticket, _price) =
				T::XcmSender::validate(&mut Some(dest), &mut Some(transact_message))
					.map_err(|_| Error::<T>::ErrorValidating)?;
			T::XcmSender::deliver(ticket).map_err(|_| Error::<T>::ErrorDelivering)?;

			Ok(())
		}

		/// Calculate the amount of fee based on the multilocation of the fee asset and
		/// the total weight to be spent
		fn calculate_fee(
			fee_location: Location,
			fee_amount: Option<u128>,
			destination: Location,
			total_weight: Weight,
		) -> Result<Asset, DispatchError> {
			// If amount is provided, just use it
			// Else, multiply weight*destination_units_per_second to see how much we should charge for
			// this weight execution
			let amount: u128 = fee_amount.map_or_else(
				|| {
					Self::take_fee_per_second_from_storage(
						fee_location.clone(),
						destination,
						total_weight,
					)
				},
				|v| Ok(v),
			)?;

			// Construct Asset
			Ok(Asset {
				id: AssetId(fee_location),
				fun: Fungible(amount),
			})
		}

		/// Construct the transact xcm message with the provided parameters
		fn transact_message(
			dest: Location,
			asset: Asset,
			dest_weight: WeightLimit,
			call: Vec<u8>,
			dispatch_weight: Weight,
			origin_kind: OriginKind,
			with_appendix: Option<Vec<Instruction<()>>>,
		) -> Result<Xcm<()>, DispatchError> {
			let mut instructions = vec![
				Self::withdraw_instruction(asset.clone(), &dest)?,
				Self::buy_execution(asset, &dest, dest_weight)?,
			];
			if let Some(appendix) = with_appendix {
				instructions.push(Self::appendix_instruction(appendix)?);
			}
			instructions.push(Transact {
				origin_kind,
				require_weight_at_most: dispatch_weight,
				call: call.into(),
			});
			Ok(Xcm(instructions))
		}

		/// Construct a buy execution xcm order with the provided parameters
		fn buy_execution(
			asset: Asset,
			at: &Location,
			weight: WeightLimit,
		) -> Result<Instruction<()>, DispatchError> {
			let universal_location = T::UniversalLocation::get();
			let fees = asset
				.reanchored(at, &universal_location)
				.map_err(|_| Error::<T>::CannotReanchor)?;

			Ok(BuyExecution {
				fees,
				weight_limit: weight,
			})
		}

		/// Construct a withdraw instruction from a sovereign account
		fn withdraw_instruction(
			asset: Asset,
			at: &Location,
		) -> Result<Instruction<()>, DispatchError> {
			let universal_location = T::UniversalLocation::get();
			let fees = asset
				.reanchored(at, &universal_location)
				.map_err(|_| Error::<T>::CannotReanchor)?;

			Ok(WithdrawAsset(fees.into()))
		}

		/// Construct a deposit instruction to a sovereign account
		fn deposit_instruction(
			mut beneficiary: Location,
			at: &Location,
			max_assets: u32,
		) -> Result<Instruction<()>, DispatchError> {
			let universal_location = T::UniversalLocation::get();
			beneficiary
				.reanchor(at, &universal_location)
				.map_err(|_| Error::<T>::CannotReanchor)?;
			Ok(DepositAsset {
				assets: Wild(AllCounted(max_assets)),
				beneficiary,
			})
		}

		/// Construct a withdraw instruction from a sovereign account
		fn appendix_instruction(
			instructions: Vec<Instruction<()>>,
		) -> Result<Instruction<()>, DispatchError> {
			Ok(SetAppendix(Xcm(instructions)))
		}

		/// Ensure `dest` has chain part and none recipient part.
		fn ensure_valid_dest(dest: &Location) -> Result<Location, DispatchError> {
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
		fn transfer_allowed(asset: &Asset, dest: &Location) -> Result<Location, DispatchError> {
			let dest = Self::ensure_valid_dest(dest)?;

			let self_location = T::SelfLocation::get();
			ensure!(dest != self_location, Error::<T>::NotCrossChainTransfer);

			let reserve =
				T::ReserveProvider::reserve(asset).ok_or(Error::<T>::AssetHasNoReserve)?;

			// We only allow to transact using a reserve asset as fee
			ensure!(reserve == dest, Error::<T>::AssetIsNotReserveInDestination);

			Ok(dest)
		}

		/// Returns weight of `weight_of_initiate_reserve_withdraw` call.
		fn weight_of_initiate_reserve_withdraw() -> Weight {
			let dest = Location::parent();

			// We can use whatever asset here
			let asset = Location::parent();

			// Construct Asset
			let fee = Asset {
				id: AssetId(asset.clone()),
				fun: Fungible(0),
			};

			let xcm: Xcm<()> = Xcm(vec![
				WithdrawAsset(fee.into()),
				InitiateReserveWithdraw {
					assets: AssetFilter::Wild(All),
					reserve: dest.clone(),
					xcm: Xcm(vec![]),
				},
			]);
			T::Weigher::weight(&mut xcm.into()).map_or(Weight::max_value(), |w| {
				T::BaseXcmWeight::get().saturating_add(w)
			})
		}

		/// Returns the fee for a given set of parameters
		/// We always round up in case of fractional division
		pub fn calculate_fee_per_second(weight: Weight, fee_per_second: u128) -> u128 {
			// grab WEIGHT_REF_TIME_PER_SECOND as u128
			let weight_per_second_u128 = WEIGHT_REF_TIME_PER_SECOND as u128;

			// we add WEIGHT_REF_TIME_PER_SECOND -1 after multiplication to make sure that
			// if there is a fractional part we round up the result
			let fee_mul_rounded_up = (fee_per_second.saturating_mul(weight.ref_time() as u128))
				.saturating_add(weight_per_second_u128 - 1);

			fee_mul_rounded_up / weight_per_second_u128
		}

		/// Returns the weight information for a destination from storage
		/// it returns the weight to be used in non-signed cases
		pub fn take_weight_from_transact_info(
			dest: Location,
			dest_weight: Weight,
			refund: bool,
		) -> Result<Weight, DispatchError> {
			// this is due to TransactInfo only has info of cases where RefundSurplus is not used
			// so we have to ensure 'refund' is false
			ensure!(!refund, Error::<T>::RefundNotSupportedWithTransactInfo);
			// Grab transact info for the destination provided
			let transactor_info = TransactInfoWithWeightLimit::<T>::get(&dest)
				.ok_or(Error::<T>::TransactorInfoNotSet)?;

			let total_weight = dest_weight
				.checked_add(&transactor_info.transact_extra_weight)
				.ok_or(Error::<T>::WeightOverflow)?;

			ensure!(
				total_weight.all_lte(transactor_info.max_weight),
				Error::<T>::MaxWeightTransactReached
			);
			Ok(total_weight)
		}

		/// Returns the weight information for a destination from storage
		/// it returns the weight to be used in signed cases
		pub fn take_weight_from_transact_info_signed(
			dest: Location,
			dest_weight: Weight,
			refund: bool,
		) -> Result<Weight, DispatchError> {
			// this is due to TransactInfo only has info of cases where RefundSurplus is not used
			// so we have to ensure 'refund' is false
			ensure!(!refund, Error::<T>::RefundNotSupportedWithTransactInfo);
			// Grab transact info for the destination provided
			let transactor_info = TransactInfoWithWeightLimit::<T>::get(&dest)
				.ok_or(Error::<T>::TransactorInfoNotSet)?;

			// If this storage item is not set, it means that the destination chain
			// does not support this kind of transact message
			let transact_in_dest_as_signed_weight = transactor_info
				.transact_extra_weight_signed
				.ok_or(Error::<T>::SignedTransactNotAllowedForDestination)?;

			let total_weight = dest_weight
				.checked_add(&transact_in_dest_as_signed_weight)
				.ok_or(Error::<T>::WeightOverflow)?;

			ensure!(
				total_weight.all_lte(transactor_info.max_weight),
				Error::<T>::MaxWeightTransactReached
			);
			Ok(total_weight)
		}

		/// Returns the fee per second charged by a reserve chain for an asset
		/// it takes this information from storage
		pub fn take_fee_per_second_from_storage(
			fee_location: Location,
			destination: Location,
			total_weight: Weight,
		) -> Result<u128, DispatchError> {
			let fee_per_second = DestinationAssetFeePerSecond::<T>::get(&fee_location)
				.ok_or(Error::<T>::FeePerSecondNotSet)?;

			// Ensure the asset is a reserve
			// We only store information about asset fee per second on its reserve chain
			// if amount is provided, we first check whether we have this information
			Self::transfer_allowed(&(fee_location, fee_per_second).into(), &destination)?;

			Ok(Self::calculate_fee_per_second(total_weight, fee_per_second))
		}

		/// Converts Currency to multilocation
		pub fn currency_to_multilocation(currency: Currency<CurrencyIdOf<T>>) -> Option<Location> {
			match currency {
				Currency::AsCurrencyId(id) => T::CurrencyIdToLocation::convert(id),
				Currency::AsMultiLocation(multiloc) => Location::try_from(*multiloc).ok(),
			}
		}
	}
}
