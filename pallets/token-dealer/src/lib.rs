// Copyright 2019-2020 PureStake Inc.
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

//! Cross-chain token transfers

#![cfg_attr(not(feature = "std"), no_std)]

pub mod support;

use frame_support::pallet;

pub use pallet::*;

#[pallet]
pub mod pallet {
	use cumulus_primitives::{relay_chain::Balance as RelayChainBalance, ParaId};
	use frame_support::{pallet_prelude::*, traits::Get};
	use frame_system::pallet_prelude::*;
	use sp_runtime::traits::{AtLeast32BitUnsigned, Convert};
	use sp_std::prelude::*;
	use xcm::v0::{
		Error as XcmError, ExecuteXcm, Junction, MultiAsset, MultiLocation, NetworkId, Order, Xcm,
	};
	use xcm_executor::traits::LocationConversion;

	#[derive(Encode, Decode, Eq, PartialEq, Clone, Copy, RuntimeDebug)]
	/// Identity of chain.
	pub enum ChainId {
		/// The relay chain.
		RelayChain,
		/// A parachain.
		ParaChain(ParaId),
	}

	#[derive(Encode, Decode, Eq, PartialEq, Clone, RuntimeDebug)]
	/// Identity of cross chain currency.
	pub struct XCurrencyId {
		/// The reserve chain of the currency. For instance, the reserve chain
		/// of DOT is Polkadot.
		pub chain_id: ChainId,
		/// The identity of the currency.
		pub currency_id: Vec<u8>,
	}

	impl Into<MultiLocation> for XCurrencyId {
		fn into(self) -> MultiLocation {
			MultiLocation::X1(Junction::GeneralKey(self.currency_id))
		}
	}

	/// Pallet for executing cross-chain transfers
	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	/// The shape of AccountId for (most) substrate chains (not Moonbeam, which is H160 so 20 bytes)
	type SubstrateAccountId = [u8; 32];

	/// Configuration trait of this pallet.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Overarching event type
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// Balances type
		type Balance: Parameter
			+ Member
			+ AtLeast32BitUnsigned
			+ Default
			+ Copy
			+ MaybeSerializeDeserialize
			+ Into<u128>;
		/// Convert local balance into relay chain balance type
		type ToRelayChainBalance: Convert<Self::Balance, RelayChainBalance>;
		/// Convert account to MultiLocation
		type ToMultiLocation: LocationConversion<Self::AccountId>;
		/// Relay chain identifier
		type RelayChainNetworkId: Get<NetworkId>;
		/// Moonbeam parachain identifier
		type ParaId: Get<ParaId>;
		/// XCM Executor
		type Executor: ExecuteXcm;
	}

	#[pallet::event]
	#[pallet::generate_deposit(fn deposit_event)]
	pub enum Event<T: Config> {
		/// Transferred to relay chain. \[src, dest, amount\]
		TransferredToRelayChain(T::AccountId, SubstrateAccountId, T::Balance),
		/// Transfer to relay chain failed. \[src, dest, amount, error\]
		TransferToRelayChainFailed(T::AccountId, SubstrateAccountId, T::Balance, XcmError),
		/// Transferred to parachain. \[x_currency_id, src, para_id, dest, dest_network, amount\]
		TransferredToParachain(
			XCurrencyId,
			T::AccountId,
			ParaId,
			SubstrateAccountId,
			NetworkId,
			T::Balance,
		),
		/// Transfer to parachain failed. \[x_currency_id, src, para_id, dest,
		/// dest_network, amount, error\]
		TransferToParachainFailed(
			XCurrencyId,
			T::AccountId,
			ParaId,
			SubstrateAccountId,
			NetworkId,
			T::Balance,
			XcmError,
		),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Bad location.
		BadLocation,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Transfer relay chain tokens to relay chain.
		#[pallet::weight(10)] // TODO add transactional
		pub fn transfer_to_relay_chain(
			origin: OriginFor<T>,
			dest: SubstrateAccountId,
			amount: T::Balance,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			let xcm = Xcm::WithdrawAsset {
				assets: vec![MultiAsset::ConcreteFungible {
					id: MultiLocation::X1(Junction::Parent),
					amount: T::ToRelayChainBalance::convert(amount),
				}],
				effects: vec![Order::InitiateReserveWithdraw {
					assets: vec![MultiAsset::All],
					reserve: MultiLocation::X1(Junction::Parent),
					effects: vec![Order::DepositAsset {
						assets: vec![MultiAsset::All],
						dest: MultiLocation::X1(Junction::AccountId32 {
							network: T::RelayChainNetworkId::get(),
							id: dest.clone(),
						}),
					}],
				}],
			};

			let xcm_origin = T::ToMultiLocation::try_into_location(who.clone())
				.map_err(|_| Error::<T>::BadLocation)?;
			// TODO: revert state on xcm execution failure.
			match T::Executor::execute_xcm(xcm_origin, xcm) {
				Ok(_) => {
					Self::deposit_event(Event::<T>::TransferredToRelayChain(who, dest, amount))
				}
				Err(err) => Self::deposit_event(Event::<T>::TransferToRelayChainFailed(
					who, dest, amount, err,
				)),
			}

			Ok(().into())
		}
		/// Transfer tokens to parachain.
		#[pallet::weight(10)]
		pub fn transfer_to_parachain(
			origin: OriginFor<T>,
			x_currency_id: XCurrencyId,
			para_id: ParaId,
			dest: SubstrateAccountId,
			dest_network: NetworkId,
			amount: T::Balance,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			if para_id == T::ParaId::get() {
				return Ok(().into());
			}

			let xcm = match x_currency_id.chain_id {
				ChainId::RelayChain => Self::transfer_relay_chain_tokens_to_parachain(
					para_id,
					&dest,
					dest_network.clone(),
					amount,
				),
				ChainId::ParaChain(reserve_chain) => {
					if T::ParaId::get() == reserve_chain {
						Self::transfer_owned_tokens_to_parachain(
							x_currency_id.clone(),
							para_id,
							&dest,
							dest_network.clone(),
							amount,
						)
					} else {
						Self::transfer_non_owned_tokens_to_parachain(
							reserve_chain,
							x_currency_id.clone(),
							para_id,
							&dest,
							dest_network.clone(),
							amount,
						)
					}
				}
			};

			let xcm_origin = T::ToMultiLocation::try_into_location(who.clone())
				.map_err(|_| Error::<T>::BadLocation)?;
			// TODO: revert state on xcm execution failure.
			match T::Executor::execute_xcm(xcm_origin, xcm) {
				Ok(_) => Self::deposit_event(Event::<T>::TransferredToParachain(
					x_currency_id,
					who,
					para_id,
					dest,
					dest_network,
					amount,
				)),
				Err(err) => Self::deposit_event(Event::<T>::TransferToParachainFailed(
					x_currency_id,
					who,
					para_id,
					dest,
					dest_network,
					amount,
					err,
				)),
			}

			Ok(().into())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Returns upward message to transfer tokens from relay chain to parachain
		/// with default substrate account type (32 byte public keys)
		fn transfer_relay_chain_tokens_to_parachain(
			para_id: ParaId,
			dest: &SubstrateAccountId,
			dest_network: NetworkId,
			amount: T::Balance,
		) -> Xcm {
			Xcm::WithdrawAsset {
				assets: vec![MultiAsset::ConcreteFungible {
					id: MultiLocation::X1(Junction::Parent),
					amount: T::ToRelayChainBalance::convert(amount),
				}],
				effects: vec![Order::InitiateReserveWithdraw {
					assets: vec![MultiAsset::All],
					reserve: MultiLocation::X1(Junction::Parent),
					effects: vec![Order::DepositReserveAsset {
						assets: vec![MultiAsset::All],
						// `dest` is children parachain(of parent).
						dest: MultiLocation::X1(Junction::Parachain { id: para_id.into() }),
						effects: vec![Order::DepositAsset {
							assets: vec![MultiAsset::All],
							dest: MultiLocation::X1(Junction::AccountId32 {
								network: dest_network,
								id: dest.clone(),
							}),
						}],
					}],
				}],
			}
		}
		/// Transfer parachain tokens "owned" by self parachain to another
		/// parachain.
		///
		/// NOTE - `para_id` must not be self parachain.
		fn transfer_owned_tokens_to_parachain(
			x_currency_id: XCurrencyId,
			para_id: ParaId,
			dest: &SubstrateAccountId,
			dest_network: NetworkId,
			amount: T::Balance,
		) -> Xcm {
			Xcm::WithdrawAsset {
				assets: vec![MultiAsset::ConcreteFungible {
					id: x_currency_id.into(),
					amount: amount.into(),
				}],
				effects: vec![Order::DepositReserveAsset {
					assets: vec![MultiAsset::All],
					dest: MultiLocation::X2(
						Junction::Parent,
						Junction::Parachain { id: para_id.into() },
					),
					effects: vec![Order::DepositAsset {
						assets: vec![MultiAsset::All],
						dest: MultiLocation::X1(Junction::AccountId32 {
							network: dest_network,
							id: dest.clone(),
						}),
					}],
				}],
			}
		}

		/// Transfer parachain tokens not "owned" by self chain to another
		/// parachain.
		fn transfer_non_owned_tokens_to_parachain(
			reserve_chain: ParaId,
			x_currency_id: XCurrencyId,
			para_id: ParaId,
			dest: &SubstrateAccountId,
			dest_network: NetworkId,
			amount: T::Balance,
		) -> Xcm {
			let deposit_to_dest = Order::DepositAsset {
				assets: vec![MultiAsset::All],
				dest: MultiLocation::X1(Junction::AccountId32 {
					network: dest_network,
					id: dest.clone(),
				}),
			};
			// If transfer to reserve chain, deposit to `dest` on reserve chain,
			// else deposit reserve asset.
			let reserve_chain_order = if para_id == reserve_chain {
				deposit_to_dest
			} else {
				Order::DepositReserveAsset {
					assets: vec![MultiAsset::All],
					dest: MultiLocation::X2(
						Junction::Parent,
						Junction::Parachain { id: para_id.into() },
					),
					effects: vec![deposit_to_dest],
				}
			};

			Xcm::WithdrawAsset {
				assets: vec![MultiAsset::ConcreteFungible {
					id: x_currency_id.into(),
					amount: amount.into(),
				}],
				effects: vec![Order::InitiateReserveWithdraw {
					assets: vec![MultiAsset::All],
					reserve: MultiLocation::X2(
						Junction::Parent,
						Junction::Parachain {
							id: reserve_chain.into(),
						},
					),
					effects: vec![reserve_chain_order],
				}],
			}
		}
	}
}
