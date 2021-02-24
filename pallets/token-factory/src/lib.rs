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
	use ethereum_types::BigEndianHash;
	use frame_support::{pallet_prelude::*, traits::Currency};
	use frame_system::pallet_prelude::*;
	use parity_scale_codec::FullCodec;
	use rustc_hex::FromHex;
	use sp_core::{H160, H256, U256};
	use sp_runtime::{
		traits::{AtLeast32BitUnsigned, Convert, Zero},
		SaturatedConversion,
	};
	use sp_std::fmt::Debug;

	/// ERC20PresetMinterBurner contract bytecode
	const CONTRACT_BYTECODE: &str = include_str!("../contract/bytecode.txt");
	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	/// The ERC token factory pallet
	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	/// Configuration trait of this pallet.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_ethereum::Config + pallet_sudo::Config {
		/// Overarching event type
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// Balances type
		type Currency: Currency<Self::AccountId>;
		/// ERC token identifier
		type TokenId: Clone + Copy + AtLeast32BitUnsigned + FullCodec + Debug;
		/// Convert from AccountId to H160, should be identity map for Moonbeam
		type AccountToH160: Convert<Self::AccountId, H160>;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// TokenID is already claimed
		IdClaimed,
		/// TokenID does not exist so cannot interact with it
		IdNotClaimed,
		/// Require sudo for registering and removing tokens from set
		RequireSudo, // can we use sudo's error type instead of redeclaring it?
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Token register success [token_id, contract_address, to_address, minted_amount]
		Registered(T::TokenId, T::AccountId),
		/// Mint token success. [token_id, who, amount]
		Minted(T::TokenId, T::AccountId, BalanceOf<T>),
		/// Burn token success. [token_id, who, amount]
		Burned(T::TokenId, T::AccountId, BalanceOf<T>),
		/// Destroy all tokens success. [token_id, amount]
		DestroyedAll(T::TokenId, BalanceOf<T>),
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::storage]
	#[pallet::getter(fn tokens)]
	pub type Tokens<T: Config> = StorageValue<_, Vec<T::TokenId>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn token_info)]
	pub type ContractAddress<T: Config> =
		StorageMap<_, Twox64Concat, T::TokenId, T::AccountId, OptionQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		pub fn register_token(origin: OriginFor<T>, id: T::TokenId) -> DispatchResultWithPostInfo {
			let caller = ensure_signed(origin)?;
			ensure!(
				caller == <pallet_sudo::Module<T>>::key(),
				Error::<T>::RequireSudo
			);
			ensure!(!Self::exists(id), Error::<T>::IdClaimed);
			let contract = FromHex::from_hex(CONTRACT_BYTECODE)
				.expect("Static smart contract is formatted incorrectly (should be hex)");
			// TODO: Is there a way to convert from AccountId to H160 w/o trivial associated type??
			let from = T::AccountToH160::convert(caller);
			// deploy contract with default sudo as MINTER_ROLE and BURNER_ROLE (without minting)
			let address = if let (_, Some(addr), _) = <pallet_ethereum::Module<T>>::execute(
				// from: H160
				from,
				// input: Vec<u8>
				contract,
				// value: U256
				U256::zero(),
				// gas limit: U256
				U256::from(0x100000),
				// gas price: U256
				Some(U256::from(1)),
				// nonce: Option<H256>
				Some(U256::zero()),
				// action: TransactionAction
				pallet_ethereum::TransactionAction::Create,
				// config: Option<EvmConfig>
				None,
			)? {
				T::AccountId::decode(&mut &addr[..]).expect("AccountId: H160")
			} else {
				// Rust forces us to cover when no address is returned and execute
				// call succeeds, but this never happens in practice with TransactionAction::Create
				panic!("Unreachable branch; execute never succeeds without deployed address");
			};
			// update runtime storage
			<ContractAddress<T>>::insert(id, address.clone());
			<Tokens<T>>::mutate(|list| {
				if let Err(loc) = list.binary_search(&id) {
					list.insert(loc, id);
				}
			});
			Self::deposit_event(Event::Registered(id, address));
			Ok(().into())
		}

		#[pallet::weight(0)]
		pub fn destroy_all(origin: OriginFor<T>, id: T::TokenId) -> DispatchResultWithPostInfo {
			frame_system::ensure_root(origin)?;
			let _address = <ContractAddress<T>>::get(id).ok_or(Error::<T>::IdNotClaimed)?;
			// TODO: ethereum transaction to remove/kill contract
			// clear storage and free id
			<ContractAddress<T>>::remove(id);
			<Tokens<T>>::mutate(|list| {
				if let Ok(loc) = list.binary_search(&id) {
					list.remove(loc);
				}
			});
			// TODO: get this
			let amount_destroyed = BalanceOf::<T>::zero();
			Self::deposit_event(Event::DestroyedAll(id, amount_destroyed));
			Ok(().into())
		}
	}

	impl<T: Config> Pallet<T> {
		fn sudo_caller() -> H160 {
			T::AccountToH160::convert(<pallet_sudo::Module<T>>::key())
		}
	}

	pub trait TokenFactory<Id, Account, Balance> {
		fn exists(id: Id) -> bool;
		// setters
		fn mint(id: Id, who: Account, amount: Balance);
		fn burn(id: Id, who: Account, amount: Balance);
		// getters
		fn total_issuance(id: Id) -> Balance;
		fn balance_of(id: Id, who: Account) -> Balance;
	}

	impl<T: Config> TokenFactory<T::TokenId, T::AccountId, BalanceOf<T>> for Pallet<T> {
		fn exists(id: T::TokenId) -> bool {
			<ContractAddress<T>>::get(id).is_some()
		}
		fn mint(id: T::TokenId, who: T::AccountId, amount: BalanceOf<T>) {
			if let Some(address) = <ContractAddress<T>>::get(id) {
				// TODO: mint by forming valid ethereum execute call using ABI
				let mut input = hex_literal::hex!("9cff1ade").to_vec();
				// append address
				input.extend_from_slice(
					H256::from(T::AccountToH160::convert(who.clone())).as_bytes(),
				);
				// append amount
				input.extend_from_slice(
					H256::from_uint(&U256::from(amount.saturated_into::<u128>())).as_bytes(),
				);
				// TODO: check if this means execution succeeded...I don't think it does lol
				if let Ok(_) = <pallet_ethereum::Module<T>>::execute(
					// from: H160
					Self::sudo_caller(),
					// input: Vec<u8>
					input,
					// value: U256
					U256::zero(),
					// gas limit: U256
					U256::from(0x100000),
					// gas price: U256
					Some(U256::from(1)),
					// nonce: Option<H256>
					Some(U256::zero()),
					// action: TransactionAction
					pallet_ethereum::TransactionAction::Call(
						// target
						T::AccountToH160::convert(address),
					),
					// config: Option<EvmConfig>
					None,
				) {
					Self::deposit_event(Event::Minted(id, who, amount));
				}
			}
		}
		fn burn(id: T::TokenId, who: T::AccountId, amount: BalanceOf<T>) {
			if let Some(address) = <ContractAddress<T>>::get(id) {
				// TODO: burn by forming valid ethereum execute call using ABI
				let mut input = hex_literal::hex!("4f10869a").to_vec();
				// append address
				input.extend_from_slice(
					H256::from(T::AccountToH160::convert(who.clone())).as_bytes(),
				);
				// append amount
				input.extend_from_slice(
					H256::from_uint(&U256::from(amount.saturated_into::<u128>())).as_bytes(),
				);
				if let Ok(_) = <pallet_ethereum::Module<T>>::execute(
					// from: H160
					Self::sudo_caller(),
					// input: Vec<u8>
					input,
					// value: U256
					U256::zero(),
					// gas limit: U256
					U256::from(0x100000),
					// gas price: U256
					Some(U256::from(1)),
					// nonce: Option<H256>
					Some(U256::zero()),
					// action: TransactionAction
					pallet_ethereum::TransactionAction::Call(
						// target
						T::AccountToH160::convert(address),
					),
					// config: Option<EvmConfig>
					None,
				) {
					Self::deposit_event(Event::Burned(id, who, amount));
				}
			}
		}
		/// Gets total issuance for the given token if it exists in local evm instance
		fn total_issuance(id: T::TokenId) -> BalanceOf<T> {
			if let Some(address) = <ContractAddress<T>>::get(id) {
				// first 4 bytes of hex output of Sha3("totalSupply()")
				let input = hex_literal::hex!("1f1881f8").to_vec();
				if let Ok((_, _, fp_evm::CallOrCreateInfo::Call(result))) =
					<pallet_ethereum::Module<T>>::execute(
						// from: H160
						Self::sudo_caller(),
						// input: Vec<u8>
						input,
						// value: U256
						U256::zero(),
						// gas limit: U256
						U256::from(0x100000),
						// gas price: U256
						Some(U256::from(1)),
						// nonce: Option<H256>
						Some(U256::zero()),
						// action: TransactionAction
						pallet_ethereum::TransactionAction::Call(
							// target
							T::AccountToH160::convert(address),
						),
						// config: Option<EvmConfig>
						None,
					) {
					let value = U256::from(result.value.as_slice()).saturated_into::<u128>();
					return value.saturated_into::<BalanceOf<T>>();
				}
			}
			BalanceOf::<T>::zero()
		}
		/// Gets token balance for the account
		fn balance_of(id: T::TokenId, who: T::AccountId) -> BalanceOf<T> {
			if let Some(address) = <ContractAddress<T>>::get(id) {
				// first 4 bytes of hex output of Sha3("balanceOf(address)")
				let mut input = hex_literal::hex!("1d7976f3").to_vec();
				// append address
				input.extend_from_slice(H256::from(T::AccountToH160::convert(who)).as_bytes());
				if let Ok((_, _, fp_evm::CallOrCreateInfo::Call(result))) =
					<pallet_ethereum::Module<T>>::execute(
						// from: H160
						Self::sudo_caller(),
						// input: Vec<u8>
						input,
						// value: U256
						U256::zero(),
						// gas limit: U256
						U256::from(0x100000),
						// gas price: U256
						Some(U256::from(1)),
						// nonce: Option<H256>
						Some(U256::zero()),
						// action: TransactionAction
						pallet_ethereum::TransactionAction::Call(
							// target
							T::AccountToH160::convert(address),
						),
						// config: Option<EvmConfig>
						None,
					) {
					let value = U256::from(result.value.as_slice()).saturated_into::<u128>();
					return value.saturated_into::<BalanceOf<T>>();
				}
			}
			BalanceOf::<T>::zero()
		}
	}
}
