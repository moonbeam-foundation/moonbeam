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
	use frame_support::{
		pallet_prelude::*,
		traits::{Currency, ReservableCurrency},
		PalletId,
	};
	use frame_system::{ensure_signed, pallet_prelude::*};
	use relay_encoder::ProxyEncodeCall;
	use sp_runtime::traits::Convert;
	use sp_runtime::AccountId32;
	use sp_std::prelude::*;

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
	pub struct StakeInfo<T: Config> {
		pub staked_without_ratio: BalanceOf<T>,
		pub staked_with_ratio: BalanceOf<T>,
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
		type CallEncoder: relay_encoder::ProxyEncodeCall;

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

		type AccountKey20Convert: Convert<Self::AccountId, [u8; 20]>;

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
	#[pallet::getter(fn current_nomination)]
	pub type Nominations<T: Config> = StorageValue<_, Vec<relay_chain::AccountId>, ValueQuery>;

	#[pallet::type_value]
	pub fn RatioDefaultValue<T: Config>() -> U64F64 {
		U64F64::from_num(1)
	}

	#[pallet::storage]
	#[pallet::getter(fn total_staked)]
	pub type TotalStaked<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn total_staked_multiplier)]
	pub type TotalStakedMultiplier<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn staked_map)]
	pub type StakedMap<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, StakeInfo<T>>;

	#[pallet::storage]
	#[pallet::getter(fn proxies)]
	pub type Proxies<T: Config> =
		StorageMap<_, Blake2_128Concat, T::RelayChainAccountId, T::AccountId>;

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
			<T as frame_system::Config>::AccountId,
			Vec<T::RelayChainAccountId>,
		),
		Unstaked(<T as frame_system::Config>::AccountId, BalanceOf<T>),
		RatioSet(BalanceOf<T>, BalanceOf<T>),
		NominationsSet(Vec<relay_chain::AccountId>),
		TransferFailed(XcmError),
		Transferred(),
		XcmSent(MultiLocation, Xcm<()>),
		ProxyCreated(T::AccountId, T::RelayChainAccountId, BalanceOf<T>),
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		pub fn transact_relay(
			origin: OriginFor<T>,
			proxy: T::RelayChainAccountId,
			fee: BalanceOf<T>,
			targets: Vec<T::RelayChainAccountId>,
			dest_weight: Weight,
		) -> DispatchResult {
			/* 			let who = ensure_signed(origin)?;

			let nominate_bytes: Vec<u8> = T::CallEncoder::encode_call(
				relay_encoder::AvailableProxyCalls::NominateThroughAnonymousProxy(
					proxy.clone(),
					targets.clone(),
				),
			);

			let mut xcm: Xcm<T::Call> = Self::transact(fee, dest_weight, nominate_bytes);

			let weight =
				T::Weigher::weight(&mut xcm).map_err(|()| Error::<T>::UnweighableMessage)?;
			let executor = MultiLocation::X1(Junction::AccountKey20 {
				network: NetworkId::Any,
				key: T::AccountKey20Convert::convert(who.clone()).clone(),
			});
			let outcome = T::XcmExecutor::execute_xcm_in_credit(executor, xcm, weight, weight);

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
			Self::deposit_event(Event::<T>::Nominated(who.clone(), targets));*/

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn create_proxy(
			origin: OriginFor<T>,
			proxy: relay_encoder::RelayChainProxyType,
			fee: BalanceOf<T>,
			index: u16,
			dest_weight: Weight,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let proxy_bytes: Vec<u8> = T::CallEncoder::encode_call(
				relay_encoder::AvailableProxyCalls::CreateAnonymusProxy(proxy, 0, index),
			);

			let mut xcm: Xcm<T::Call> = Self::transact(
				who.clone(),
				T::CreateProxyDeposit::get() + fee,
				dest_weight,
				proxy_bytes,
			);

			let weight =
				T::Weigher::weight(&mut xcm).map_err(|()| Error::<T>::UnweighableMessage)?;
			let executor = MultiLocation::X1(Junction::AccountKey20 {
				network: NetworkId::Any,
				key: T::AccountKey20Convert::convert(who.clone()).clone(),
			});
			let outcome = T::XcmExecutor::execute_xcm_in_credit(executor, xcm, weight, weight);

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
			let who = ensure_signed(origin)?;
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
						dest: MultiLocation::X1(Junction::AccountKey20 {
							network: NetworkId::Any,
							key: T::AccountKey20Convert::convert(who.clone()).clone(),
						}),
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
			let executor = MultiLocation::X1(Junction::AccountKey20 {
				network: NetworkId::Any,
				key: T::AccountKey20Convert::convert(who).clone(),
			});
			let outcome = T::XcmExecutor::execute_xcm_in_credit(executor, xcm, weight, weight);

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
			who: T::AccountId,
			amount: BalanceOf<T>,
			dest_weight: Weight,
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
						dest: MultiLocation::X1(Junction::AccountKey20 {
							network: NetworkId::Any,
							key: T::AccountKey20Convert::convert(who.clone()).clone(),
						}),
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
					origin_type: OriginKind::SovereignAccount,
					require_weight_at_most: dest_weight,
					call: call.into(),
				}],
			};
			Xcm::WithdrawAsset {
				assets: vec![MultiAsset::ConcreteFungible {
					id: MultiLocation::X1(Parent),
					amount: T::ToRelayChainBalance::convert(amount),
				}],
				effects: vec![Order::InitiateReserveWithdraw {
					assets: vec![MultiAsset::All],
					reserve: MultiLocation::X1(Parent),
					effects: vec![buy_order, return_fees],
				}],
			}
		}

		fn fee_return(who: T::AccountId, dest_weight: Weight) -> Order<()> {
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
						dest: MultiLocation::X1(Junction::AccountKey20 {
							network: NetworkId::Any,
							key: T::AccountKey20Convert::convert(who.clone()).clone(),
						}),
					},
				],
			}
		}
	}
}
