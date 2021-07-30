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
pub(crate) mod mock;
#[cfg(test)]
mod tests;

#[pallet]
pub mod pallet {

	use cumulus_primitives_core::relay_chain;
	use frame_support::dispatch::fmt::Debug;
	use frame_support::traits::OnGenesis;
	use frame_support::{
		pallet_prelude::*,
		traits::{Currency, ReservableCurrency},
		PalletId,
	};
	use frame_system::{ensure_signed, pallet_prelude::*};
	use relay_encoder::UtilityEncodeCall;
	use sp_io::hashing::blake2_256;
	use sp_runtime::traits::Convert;
	use sp_runtime::AccountId32;
	use sp_std::prelude::*;
	use xcm_executor::traits::Convert as XConvert;

	use substrate_fixed::types::U64F64;
	use xcm::v0::prelude::*;

	use xcm::v0::Junction;
	use xcm_executor::traits::WeightBounds;

	type BalanceOf<T> =
		<<T as Config>::RelayCurrency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	/// Stores info about how many DOTS someone has staked and the relation with the ratio
	#[derive(Default, Clone, Encode, Decode, RuntimeDebug)]
	pub struct DerivativeInfo<T: Config> {
		pub index: u16,
		pub account: T::AccountId,
	}

	/// Configuration trait of this pallet. We tightly couple to Parachain Staking in order to
	/// ensure that only staked accounts can create registrations in the first place. This could be
	/// generalized.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// The currency type for Relay balances
		type RelayCurrency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

		/// Convert local balance into relay chain balance type
		type ToRelayChainBalance: Convert<BalanceOf<Self>, relay_chain::Balance>;

		/// The Pallets PalletId
		type PalletId: Get<PalletId>;

		type RelayChainAccountId: Parameter
			+ Member
			+ MaybeSerializeDeserialize
			+ Ord
			+ Default
			+ Debug
			+ Into<AccountId32>;

		type SovereignAccount: Get<Self::RelayChainAccountId>;

		/// XCM executor.
		type CallEncoder: relay_encoder::UtilityEncodeCall;

		type RelayChainNetworkId: Get<NetworkId>;

		type CreateProxyDeposit: Get<BalanceOf<Self>>;

		type RelayChainProxyType: Parameter
			+ Member
			+ Ord
			+ PartialOrd
			+ Default
			+ Debug
			+ MaxEncodedLen;

		/// XCM executor.
		type XcmExecutor: ExecuteXcm<Self::Call>;

		type OriginToMultiLocation: XConvert<OriginFor<Self>, MultiLocation>;

		/// Means of measuring the weight consumed by an XCM message locally.
		type Weigher: WeightBounds<Self::Call>;

		type OwnLocation: Get<xcm::v0::MultiLocation>;
	}

	/// All possible messages that may be delivered to generic Substrate chain.
	///
	/// Note this enum may be used in the context of both Source (as part of `encode-call`)
	/// and Target chain (as part of `encode-message/send-message`).
	#[derive(Debug, PartialEq, Eq)]
	pub enum AvailableCalls<T: Config> {
		CreateAnonymusProxy(T::RelayChainProxyType, relay_chain::BlockNumber, u16),
		BondThroughAnonymousProxy(T::RelayChainAccountId, relay_chain::Balance),
		NominateThroughAnonymousProxy(T::RelayChainAccountId, Vec<T::RelayChainAccountId>),
	}

	pub trait EncodeCall<T: Config> {
		/// Encode call from the relay.
		fn encode_call(call: AvailableCalls<T>) -> Vec<u8>;
	}

	#[pallet::storage]
	#[pallet::getter(fn current_index)]
	pub type CurrentIndex<T: Config> = StorageValue<_, u16, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn derivatives)]
	pub type Derivatives<T: Config> =
		StorageMap<_, Blake2_128Concat, T::RelayChainAccountId, DerivativeInfo<T>>;

	#[pallet::storage]
	#[pallet::getter(fn queries)]
	pub type Queries<T: Config> = StorageMap<_, Blake2_128Concat, u64, T::AccountId>;

	/// An error that can occur while executing the mapping pallet's logic.
	#[pallet::error]
	pub enum Error<T> {
		MyError,
		WrongConversionU128ToBalance,
		SendFailure,
		ExecuteFailure,
		Overflow,
		NothingStakedToSetRatio,
		NoRewardsAvailable,
		UnstakingMoreThanStaked,
		ProxyAlreadyCreated,
		UnweighableMessage,
		WrongAccountToMUltiLocationConversion,
		IndexInUse,
		AddressListExhausted,
		InvalidRelayAddress,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		Staked(
			<T as frame_system::Config>::AccountId,
			T::RelayChainAccountId,
			BalanceOf<T>,
		),
		Nominated(
			Vec<T::RelayChainAccountId>,
			<T as frame_system::Config>::AccountId,
		),
		Unstaked(<T as frame_system::Config>::AccountId, BalanceOf<T>),
		RatioSet(BalanceOf<T>, BalanceOf<T>),
		NominationsSet(Vec<relay_chain::AccountId>),
		TransferFailed(XcmError),
		Transferred(),
		XcmSent(MultiLocation, Xcm<T::Call>),
		ProxyCreated(T::AccountId, T::RelayChainAccountId, BalanceOf<T>),
		RegisterdDerivative(T::AccountId, T::RelayChainAccountId, u16),
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		pub fn register(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;

			let index = CurrentIndex::<T>::get();

			let derivative = Self::derivative_account_id(index);
			Derivatives::<T>::insert(
				derivative.clone(),
				DerivativeInfo {
					index: index,
					account: who.clone(),
				},
			);
			let next_index = index
				.checked_add(1)
				.ok_or(Error::<T>::AddressListExhausted)?;

			CurrentIndex::<T>::put(next_index);

			// Deposit event
			Self::deposit_event(Event::<T>::RegisterdDerivative(who, derivative, index));

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn transact_through_derivative(
			origin: OriginFor<T>,
			relay_address: T::RelayChainAccountId,
			fee: BalanceOf<T>,
			dest_weight: Weight,
			call: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;
			ensure!(
				Derivatives::<T>::get(relay_address.clone()).is_some(),
				Error::<T>::InvalidRelayAddress
			);
			let info = Derivatives::<T>::get(relay_address).unwrap();
			ensure!(info.account == who, Error::<T>::InvalidRelayAddress);

			let call_bytes: Vec<u8> = T::CallEncoder::encode_call(
				relay_encoder::AvailableUtilityCalls::AsDerivative(info.index, call),
			);

			let origin_as_mult = T::OriginToMultiLocation::convert(origin)
				.map_err(|_e| Error::<T>::WrongAccountToMUltiLocationConversion)?;
			let mut xcm: Xcm<T::Call> = Self::transact(
				origin_as_mult.clone(),
				T::CreateProxyDeposit::get() + fee,
				dest_weight,
				OriginKind::SovereignAccount,
				call_bytes,
			);
			Self::deposit_event(Event::<T>::XcmSent(origin_as_mult.clone(), xcm.clone()));
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
				Self::deposit_event(Event::<T>::TransferFailed(xcm_err));
			} else {
				Self::deposit_event(Event::<T>::Transferred());
			}

			// Deposit event
			Self::deposit_event(Event::<T>::ProxyCreated(
				who.clone(),
				T::SovereignAccount::get(),
				T::CreateProxyDeposit::get(),
			));

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn transact_plain(
			origin: OriginFor<T>,
			fee: BalanceOf<T>,
			origin_kind: OriginKind,
			dest_weight: Weight,
			call: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;

			let origin_as_mult = T::OriginToMultiLocation::convert(origin)
				.map_err(|_e| Error::<T>::WrongAccountToMUltiLocationConversion)?;
			let mut xcm: Xcm<T::Call> = Self::transact(
				origin_as_mult.clone(),
				T::CreateProxyDeposit::get() + fee,
				dest_weight,
				origin_kind,
				call,
			);
			Self::deposit_event(Event::<T>::XcmSent(origin_as_mult.clone(), xcm.clone()));
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
				Self::deposit_event(Event::<T>::TransferFailed(xcm_err));
			} else {
				Self::deposit_event(Event::<T>::Transferred());
			}

			// Deposit event
			Self::deposit_event(Event::<T>::ProxyCreated(
				who.clone(),
				T::SovereignAccount::get(),
				T::CreateProxyDeposit::get(),
			));

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn send_tokens_to_relay(
			origin: OriginFor<T>,
			dest: T::RelayChainAccountId,
			amount: BalanceOf<T>,
			fee: BalanceOf<T>,
			dest_weight: Weight,
		) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;
			let origin_as_mult = T::OriginToMultiLocation::convert(origin)
				.map_err(|_e| Error::<T>::WrongAccountToMUltiLocationConversion)?;

			let relay: AccountId32 = dest.into();
			let buy_order = BuyExecution {
				fees: All,
				// Zero weight for additional XCM (since there are none to execute)
				weight: dest_weight,
				debt: dest_weight,
				halt_on_error: false,
				xcm: vec![],
			};

			let return_fees = Order::DepositReserveAsset {
				assets: vec![MultiAsset::All],
				dest: T::OwnLocation::get(),
				effects: vec![
					BuyExecution {
						fees: All,
						// Zero weight for additional XCM (since there are none to execute)
						weight: 0,
						debt: dest_weight,
						halt_on_error: false,
						xcm: vec![],
					},
					DepositAsset {
						assets: vec![All],
						dest: origin_as_mult.clone(),
					},
				],
			};

			let mut xcm: Xcm<T::Call> = Xcm::WithdrawAsset {
				assets: vec![MultiAsset::ConcreteFungible {
					id: MultiLocation::X1(Parent),
					amount: T::ToRelayChainBalance::convert(amount + fee),
				}],
				effects: vec![Order::InitiateReserveWithdraw {
					assets: vec![MultiAsset::All],
					reserve: MultiLocation::X1(Parent),
					effects: vec![
						buy_order,
						Order::DepositAsset {
							assets: vec![MultiAsset::ConcreteFungible {
								id: MultiLocation::Null,
								amount: T::ToRelayChainBalance::convert(amount),
							}],
							dest: MultiLocation::X1(Junction::AccountId32 {
								network: T::RelayChainNetworkId::get(),
								id: *relay.as_ref(),
							}),
						},
						return_fees,
					],
				}],
			};

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
				Self::deposit_event(Event::<T>::TransferFailed(xcm_err));
			} else {
				Self::deposit_event(Event::<T>::Transferred());
			}

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn transact(
			origin_as_mult: MultiLocation,
			amount: BalanceOf<T>,
			dest_weight: Weight,
			origin_kind: OriginKind,
			call: Vec<u8>,
		) -> Xcm<T::Call> {
			let return_fees = Order::DepositReserveAsset {
				assets: vec![MultiAsset::All],
				dest: T::OwnLocation::get(),
				effects: vec![
					BuyExecution {
						fees: All,
						// Zero weight for additional XCM (since there are none to execute)
						weight: 0,
						debt: dest_weight,
						halt_on_error: false,
						xcm: vec![],
					},
					DepositAsset {
						assets: vec![All],
						dest: origin_as_mult,
					},
				],
			};

			let buy_order = BuyExecution {
				fees: All,
				// Zero weight for additional XCM (since there are none to execute)
				weight: dest_weight,
				debt: dest_weight,
				halt_on_error: false,
				xcm: vec![Transact {
					origin_type: origin_kind,
					require_weight_at_most: dest_weight,
					call: call.into(),
				}],
			};
			let mut effects: Vec<Order<()>> = vec![buy_order, return_fees];

			Xcm::WithdrawAsset {
				assets: vec![MultiAsset::ConcreteFungible {
					id: MultiLocation::X1(Parent),
					amount: T::ToRelayChainBalance::convert(amount),
				}],
				effects: vec![Order::InitiateReserveWithdraw {
					assets: vec![MultiAsset::All],
					reserve: MultiLocation::X1(Parent),
					effects: effects,
				}],
			}
		}

		fn fee_return(who: MultiLocation, dest_weight: Weight) -> Order<()> {
			Order::DepositReserveAsset {
				assets: vec![MultiAsset::All],
				dest: T::OwnLocation::get(),
				effects: vec![
					BuyExecution {
						fees: All,
						// Zero weight for additional XCM (since there are none to execute)
						weight: 0,
						debt: dest_weight,
						halt_on_error: false,
						xcm: vec![],
					},
					DepositAsset {
						assets: vec![All],
						dest: who,
					},
				],
			}
		}

		pub fn derivative_account_id(index: u16) -> T::RelayChainAccountId {
			let entropy =
				(b"modlpy/utilisuba", T::SovereignAccount::get(), index).using_encoded(blake2_256);
			T::RelayChainAccountId::decode(&mut &entropy[..]).unwrap_or_default()
		}
	}
}
