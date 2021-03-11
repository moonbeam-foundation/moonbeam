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

//! # Crowdloan Rewards Pallet
//!
//! This pallet issues rewards to citizens who participated in a crowdloan on the backing relay
//! chain (eg Kusama) in order to help this parrachain acquire a parachain slot.
//!
//! ## Monetary Policy
//!
//! This is simple and mock for now. We can do whatever we want.
//! This pallet stores a constant  "reward ratio" which is the number of reward tokens to pay per
//! contributed token. In simple cases this can be 1, but needs to be customizeable to allow for
//! vastly differing absolute token supplies between relay and para.
//! Vesting is also linear. No tokens are vested at genesis and they unlock linearly until a
//! predecided block number. Vesting computations happen on demand when payouts are requested. So
//! no block weight is ever wasted on this, and there is no "base-line" cost of updating vestings.
//! Like I said, we can anything we want there. Even a non-linear reward curve to disincentivize
//! whales.
//!
//! ## Payout Mechanism
//!
//! The current payout mechanism requires contributors to claim their payouts. Because they are
//! paying the transaction fees for this themselves, they can do it as often as every block, or
//! wait and claim the entire thing once it is fully vested. We could consider auto payouts if we
//! want.
//!
//! ## Sourcing Contribution Information
//!
//! The pallet can learn about the crowdloan contributions in several ways.
//!
//! * **Assocaited at Genesis**
//!
//! The simplest way is that the native identity and contribution amount are configured at genesis.
//! This makes sense in a scenario where the crowdloan took place entirely offchain.
//!
//! * **Unassociated at Genesis**
//!
//! When the crowdloan takes place on-relay-chain, contributors will not have a way to specify a native account
//! into which they will receive rewards on the parachain. TODO that would be easy to add to the
//! relay chain actually. In this case the genesis config contains information about the
//! relay chain style contributor address, and the contribution amount. In this case the
//! contributor is responsible for making a transaction that associates a native ID. The tx
//! includes a signature by the relay chain idetity over the native identity.
//!
//! * **ReadingRelayState**
//!
//! The most elegant, but most complex solution would be for the para to read the contributions
//! directly from the relay state. Blocked by https://github.com/paritytech/cumulus/issues/320 so
//! I won't persue it further right now. I can't decide whether that would really add security /
//! trustlessness, or is just a sexy blockchain thing to do. Contributors can always audit the
//! genesis block and make sure their contribution is in it, so in that sense reading relay state
//! isn't necessary. But if a single contribution is left out, the rest of the contributors might
//! not care enough to delay network launch. The little guy might get sensored.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet;

pub use pallet::*;

#[pallet]
pub mod pallet {

	use frame_support::debug;
	use frame_support::pallet_prelude::*;
	use frame_support::traits::Randomness;
	use frame_support::traits::Vec;
	use frame_system::pallet_prelude::*;
	use frame_support::traits::Currency;
	use parity_scale_codec::{Encode, Decode};

	/// The Author Filter pallet
	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	/// Configuration trait of this pallet.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// The currency in which the rewards will be paid (probably the parachain native currency)
		type RewardCurrency: Currency<Self::AccountId>;

		// TODO What trait bounds do I need here? I think concretely we would
		// be using MultiSigner? Or maybe MultiAccount?
		/// The AccountId type contributors used on the relay chain.
		type RelayChainAccountId: Encode + Decode;
	}

	type BalanceOf<T> = <<T as Config>::RewardCurrency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	// No hooks
	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Associate a native identity with a relay chain identity that made a contribution to the
		/// Crowdloan. Rewards will be paid the the native identity.
		#[pallet::weight(0)]
		pub fn associate_native_identity(origin: OriginFor<T>, proof: Vec<u8>) -> DispatchResultWithPostInfo {
			ensure_signed(origin)?;

			//TODO check the proof:
			// 1. Is signed by an actual contributor
			// 2. Signes a valid native identity
			// 3. The signed identity is the caller
			//TODO I don't think #3 is actually necessary, and it might complicate things.
			// What if the recipient doesn't have any native funds to pay the tx fee yet.
			// Maybe we should let anyone submit, or have them be unsigned like the dot allocations
			// https://github.com/paritytech/polkadot/blob/master/runtime/common/src/claims.rs

			//TODO update storage

			//TODO Emit event

			Ok(Default::default())
		}

		/// Collect whatever portion of your reward are currently vested.
		#[pallet::weight(0)]
		pub fn show_me_the_money(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let payee = ensure_signed(origin)?;

			//TODO check that the payee has rewards coming to them

			//TODO calculate the newly-vested amount

			//TODO update this pallets storage

			//TODO make the payment

			//TODO Emit event

			Ok(Default::default())
		}
	}

	/// The ratio of (reward tokens to be paid) / (relay chain funds contributed)
	/// This is dead stupid simple using a u32. So the reward amount has to be an integer
	/// multiple of the contribution amount. A better fixed-ration solution would be
	/// https://crates.parity.io/sp_arithmetic/fixed_point/struct.FixedU128.html
	/// We could also do something fancy and non-linear if the need arises.
	#[pallet::storage]
	pub type EligibleRatio<T> = StorageValue<_, u32, ValueQuery, OneToOneRewards<T>>;

	#[pallet::type_value]
	pub fn OneToOneRewards<T: Config>() -> u32 {
		1
	}

	// Design decision:
	// Genesis config contributions are specified relay-chain currency
	// Conversion to reward currency happens onchain:
	// 1. When building genesis block for pre-associated contributors
	// 2. When associating an account for others
	// This pallets storages are all in thems of rewards
	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		/// Contributions that have a native account id associated already.
		pub associated: Vec<(T::AccountId, u32)>,
		/// Contributions that will need a native account id to be associated through an extrinsic.
		pub unassociated: Vec<(/*T::RelayChainAccountId,*/ u32)>,
	}

	//TODO can I derive default?
	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self {
				associated: Vec::new(),
				unassociated: Vec::new(),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {

			//TODO warn if no contributions (associated or not) are specified
			//TODO calculate reward amounts and copy into storage
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(fn deposit_event)]
	pub enum Event<T: Config> {
		/// Someone has proven they made a contribution and associated a native identity with it.
		/// Data is the native account and the total amount of _rewards_ that will be paid
		NativeIdentityAssociated(T::AccountId, BalanceOf<T>),
		/// A contributor has claimed some rewards.
		/// Data is the account getting paid and the amount of rewards paid.
		RewardsPaid(T::AccountId, BalanceOf<T>),
	}
}
