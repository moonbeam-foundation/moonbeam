// Copyright 2019-2020 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

//! Some configurable implementations as associated type for the substrate runtime.
use sp_std::prelude::*;

use codec::{Decode, Encode, HasCompact};
use frame_support::traits::{Currency, OnUnbalanced};
use sp_runtime::traits::Convert;
use sp_runtime::{Perbill, RuntimeDebug};
use sp_staking::offence::OffenceDetails;

use crate::{Authorship, BalanceOf, Balances, NegativeImbalance};
use node_primitives::Balance;

pub struct Author;
impl OnUnbalanced<NegativeImbalance> for Author {
	fn on_nonzero_unbalanced(amount: NegativeImbalance) {
		Balances::resolve_creating(&Authorship::author(), amount);
	}
}

/// Struct that handles the conversion of Balance -> `u64`. This is used for staking's election
/// calculation.
pub struct CurrencyToVoteHandler;

impl CurrencyToVoteHandler {
	fn factor() -> Balance {
		(Balances::total_issuance() / u64::max_value() as Balance).max(1)
	}
}

impl Convert<Balance, u64> for CurrencyToVoteHandler {
	fn convert(x: Balance) -> u64 {
		(x / Self::factor()) as u64
	}
}

impl Convert<u128, Balance> for CurrencyToVoteHandler {
	fn convert(x: u128) -> Balance {
		x * Self::factor()
	}
}

// All below are trait implemenations that we need to satisfy for the historical feature of the pallet-session
// and by offences.

/// The amount of exposure (to slashing) than an individual nominator has.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, RuntimeDebug)]
pub struct IndividualExposure<AccountId, Balance: HasCompact> {
	/// The stash account of the nominator in question.
	who: AccountId,
	/// Amount of funds exposed.
	#[codec(compact)]
	value: Balance,
}

/// A snapshot of the stake backing a single validator in the system.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, RuntimeDebug)]
pub struct Exposure<AccountId, Balance: HasCompact> {
	/// The total balance backing this validator.
	#[codec(compact)]
	pub total: Balance,
	/// The validator's own stash that is exposed.
	#[codec(compact)]
	pub own: Balance,
	/// The portions of nominators stashes that are exposed.
	pub others: Vec<IndividualExposure<AccountId, Balance>>,
}

/// A typed conversion from stash account ID to the current exposure of nominators
/// on that account.
pub struct ExposureOf<T>(sp_std::marker::PhantomData<T>);
impl<T: mb_session::Trait> Convert<T::AccountId, Option<Exposure<T::AccountId, BalanceOf<T>>>>
	for ExposureOf<T>
{
	fn convert(_validator: T::AccountId) -> Option<Exposure<T::AccountId, BalanceOf<T>>> {
		None
	}
}

pub struct StakingOffences<T>(sp_std::marker::PhantomData<T>);
impl<T: mb_session::Trait>
	sp_staking::offence::OnOffenceHandler<
		T::AccountId,
		pallet_session::historical::IdentificationTuple<T>,
	> for StakingOffences<T>
where
	T: pallet_session::Trait<ValidatorId = <T as system::Trait>::AccountId>,
	T: pallet_session::historical::Trait<
		FullIdentification = Exposure<<T as system::Trait>::AccountId, BalanceOf<T>>,
		FullIdentificationOf = ExposureOf<T>,
	>,
	T::SessionHandler: pallet_session::SessionHandler<<T as system::Trait>::AccountId>,
	T::SessionManager: pallet_session::SessionManager<<T as system::Trait>::AccountId>,
	T::ValidatorIdOf:
		Convert<<T as system::Trait>::AccountId, Option<<T as system::Trait>::AccountId>>,
{
	fn on_offence(
		_offenders: &[OffenceDetails<
			T::AccountId,
			pallet_session::historical::IdentificationTuple<T>,
		>],
		_slash_fraction: &[Perbill],
		_slash_session: u32,
	) -> Result<(), ()> {
		Ok(())
	}

	fn can_report() -> bool {
		true
	}
}
