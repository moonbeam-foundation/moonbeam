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

//! Pallet for registering, minting, and burning ERC20 tokens (in EVM from runtime)

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet;

pub use pallet::*;

#[pallet]
pub mod pallet {
	use frame_support::{pallet_prelude::*, traits::Currency};
	use frame_system::pallet_prelude::*;
	use parity_scale_codec::FullCodec;
	use sp_runtime::{
		traits::{AtLeast32BitUnsigned, Zero},
		DispatchError,
	};
	use sp_std::fmt::Debug;

	/// ERC bytecode
	const CONTRACT_BYTECODE: &str = include_str!("../contract/bytecode.txt");
	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	/// The ERC token factory pallet
	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	/// Configuration trait of this pallet.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_evm::Config {
		/// Overarching event type
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// Balances type
		type Currency: Currency<Self::AccountId>;
		/// ERC token identifier
		type TokenId: Clone + Copy + AtLeast32BitUnsigned + FullCodec + Debug;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Registration fails because ID was already claimed
		IdAlreadyClaimed,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Token register success [token_id, to_address, minted_amount]
		Registered(T::TokenId, T::AccountId, BalanceOf<T>),
		/// Mint token success. [token_id, who, amount]
		Minted(T::TokenId, T::AccountId, BalanceOf<T>),
		/// Burn token success. [token_id, who, amount]
		Burned(T::TokenId, BalanceOf<T>),
		/// Destroy all tokens success. [token_id, amount]
		DestroyedAll(T::TokenId, BalanceOf<T>),
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {}

	#[pallet::storage]
	#[pallet::getter(fn tokens)]
	pub type Tokens<T: Config> = StorageValue<_, Vec<T::TokenId>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn token_info)]
	pub type TokenInfo<T: Config> =
		StorageMap<_, Twox64Concat, T::TokenId, T::AccountId, OptionQuery>;

	pub trait TokenFactory<Id, Account, Balance> {
		fn exists(id: Id) -> bool;
		// TODO: make runtime dispatchable for calling this directly in this pallet, same for `destroy_all`
		fn register(origin: Account, id: Id) -> Result<Account, DispatchError>;
		fn destroy_all(id: Id);
		fn mint(id: Id, who: Account, amount: Balance);
		fn burn(id: Id, who: Account, amount: Balance);
		fn total_issuance(id: Id) -> Balance;
	}

	impl<T: Config> TokenFactory<T::TokenId, T::AccountId, BalanceOf<T>> for Pallet<T> {
		fn exists(id: T::TokenId) -> bool {
			<TokenInfo<T>>::get(id).is_none()
		}
		fn register(origin: T::AccountId, id: T::TokenId) -> Result<T::AccountId, DispatchError> {
			ensure!(!Self::exists(id), Error::<T>::IdAlreadyClaimed);
			//
			// fn default_erc20_creation_unsigned_transaction() -> UnsignedTransaction {
			// 	UnsignedTransaction {
			// 		nonce: U256::zero(),
			// 		gas_price: U256::from(1),
			// 		gas_limit: U256::from(0x100000),
			// 		action: ethereum::TransactionAction::Create,
			// 		value: U256::zero(),
			// 		input: FromHex::from_hex(CONTRACT_BYTECODE).unwrap(),
			// 	}
			// }

			// fn default_erc20_creation_transaction(account: &AccountInfo) -> Transaction {
			// 	default_erc20_creation_unsigned_transaction().sign(&account.private_key)
			// }
			// TODO: assuming we have the bytecode for the contract, how to call its methods
			let deploy = T::AccountId::default();
			// update runtime storage
			<TokenInfo<T>>::insert(id, deploy);
			<Tokens<T>>::mutate(|list| match list.binary_search(&id) {
				Err(loc) => {
					list.insert(loc, id);
				}
				_ => (),
			});
			todo!()
		}
		fn destroy_all(id: T::TokenId) {
			if !Self::exists(id) {
				return;
			}
			// TODO: ethereum transaction to kill contract
			// clean storage
			<TokenInfo<T>>::remove(id);
			<Tokens<T>>::mutate(|list| match list.binary_search(&id) {
				Ok(loc) => {
					list.remove(loc);
				}
				_ => (),
			});
			todo!()
		}
		fn mint(id: T::TokenId, who: T::AccountId, amount: BalanceOf<T>) {
			if !Self::exists(id) {
				return;
			}
			todo!()
		}
		fn burn(id: T::TokenId, who: T::AccountId, amount: BalanceOf<T>) {
			if !Self::exists(id) {
				return;
			}
			todo!()
		}
		fn total_issuance(id: T::TokenId) -> BalanceOf<T> {
			if !Self::exists(id) {
				return BalanceOf::<T>::zero();
			}
			todo!()
		}
	}
}
