// Copyright 2025 Moonbeam foundation
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

#![cfg(feature = "runtime-benchmarks")]

use crate::Config;
use crate::{BalanceOf, Call, Pallet, WRAPPED_BYTES_POSTFIX, WRAPPED_BYTES_PREFIX};
use ed25519_dalek::Signer;
use frame_benchmarking::{account, benchmarks};
use frame_support::traits::{Currency, Get, OnFinalize};
use frame_system::pallet_prelude::*;
use frame_system::RawOrigin;
use parity_scale_codec::Encode;
use sp_core::{
	crypto::{AccountId32, UncheckedFrom},
	ed25519,
};
use sp_runtime::{
	traits::{BlockNumberProvider, One},
	MultiSignature,
};
use sp_std::vec;
use sp_std::vec::Vec;

/// Default balance amount is minimum contribution
fn default_balance<T: Config>() -> BalanceOf<T> {
	T::MinimumReward::get()
}

/// Create a funded user.
fn fund_specific_account<T: Config>(pallet_account: T::AccountId, extra: BalanceOf<T>) {
	let default_balance = default_balance::<T>();
	let total = default_balance + extra;
	T::RewardCurrency::make_free_balance_be(&pallet_account, total);
	let _ = T::RewardCurrency::issue(total);
}

/// Create a funded user.
fn create_funded_user<T: Config>(
	string: &'static str,
	n: u32,
	extra: BalanceOf<T>,
) -> T::AccountId {
	const SEED: u32 = 0;
	let user = account(string, n, SEED);
	let default_balance = default_balance::<T>();
	let total = default_balance + extra;
	T::RewardCurrency::make_free_balance_be(&user, total);
	let _ = T::RewardCurrency::issue(total);
	user
}

/// Insert contributors.
fn insert_contributors<T: Config>(
	contributors: Vec<(T::RelayChainAccountId, Option<T::AccountId>, BalanceOf<T>)>,
) -> Result<(), &'static str> {
	let mut sub_vec = Vec::new();
	let batch = max_batch_contributors::<T>();
	// Due to the MaxInitContributors associated type, we need ton insert them in batches
	// When we reach the batch size, we insert them
	for i in 0..contributors.len() {
		sub_vec.push(contributors[i].clone());
		// If we reached the batch size, we should insert them
		if i as u32 % batch == batch - 1 || i == contributors.len() - 1 {
			Pallet::<T>::initialize_reward_vec(RawOrigin::Root.into(), sub_vec.clone())?;
			sub_vec.clear()
		}
	}
	Ok(())
}

/// Create a Contributor.
fn close_initialization<T: Config>(
	end_vesting_block: T::VestingBlockNumber,
) -> Result<(), &'static str> {
	Pallet::<T>::complete_initialization(RawOrigin::Root.into(), end_vesting_block)?;
	Ok(())
}

fn create_sig<T: Config>(seed: u32, payload: Vec<u8>) -> (AccountId32, MultiSignature) {
	// Crate seed
	let mut seed_32: [u8; 32] = [0u8; 32];
	let seed_as_slice = seed.to_be_bytes();
	for j in 0..seed_as_slice.len() {
		seed_32[j] = seed_as_slice[j]
	}

	let signing_key = ed25519_dalek::SigningKey::from_bytes(&seed_32);
	let public = signing_key.verifying_key();
	let sig = signing_key.sign(&payload).to_bytes();
	let signature: MultiSignature = ed25519::Signature::from_raw(sig).into();

	let ed_public: ed25519::Public = ed25519::Public::unchecked_from(public.to_bytes());
	let account: AccountId32 = ed_public.into();
	(account, signature.into())
}

fn max_batch_contributors<T: Config>() -> u32 {
	T::MaxInitContributors::get()
}

const SEED: u32 = 999999999;

benchmarks! {
	claim {
		// Fund pallet account
		let total_pot = 100u32;
		fund_specific_account::<T>(Pallet::<T>::account_id(), total_pot.into());

		// The user that will make the call
		let caller: T::AccountId = create_funded_user::<T>("user", SEED, 100u32.into());

		// We verified there is no dependency of the number of contributors already inserted in claim
		// Create 1 contributor
		let contributors: Vec<(T::RelayChainAccountId, Option<T::AccountId>, BalanceOf<T>)> =
			vec![(AccountId32::from([1u8;32]).into(), Some(caller.clone()), total_pot.into())];

		// Insert them
		insert_contributors::<T>(contributors)?;

		// Close initialization
		close_initialization::<T>(10u32.into())?;

		// First inherent
		T::VestingBlockProvider::set_block_number(1u32.into());
		Pallet::<T>::on_finalize(BlockNumberFor::<T>::one());

		// Create 4th relay block, by now the user should have vested some amount
		T::VestingBlockProvider::set_block_number(4u32.into());
	}:  _(RawOrigin::Signed(caller.clone()))
	verify {
	  assert_eq!(Pallet::<T>::accounts_payable(&caller).unwrap().total_reward, (100u32.into()));
	}

	update_reward_address {
		// Fund pallet account
		let total_pot = 100u32;
		fund_specific_account::<T>(Pallet::<T>::account_id(), total_pot.into());

		// The user that will make the call
		let caller: T::AccountId = create_funded_user::<T>("user", SEED, 100u32.into());

		let relay_account: T::RelayChainAccountId = AccountId32::from([1u8;32]).into();
		// We verified there is no dependency of the number of contributors already inserted in update_reward_address
		// Create 1 contributor
		let contributors: Vec<(T::RelayChainAccountId, Option<T::AccountId>, BalanceOf<T>)> =
			vec![(relay_account.clone(), Some(caller.clone()), total_pot.into())];

		// Insert them
		insert_contributors::<T>(contributors)?;

		// Close initialization
		close_initialization::<T>(10u32.into())?;

		// First inherent
		T::VestingBlockProvider::set_block_number(1u32.into());
		Pallet::<T>::on_finalize(BlockNumberFor::<T>::one());


		// Let's advance the relay so that the vested  amount get transferred
		T::VestingBlockProvider::set_block_number(4u32.into());

		// The new user
		let new_user = create_funded_user::<T>("user", SEED+1, 0u32.into());

	}:  _(RawOrigin::Signed(caller.clone()), new_user.clone())
	verify {
		assert_eq!(Pallet::<T>::accounts_payable(&new_user).unwrap().total_reward, (100u32.into()));
		assert!(Pallet::<T>::claimed_relay_chain_ids(&relay_account).is_some());
	}

	associate_native_identity {
		// Fund pallet account
		let total_pot = 100u32;
		fund_specific_account::<T>(Pallet::<T>::account_id(), total_pot.into());

		// The caller that will associate the account
		let caller: T::AccountId = create_funded_user::<T>("user", SEED, 100u32.into());

		// Construct payload
		let mut payload = WRAPPED_BYTES_PREFIX.to_vec();
		payload.append(&mut T::SignatureNetworkIdentifier::get().to_vec());
		payload.append(&mut caller.clone().encode());
		payload.append(&mut WRAPPED_BYTES_POSTFIX.to_vec());

		// Create a fake sig for such an account
		let (relay_account, signature) = create_sig::<T>(SEED, payload);

		// We verified there is no dependency of the number of contributors already inserted in associate_native_identity
		// Create 1 contributor
		let contributors: Vec<(T::RelayChainAccountId, Option<T::AccountId>, BalanceOf<T>)> =
		vec![(relay_account.clone().into(), None, total_pot.into())];

		// Insert them
		insert_contributors::<T>(contributors)?;

		// Clonse initialization
		close_initialization::<T>(10u32.into())?;

		// First inherent
		T::VestingBlockProvider::set_block_number(1u32.into());
		Pallet::<T>::on_finalize(BlockNumberFor::<T>::one());

	}:  _(RawOrigin::Signed(caller.clone()), caller.clone(), relay_account.into(), signature)
	verify {
		assert_eq!(Pallet::<T>::accounts_payable(&caller).unwrap().total_reward, (100u32.into()));
	}

	change_association_with_relay_keys {

		// The weight will depend on the number of proofs provided
		// We need to parameterize this value
		// We leave this as the max batch length
		let x in 1..max_batch_contributors::<T>();

		// Fund pallet account
		let total_pot = 100u32*x;
		fund_specific_account::<T>(Pallet::<T>::account_id(), total_pot.into());

		// The first reward account that will associate the account
		let first_reward_account: T::AccountId = create_funded_user::<T>("user", SEED, 100u32.into());

		// The account to which we will update our reward account
		let second_reward_account: T::AccountId = create_funded_user::<T>("user", SEED-1, 100u32.into());

		let mut proofs: Vec<(T::RelayChainAccountId, MultiSignature)> = Vec::new();

		// Construct payload
		let mut payload = WRAPPED_BYTES_PREFIX.to_vec();
		payload.append(&mut T::SignatureNetworkIdentifier::get().to_vec());
		payload.append(&mut second_reward_account.clone().encode());
		payload.append(&mut first_reward_account.clone().encode());
		payload.append(&mut WRAPPED_BYTES_POSTFIX.to_vec());

		// Create N sigs for N accounts
		for i in 0..x {
			let (relay_account, signature) = create_sig::<T>(SEED-i, payload.clone());
			proofs.push((relay_account.into(), signature));
		}

		// Create x contributors
		// All of them map to the same account
		let mut contributors: Vec<(T::RelayChainAccountId, Option<T::AccountId>, BalanceOf<T>)> = Vec::new();
		for (relay_account, _) in proofs.clone() {
			contributors.push((relay_account, Some(first_reward_account.clone()), 100u32.into()));
		}

		// Insert them
		insert_contributors::<T>(contributors.clone())?;

		// Clonse initialization
		close_initialization::<T>(10u32.into())?;

		// First inherent
		T::VestingBlockProvider::set_block_number(1u32.into());
		Pallet::<T>::on_finalize(BlockNumberFor::<T>::one());

	}:  _(RawOrigin::Signed(first_reward_account.clone()), second_reward_account.clone(), first_reward_account.clone(), proofs)
	verify {
		assert!(Pallet::<T>::accounts_payable(&second_reward_account).is_some());
		assert_eq!(Pallet::<T>::accounts_payable(&second_reward_account).unwrap().total_reward, (100u32*x).into());
		assert!(Pallet::<T>::accounts_payable(&first_reward_account).is_none());

	}

}
