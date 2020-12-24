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

//! pallet-staking common objects and conversion impls

use crate::*;
use parity_scale_codec::HasCompact;
use sp_runtime::traits::Convert;

/// The amount of exposure (to slashing) than an individual nominator has.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, RuntimeDebug)]
pub struct IndividualExposure<AccountId, Balance: HasCompact> {
	/// The stash account of the nominator in question.
	pub who: AccountId,
	/// Amount of funds exposed.
	#[codec(compact)]
	pub value: Balance,
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

impl<A, B: HasCompact> From<Nomination<A, B>> for IndividualExposure<A, B> {
	fn from(nom: Nomination<A, B>) -> IndividualExposure<A, B> {
		IndividualExposure {
			who: nom.owner,
			value: nom.amount,
		}
	}
}

impl<A: PartialEq, B: HasCompact + Zero> From<ValState<A, B>> for Exposure<A, B> {
	fn from(state: ValState<A, B>) -> Exposure<A, B> {
		let mut others = Vec::<IndividualExposure<A, B>>::new();
		let mut own = Zero::zero();
		for Nomination { owner, amount } in state.nominations {
			if owner == state.validator {
				own = amount;
			} else {
				others.push(Nomination { owner, amount }.into());
			}
		}
		Exposure {
			total: state.total,
			own,
			others,
		}
	}
}

/// Means for interacting with a specialized version of the `session` trait.
///
/// This is needed because `Staking` sets the `ValidatorIdOf` of the `pallet_session::Config`
pub trait SessionInterface<AccountId>: frame_system::Config {
	/// Disable a given validator by stash ID.
	///
	/// Returns `true` if new era should be forced at the end of this session.
	/// This allows preventing a situation where there is too many validators
	/// disabled and block production stalls.
	fn disable_validator(validator: &AccountId) -> Result<bool, ()>;
	/// Get the validators from session.
	fn validators() -> Vec<AccountId>;
	/// Prune historical session tries up to but not including the given index.
	fn prune_historical_up_to(up_to: RoundIndex);
}

impl<T: Config> SessionInterface<<T as frame_system::Config>::AccountId> for T
where
	T: pallet_session::Config<ValidatorId = <T as frame_system::Config>::AccountId>,
	T: pallet_session::historical::Config<
		FullIdentification = Exposure<<T as frame_system::Config>::AccountId, BalanceOf<T>>,
		FullIdentificationOf = ExposureOf<T>,
	>,
	T::SessionHandler: pallet_session::SessionHandler<<T as frame_system::Config>::AccountId>,
	T::SessionManager: pallet_session::SessionManager<<T as frame_system::Config>::AccountId>,
	T::ValidatorIdOf: Convert<
		<T as frame_system::Config>::AccountId,
		Option<<T as frame_system::Config>::AccountId>,
	>,
{
	fn disable_validator(validator: &<T as frame_system::Config>::AccountId) -> Result<bool, ()> {
		<pallet_session::Module<T>>::disable(validator)
	}

	fn validators() -> Vec<<T as frame_system::Config>::AccountId> {
		<pallet_session::Module<T>>::validators()
	}

	fn prune_historical_up_to(up_to: RoundIndex) {
		<pallet_session::historical::Module<T>>::prune_up_to(up_to);
	}
}
