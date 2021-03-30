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

//! Xtransfer handles cross-chain transfers.

#![cfg_attr(not(feature = "std"), no_std)]

pub mod adapter;
pub mod support;

use frame_support::pallet;

pub use pallet::*;

#[pallet]
pub mod pallet {
	use cumulus_primitives_core::{relay_chain::Balance as RelayChainBalance, ParaId};
	use frame_support::{pallet_prelude::*, traits::Get, transactional};
	use frame_system::pallet_prelude::*;
	use sp_runtime::traits::{AtLeast32BitUnsigned, Convert};
	use sp_std::{convert::Into, prelude::*};
	use token_factory::CurrencyId;
	use xcm::v0::{Junction, MultiAsset, MultiLocation, NetworkId, Order, Xcm};

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

	/// The shape of AccountId for (most) substrate chains (not Moonbeam, which is H160 => 20 bytes)
	type AccountId32 = [u8; 32];

	/// Configuration trait of this pallet.
	#[pallet::config]
	pub trait Config: frame_system::Config + cumulus_pallet_xcm_handler::Config {
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
		/// Convert system::AccountId to shape of inner field for Junction::AccountKey20 [u8; 20]
		type AccountKey20Convert: Convert<Self::AccountId, [u8; 20]>;
		/// Relay chain identifier
		type RelayChainNetworkId: Get<NetworkId>;
		/// This parachain's identifier
		type SelfParaId: Get<ParaId>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(fn deposit_event)]
	pub enum Event<T: Config> {
		/// Transferred to relay chain. \[src, dest, amount\]
		TransferredToRelayChain(T::AccountId, AccountId32, T::Balance),
		/// Transferred to parachain. \[x_currency_id, src, para_id, dest, dest_network, amount\]
		TransferredToAccountId32Parachain(
			XCurrencyId,
			T::AccountId,
			ParaId,
			AccountId32,
			NetworkId,
			T::Balance,
		),
		/// Transferred to parachain. \[x_currency_id, src, para_id, dest, dest_network, amount\]
		TransferredToAccountKey20Parachain(
			XCurrencyId,
			T::AccountId,
			ParaId,
			T::AccountId,
			NetworkId,
			T::Balance,
		),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Cannot send message from parachain to self
		CannotSendToSelf,
		/// Call to SendXcm failed
		FailedToSendXcm,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Transfer relay chain tokens to relay chain.
		/// TODO: check that relay chain token type is registered locally beforehand
		#[pallet::weight(10)]
		#[transactional]
		pub fn transfer_to_relay_chain(
			origin: OriginFor<T>,
			dest: AccountId32,
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
			// TODO: revert state on xcm execution failure.
			<cumulus_pallet_xcm_handler::Pallet<T>>::execute_xcm(who.clone(), xcm)?;
			Self::deposit_event(Event::TransferredToRelayChain(who, dest, amount));
			Ok(().into())
		}
		/// Transfer tokens to parachain that uses [u8; 32] for system::AccountId
		/// TODO: check if channel exists before making any changes
		#[pallet::weight(10)]
		#[transactional]
		pub fn transfer_to_account32_parachain(
			origin: OriginFor<T>,
			x_currency_id: XCurrencyId,
			para_id: ParaId,
			dest: AccountId32,
			dest_network: NetworkId,
			amount: T::Balance,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			ensure!(
				para_id != T::SelfParaId::get(),
				Error::<T>::CannotSendToSelf
			);

			let destination = Self::account_id_32_destination(dest_network.clone(), &dest);

			let xcm = match x_currency_id.chain_id {
				ChainId::RelayChain => {
					Self::transfer_relay_chain_tokens_to_parachain(para_id, destination, amount)
				}
				ChainId::ParaChain(reserve_chain) => {
					if T::SelfParaId::get() == reserve_chain {
						Self::transfer_owned_tokens_to_parachain(
							x_currency_id.clone(),
							para_id,
							destination,
							amount,
						)
					} else {
						Self::transfer_non_owned_tokens_to_parachain(
							reserve_chain,
							x_currency_id.clone(),
							para_id,
							destination,
							amount,
						)
					}
				}
			};
			// TODO: revert state on xcm execution failure.
			<cumulus_pallet_xcm_handler::Pallet<T>>::execute_xcm(who.clone(), xcm)?;
			Self::deposit_event(Event::TransferredToAccountId32Parachain(
				x_currency_id,
				who,
				para_id,
				dest,
				dest_network,
				amount,
			));
			Ok(().into())
		}
		/// Transfer native tokens to other parachain that uses [u8; 20]
		/// TODO: check if channel exists before making any changes
		#[pallet::weight(10)]
		#[transactional]
		pub fn transfer_native_to_account20_parachain(
			origin: OriginFor<T>,
			para_id: ParaId,
			dest: T::AccountId,
			amount: T::Balance,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let self_id = T::SelfParaId::get();
			// TODO: ensure channel exists
			ensure!(para_id != self_id, Error::<T>::CannotSendToSelf);
			let x_currency_id = XCurrencyId {
				chain_id: ChainId::ParaChain(self_id),
				currency_id: CurrencyId::Native.into(),
			};
			let dest_network = T::RelayChainNetworkId::get();
			let destination = Self::account_key_20_destination(dest_network.clone(), dest.clone());
			let xcm = Self::transfer_owned_tokens_to_parachain(
				x_currency_id.clone(),
				para_id,
				destination,
				amount,
			);
			// TODO: revert state on xcm execution failure.
			<cumulus_pallet_xcm_handler::Pallet<T>>::execute_xcm(who.clone(), xcm)?;
			Self::deposit_event(Event::TransferredToAccountKey20Parachain(
				x_currency_id,
				who,
				para_id,
				dest,
				dest_network,
				amount,
			));
			Ok(().into())
		}
		/// Transfer tokens to parachain that uses [u8; 20] for system::AccountId
		/// TODO: check if channel exists before making any changes
		#[pallet::weight(10)]
		#[transactional]
		pub fn transfer_to_account20_parachain(
			origin: OriginFor<T>,
			x_currency_id: XCurrencyId,
			para_id: ParaId,
			dest: T::AccountId,
			dest_network: NetworkId,
			amount: T::Balance,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			ensure!(
				para_id != T::SelfParaId::get(),
				Error::<T>::CannotSendToSelf
			);

			let destination = Self::account_key_20_destination(dest_network.clone(), dest.clone());

			let xcm = match x_currency_id.chain_id {
				ChainId::RelayChain => {
					Self::transfer_relay_chain_tokens_to_parachain(para_id, destination, amount)
				}
				ChainId::ParaChain(reserve_chain) => {
					if T::SelfParaId::get() == reserve_chain {
						Self::transfer_owned_tokens_to_parachain(
							x_currency_id.clone(),
							para_id,
							destination,
							amount,
						)
					} else {
						Self::transfer_non_owned_tokens_to_parachain(
							reserve_chain,
							x_currency_id.clone(),
							para_id,
							destination,
							amount,
						)
					}
				}
			};
			// TODO: revert state on xcm execution failure.
			<cumulus_pallet_xcm_handler::Pallet<T>>::execute_xcm(who.clone(), xcm)?;
			Self::deposit_event(Event::TransferredToAccountKey20Parachain(
				x_currency_id,
				who,
				para_id,
				dest,
				dest_network,
				amount,
			));
			Ok(().into())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Form multilocation when recipient chain uses AccountId32 as system::AccountId type
		fn account_id_32_destination(network: NetworkId, id: &AccountId32) -> MultiLocation {
			MultiLocation::X1(Junction::AccountId32 {
				network,
				id: id.clone(),
			})
		}
		/// Form multilocation when recipient chain uses AccountKey20 as system::AccountId type
		fn account_key_20_destination(network: NetworkId, key: T::AccountId) -> MultiLocation {
			MultiLocation::X1(Junction::AccountKey20 {
				network,
				key: T::AccountKey20Convert::convert(key).clone(),
			})
		}
		/// Returns upward message to transfer tokens from relay chain to parachain
		fn transfer_relay_chain_tokens_to_parachain(
			para_id: ParaId,
			destination: MultiLocation,
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
							dest: destination,
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
			destination: MultiLocation,
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
						dest: destination,
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
			destination: MultiLocation,
			amount: T::Balance,
		) -> Xcm {
			let deposit_to_dest = Order::DepositAsset {
				assets: vec![MultiAsset::All],
				dest: destination,
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
