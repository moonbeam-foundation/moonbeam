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

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[pallet]
pub mod pallet {
	use ethereum_types::BigEndianHash;
	use fp_evm::ExecutionInfo;
	use frame_support::{pallet_prelude::*, traits::OriginTrait};
	use frame_system::pallet_prelude::*;
	use pallet_evm::{AddressMapping, ExitReason, Runner};
	use parity_scale_codec::{Decode, Encode, FullCodec};
	use rustc_hex::FromHex;
	#[cfg(feature = "std")]
	use serde::{Deserialize, Serialize};
	use sp_core::{H160, H256, U256};
	use sp_runtime::{
		traits::{AtLeast32BitUnsigned, Convert, MaybeSerializeDeserialize, Zero},
		DispatchError, RuntimeDebug, SaturatedConversion,
	};
	use sp_std::{convert::TryFrom, fmt::Debug, vec::Vec};

	/// ERC20PresetMinterBurner contract bytecode
	const CONTRACT_BYTECODE: &str = include_str!("../contract/bytecode.txt");

	#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord)]
	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	#[non_exhaustive]
	/// The name and unique ID for each token registered in `token-factory`
	pub enum Ticker {
		DOT = 0,
		KSM = 1,
		ACA = 2,
		AUSD = 3,
	}

	impl TryFrom<u8> for Ticker {
		type Error = ();

		fn try_from(v: u8) -> Result<Self, Self::Error> {
			match v {
				0 => Ok(Ticker::DOT),
				1 => Ok(Ticker::KSM),
				2 => Ok(Ticker::ACA),
				3 => Ok(Ticker::AUSD),
				_ => Err(()),
			}
		}
	}

	#[derive(sp_runtime::RuntimeDebug)]
	/// The supported currency types
	pub enum CurrencyId {
		/// The local instance of `balances` pallet, default GLMR
		Native,
		/// Token registered in `token-factory` pallet
		Token(Ticker),
	}

	impl TryFrom<Vec<u8>> for CurrencyId {
		type Error = ();
		fn try_from(v: Vec<u8>) -> Result<CurrencyId, ()> {
			match v.as_slice() {
				b"GLMR" => Ok(CurrencyId::Native),
				b"DOT" => Ok(CurrencyId::Token(Ticker::DOT)),
				b"KSM" => Ok(CurrencyId::Token(Ticker::KSM)),
				b"ACA" => Ok(CurrencyId::Token(Ticker::ACA)),
				b"AUSD" => Ok(CurrencyId::Token(Ticker::AUSD)),
				_ => Err(()),
			}
		}
	}

	/// The ERC token factory pallet
	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	/// Configuration trait of this pallet.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_evm::Config + pallet_sudo::Config {
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
		/// Token identifier
		type TokenId: Clone + Copy + FullCodec + Debug + PartialEq + Ord + MaybeSerializeDeserialize;
		/// Convert from AccountId to H160, is identity map for Moonbeam
		type AccountToH160: Convert<Self::AccountId, H160>;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// TokenID is already claimed
		IdClaimed,
		/// TokenID does not exist so cannot interact with it
		IdNotClaimed,
	}

	#[derive(PartialEq, Clone, Copy, Encode, Decode, sp_runtime::RuntimeDebug)]
	pub enum EvmCall {
		Register,
		Mint,
		Burn,
		TotalIssuance,
		BalanceOf,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Token register success [token_id, contract_address]
		Registered(T::TokenId, H160),
		/// Mint token success. [token_id, who, amount]
		Minted(T::TokenId, T::AccountId, T::Balance),
		/// Burn token success. [token_id, who, amount]
		Burned(T::TokenId, T::AccountId, T::Balance),
		/// Destroy all tokens success. [token_id, amount]
		DestroyedAll(T::TokenId, T::Balance),
		/// Call failed with exit reason [call, reason]
		EvmCallFailed(EvmCall, ExitReason),
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::storage]
	#[pallet::getter(fn nonce)]
	pub type Nonce<T: Config> = StorageValue<_, U256, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn tokens)]
	pub type Tokens<T: Config> = StorageValue<_, Vec<T::TokenId>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn contract_address)]
	pub type ContractAddress<T: Config> =
		StorageMap<_, Twox64Concat, T::TokenId, H160, OptionQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub nonce: U256,
		pub tokens: Vec<T::TokenId>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self {
				nonce: U256::zero(),
				tokens: vec![],
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			<Nonce<T>>::put(self.nonce);
			let mut tokens = self.tokens.clone();
			tokens.sort();
			tokens.dedup();
			for token in tokens {
				let _ = <Pallet<T>>::register_token(T::Origin::root(), token);
			}
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		pub fn register_token(origin: OriginFor<T>, id: T::TokenId) -> DispatchResultWithPostInfo {
			frame_system::ensure_root(origin)?;
			ensure!(!Self::exists(&id), Error::<T>::IdClaimed);
			let contract = FromHex::from_hex(CONTRACT_BYTECODE)
				.expect("Static smart contract is formatted incorrectly (should be hex)");
			let mut nonce = <Nonce<T>>::get();
			// deploy contract with sudo as MINTER_ROLE and BURNER_ROLE (without minting)
			match T::Runner::create(
				// from: H160
				Self::sudo_caller(),
				// input: Vec<u8>
				contract,
				// value: U256
				U256::zero(),
				// gas limit: U256
				U256::from(0x10000000).low_u64(),
				// gas price: U256
				Some(U256::from(1)),
				// nonce: Option<U256>
				Some(nonce),
				// config: EvmConfig
				T::config(),
			)? {
				ExecutionInfo {
					exit_reason: ExitReason::Succeed(_),
					value: address,
					..
				} => {
					// update runtime storage
					nonce += U256::from(1);
					<Nonce<T>>::put(nonce);
					<ContractAddress<T>>::insert(&id, address.clone());
					<Tokens<T>>::mutate(|list| {
						if let Err(loc) = list.binary_search(&id) {
							list.insert(loc, id.clone());
						}
					});
					Self::deposit_event(Event::Registered(id, address));
				}
				ExecutionInfo {
					exit_reason: reason,
					..
				} => {
					Self::deposit_event(Event::EvmCallFailed(EvmCall::Register, reason));
				}
			}
			Ok(().into())
		}

		#[pallet::weight(0)]
		pub fn destroy_all(origin: OriginFor<T>, id: T::TokenId) -> DispatchResultWithPostInfo {
			frame_system::ensure_root(origin)?;
			let _ = <ContractAddress<T>>::get(&id).ok_or(Error::<T>::IdNotClaimed)?;
			// TODO: ethereum transaction to remove/kill contract
			// TODO: get this via evm call, a balanceOf call before clearing the contract
			let amount_destroyed = T::Balance::zero();
			// clear storage and free id
			<ContractAddress<T>>::remove(&id);
			<Tokens<T>>::mutate(|list| {
				if let Ok(loc) = list.binary_search(&id) {
					list.remove(loc);
				}
			});
			Self::deposit_event(Event::DestroyedAll(id, amount_destroyed));
			Ok(().into())
		}
	}

	impl<T: Config> Pallet<T> {
		fn sudo_caller() -> H160 {
			T::AccountToH160::convert(<pallet_sudo::Module<T>>::key())
		}
	}

	/// Isolates behavior for minting/burning tokens from registration
	pub trait TokenMinter<Id, Account, Balance> {
		fn exists(id: &Id) -> bool;
		// setters
		fn mint(id: Id, who: Account, amount: Balance) -> DispatchResultWithPostInfo;
		fn burn(id: Id, who: Account, amount: Balance) -> DispatchResultWithPostInfo;
		// getters
		fn total_issuance(id: Id) -> Result<Balance, DispatchError>;
		fn balance_of(id: Id, who: Account) -> Result<Balance, DispatchError>;
	}

	impl<T: Config> TokenMinter<T::TokenId, H160, T::Balance> for Pallet<T> {
		fn exists(id: &T::TokenId) -> bool {
			<ContractAddress<T>>::get(id).is_some()
		}
		fn mint(id: T::TokenId, who: H160, amount: T::Balance) -> DispatchResultWithPostInfo {
			let address = <ContractAddress<T>>::get(&id).ok_or(Error::<T>::IdNotClaimed)?;
			let mut nonce = <Nonce<T>>::get();
			let mut input = hex_literal::hex!("40c10f19").to_vec();
			// append address
			input.extend_from_slice(H256::from(who.clone()).as_bytes());
			// append amount
			input.extend_from_slice(
				H256::from_uint(&U256::from(amount.saturated_into::<u128>())).as_bytes(),
			);
			// call evm
			match T::Runner::call(
				// source: H160
				Self::sudo_caller(),
				// target
				address,
				// input: Vec<u8>
				input,
				// value: U256
				U256::zero(),
				// gas limit: U256
				U256::from(0x10000000).low_u64(),
				// gas price: U256
				Some(U256::from(1)),
				// nonce: Option<U256>
				Some(nonce),
				// config: EvmConfig
				T::config(),
			)? {
				ExecutionInfo {
					exit_reason: ExitReason::Succeed(_),
					..
				} => {
					// increment nonce
					nonce += U256::from(1);
					<Nonce<T>>::put(nonce);
					Self::deposit_event(Event::Minted(
						id,
						T::AddressMapping::into_account_id(who),
						amount,
					));
				}
				ExecutionInfo {
					exit_reason: reason,
					..
				} => {
					// TODO: no need to increment nonce for this path right?
					Self::deposit_event(Event::EvmCallFailed(EvmCall::Mint, reason));
				}
			}
			Ok(().into())
		}
		fn burn(id: T::TokenId, who: H160, amount: T::Balance) -> DispatchResultWithPostInfo {
			let address = <ContractAddress<T>>::get(&id).ok_or(Error::<T>::IdNotClaimed)?;
			let mut nonce = <Nonce<T>>::get();
			let mut input = hex_literal::hex!("9dc29fac").to_vec();
			// append address
			input.extend_from_slice(H256::from(who.clone()).as_bytes());
			// append amount
			input.extend_from_slice(
				H256::from_uint(&U256::from(amount.saturated_into::<u128>())).as_bytes(),
			);
			match T::Runner::call(
				// source: H160
				Self::sudo_caller(),
				// target
				address,
				// input: Vec<u8>
				input,
				// value: U256
				U256::zero(),
				// gas limit: U256
				U256::from(0x10000000).low_u64(),
				// gas price: U256
				Some(U256::from(1)),
				// nonce: Option<H256>
				Some(nonce),
				// config: EvmConfig
				T::config(),
			)? {
				ExecutionInfo {
					exit_reason: ExitReason::Succeed(_),
					..
				} => {
					// increment nonce
					nonce += U256::from(1);
					<Nonce<T>>::put(nonce);
					Self::deposit_event(Event::Burned(
						id,
						T::AddressMapping::into_account_id(who),
						amount,
					));
				}
				ExecutionInfo {
					exit_reason: reason,
					..
				} => {
					Self::deposit_event(Event::EvmCallFailed(EvmCall::Burn, reason));
				}
			}
			Ok(().into())
		}
		/// Gets total issuance for the given token if it exists in local evm instance
		fn total_issuance(id: T::TokenId) -> Result<T::Balance, DispatchError> {
			let address = <ContractAddress<T>>::get(id).ok_or(Error::<T>::IdNotClaimed)?;
			let mut nonce = <Nonce<T>>::get();
			// first 4 bytes of hex output of Sha3("totalSupply()")
			let input = hex_literal::hex!("18160ddd").to_vec();
			match T::Runner::call(
				// source: H160
				Self::sudo_caller(),
				// target
				address,
				// input: Vec<u8>
				input,
				// value: U256
				U256::zero(),
				// gas limit: U256
				U256::from(0x10000000).low_u64(),
				// gas price: U256
				Some(U256::from(1)),
				// nonce: Option<H256>
				Some(nonce),
				// config: EvmConfig
				T::config(),
			) {
				Ok(ExecutionInfo {
					exit_reason: ExitReason::Succeed(_),
					value: result,
					..
				}) => {
					// increment nonce
					nonce += U256::from(1);
					<Nonce<T>>::put(nonce);
					let value = U256::from(result.as_slice()).saturated_into::<u128>();
					Ok(value.saturated_into::<T::Balance>())
				}
				Ok(ExecutionInfo {
					exit_reason: reason,
					..
				}) => {
					Self::deposit_event(Event::EvmCallFailed(EvmCall::TotalIssuance, reason));
					Ok(T::Balance::zero())
				}
				Err(e) => Err(e.into()),
			}
		}
		/// Gets token balance for the account
		fn balance_of(id: T::TokenId, who: H160) -> Result<T::Balance, DispatchError> {
			let address = <ContractAddress<T>>::get(id).ok_or(Error::<T>::IdNotClaimed)?;
			let mut nonce = <Nonce<T>>::get();
			// first 4 bytes of hex output of Sha3("balanceOf(address)")
			let mut input = hex_literal::hex!("70a08231").to_vec();
			// append address
			input.extend_from_slice(H256::from(who).as_bytes());
			match T::Runner::call(
				// source: H160
				Self::sudo_caller(),
				// target
				address,
				// input: Vec<u8>
				input,
				// value: U256
				U256::zero(),
				// gas limit: U256
				U256::from(0x10000000).low_u64(),
				// gas price: U256
				Some(U256::from(1)),
				// nonce: Option<H256>
				Some(nonce),
				// config: EvmConfig
				T::config(),
			) {
				Ok(ExecutionInfo {
					exit_reason: ExitReason::Succeed(_),
					value: result,
					..
				}) => {
					// increment nonce
					nonce += U256::from(1);
					<Nonce<T>>::put(nonce);
					let value = U256::from(result.as_slice()).saturated_into::<u128>();
					return Ok(value.saturated_into::<T::Balance>());
				}
				Ok(ExecutionInfo {
					exit_reason: reason,
					..
				}) => {
					Self::deposit_event(Event::EvmCallFailed(EvmCall::BalanceOf, reason));
					Ok(T::Balance::zero())
				}
				Err(e) => Err(e.into()),
			}
		}
	}
}
