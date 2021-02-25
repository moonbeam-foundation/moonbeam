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

use frame_support::pallet;

pub use pallet::*;

#[pallet]
pub mod pallet {
	use cumulus_primitives::ParaId;
	use frame_support::{
		pallet_prelude::*,
		traits::{Currency, Get},
	};
	use frame_system::pallet_prelude::*;
	use sp_runtime::traits::Convert;
	use token_factory::TokenFactory;
	use xcm::v0::{Error as XcmError, ExecuteXcm, Junction::*, MultiAsset, NetworkId, Order, Xcm};
	use xcm_executor::traits::LocationConversion;

	type TokenId = Vec<u8>;
	pub enum CurrencyId {
		/// The local instance of `balances` pallet
		Native,
		/// Token registered in `token-factory` pallet
		Token(TokenId),
	}

	impl From<CurrencyId> for Vec<u8> {
		fn from(other: CurrencyId) -> Vec<u8> {
			match other {
				CurrencyId::Native => b"GLMR".to_vec(),
				CurrencyId::Token(t) => t,
			}
		}
	}
	type BalanceOf<T> = <<T as Config>::NativeCurrency as Currency<
		<T as frame_system::Config>::AccountId,
	>>::Balance;

	/// Pallet for executing cross-chain transfers
	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	/// Configuration trait of this pallet.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Native balances type
		type NativeCurrency: Currency<Self::AccountId>;
		/// Relay chain identifier
		type RelayId: Get<NetworkId>;
		/// Moonbeam's parachain identifier
		type ParaId: Get<ParaId>;
		/// Abstraction over EVM to register, mint, and burn ERC20 tokens
		type TokenFactory: TokenFactory<TokenId, Self::AccountId, BalanceOf<Self>>;
		/// Convert to relay chain account type
		type ToRelayAccount: Convert<Self::AccountId, [u8; 32]>;
		/// XCM Executor
		type Executor: ExecuteXcm;
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {}

	impl<T: Config> Pallet<T> {
		/// Sends message to deposit amount of currency on parachain with `id`
		/// - calling code must reserve amount locally before calling
		/// and burn locally upon finalization on recipient parachain
		fn deposit_to_parachain(
			to_chain: ParaId,
			to_account: T::AccountId,
			network: NetworkId,
			currency: CurrencyId,
			amount: u128,
		) -> Xcm {
			Xcm::WithdrawAsset {
				assets: vec![MultiAsset::ConcreteFungible {
					id: GeneralKey(currency.into()).into(),
					amount,
				}],
				effects: vec![Order::DepositReserveAsset {
					assets: vec![MultiAsset::All],
					dest: (
						Parent,
						Parachain {
							id: to_chain.into(),
						},
					)
						.into(),
					effects: vec![Order::DepositAsset {
						assets: vec![MultiAsset::All],
						dest: AccountId32 {
							network,
							id: T::ToRelayAccount::convert(to_account),
						}
						.into(),
					}],
				}],
			}
		}
	}
}
