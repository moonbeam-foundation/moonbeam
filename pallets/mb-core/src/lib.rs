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

#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;

use frame_support::dispatch::DispatchResult;
use frame_support::traits::{Currency, EnsureOrigin, Imbalance, LockableCurrency, OnUnbalanced};
use frame_support::{decl_event, decl_module, decl_storage};
use sp_runtime::traits::{CheckedAdd, CheckedSub};
use system::{ensure_root, RawOrigin};

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

type NegativeImbalanceOf<T> =
	<<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::NegativeImbalance;

type PositiveImbalanceOf<T> =
	<<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::PositiveImbalance;

pub trait Trait: system::Trait + pallet_balances::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
	type Currency: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;
}

decl_storage! {
	trait Store for Module<T: Trait> as MoonbeamModule {
		Treasury get(fn treasury): BalanceOf<T>;
		GenesisAccounts get(fn genesis_accounts): Vec<T::AccountId>;
	}
	add_extra_genesis {
		config(fntreasury): BalanceOf<T>;
		config(genesis_accounts): Vec<T::AccountId>;
		build(|config: &GenesisConfig<T>| {
			<Treasury<T>>::put(config.fntreasury);
			let _ = <GenesisAccounts<T>>::put(config.genesis_accounts.clone());
		});
	}
}

decl_event!(
	pub enum Event<T>
	where
		AccountId = <T as system::Trait>::AccountId,
		BalanceOf = BalanceOf<T>,
	{
		Absorbed(BalanceOf, BalanceOf),
		Rewarded(BalanceOf, BalanceOf),
		TreasuryTransferOk(AccountId, BalanceOf, BalanceOf),
	}
);

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {

		fn deposit_event() = default;

		#[weight = 0]
		fn mint(
			origin, _to: T::AccountId, _ammount: BalanceOf<T>
		) -> DispatchResult {
			let _caller = ensure_root(origin);
			Ok(())
		}

	}
}

pub struct Collective<AccountId>(AccountId);
impl<O: Into<Result<RawOrigin<AccountId>, O>> + From<RawOrigin<AccountId>>, AccountId>
	EnsureOrigin<O> for Collective<AccountId>
{
	type Success = ();
	fn try_origin(_o: O) -> Result<Self::Success, O> {
		Ok(())
	}
}

// https://substrate.dev/rustdocs/pre-v2.0-3e65111/pallet_staking/trait.Trait.html#associatedtype.RewardRemainder
pub struct RewardRemainder<T>(T);
impl<T: Trait> OnUnbalanced<NegativeImbalanceOf<T>> for RewardRemainder<T> {
	fn on_nonzero_unbalanced(_amount: NegativeImbalanceOf<T>) {
		// TODO Tokens have been minted and are unused for validator-reward.
		let _a = 1;
	}
}

// NegativeImbalance:
// Some balance has been subtracted somewhere, needs to be added somewhere else.
pub struct Absorb<T>(T);
impl<T: Trait> OnUnbalanced<NegativeImbalanceOf<T>> for Absorb<T> {
	fn on_nonzero_unbalanced(amount: NegativeImbalanceOf<T>) {
		let raw_amount = amount.peek();
		let treasury = <Treasury<T>>::get();
		if let Some(next_treasury) = treasury.checked_add(&raw_amount) {
			<Treasury<T>>::put(next_treasury);
		} else {
			// TODO
		}
		<Module<T>>::deposit_event(RawEvent::Absorbed(raw_amount, <Treasury<T>>::get()));
	}
}

// PositiveImbalance:
// Some balance has been added somewhere, needs to be subtracted somewhere else.
pub struct Reward<T>(T);
impl<T: Trait> OnUnbalanced<PositiveImbalanceOf<T>> for Reward<T> {
	fn on_nonzero_unbalanced(amount: PositiveImbalanceOf<T>) {
		let raw_amount = amount.peek();
		let treasury = <Treasury<T>>::get();
		if let Some(next_treasury) = treasury.checked_sub(&raw_amount) {
			<Treasury<T>>::put(next_treasury);
		} else {
			// TODO
		}
		<Module<T>>::deposit_event(RawEvent::Rewarded(raw_amount, <Treasury<T>>::get()));
	}
}
