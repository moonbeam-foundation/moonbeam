// Copyright 2019-2021 PureStake Inc.
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

//! Maps Author Ids as used in nimbus consensus layer to account ids as used i nthe runtime.
//! This should likely be moved to nimbus eventually.
//!
//! This pallet maps AuthorId => AccountId which is most useful when using propositional style
//! queries. This mapping will likely need to go the other way if using exhaustive authority sets.
//! That could either be a seperate pallet, or this pallet could implement a two-way mapping. But
//! for now it it one-way

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet;

pub use pallet::*;

#[pallet]
pub mod pallet {

	use frame_support::pallet_prelude::*;
	use frame_support::traits::FindAuthor;
	use frame_system::pallet_prelude::*;
	use nimbus_primitives::{CanAuthor, EventHandler};
	use sp_runtime::ConsensusEngineId;

	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	/// Configuration trait of this pallet.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The type of authority id that will be used at the conensus layer.
		type AuthorId: Member + Parameter + MaybeSerializeDeserialize;
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	/// An error that can occur while executing the mapping pallet's logic.
	#[pallet::error]
	pub enum Error<T> {
		/// The association can't be cleared because it is not found.
		AssociationNotFound,
		/// The association can't be cleared because it belongs to another account.
		NotYourAssociation,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Register your AuthorId onchain so blocks you author are associated with your account.
		///
		/// Users who have been (or will soon be) elected active collators in staking,
		/// should submit this extrinsic to earn rewards.
		#[pallet::weight(0)]
		pub fn add_association(origin: OriginFor<T>, author_id: T::AuthorId) -> DispatchResult {
			let account_id = ensure_signed(origin)?;

			//TODO make sure they're a candidate

			//TODO take the security deposit

			AuthorIds::<T>::insert(author_id, account_id);

			//TODO Emit an event

			Ok(())
		}

		/// Change your AuthorId.
		///
		/// This is useful for normal key rotation or for when switching from one pysical collator
		/// machine to another. No new security deposit is required.
		#[pallet::weight(0)]
		pub fn update_association(
			origin: OriginFor<T>,
			old_author_id: T::AuthorId,
			new_author_id: T::AuthorId,
		) -> DispatchResult {
			todo!()
		}

		/// Clear your AuthorId.
		///
		/// This is useful when you are no longer an author and would like to re-claim your security
		/// deposit.
		#[pallet::weight(0)]
		pub fn clear_association(
			origin: OriginFor<T>,
			author_id: T::AuthorId,
		) -> DispatchResultWithPostInfo {
			let calling_accout = ensure_signed(origin)?;

			let stored_account =
				AuthorIds::<T>::try_get(&author_id).map_err(|_| Error::<T>::AssociationNotFound)?;

			ensure!(
				calling_accout == stored_account,
				Error::<T>::NotYourAssociation
			);

			AuthorIds::<T>::remove(&author_id);

			//TODO return the security deposit

			//TODO Emit event

			Ok(().into())
		}

		/// Narc on another account for having a useless association and colelct a bounty.
		///
		/// This incentivizes good citizenship in the form of cleaning up others' defunct associations.
		/// When you clean up another account's association, you will receive X percent of their security deposit.
		/// TODO there probably needs to be some kind of grace period. Like you can't clean someone else's up
		/// within the first Y blocks it has been registered. Actually this is a great idea. That _forces_
		/// collators to clean up their associations or else risk having them cleaned out from under them.
		#[pallet::weight(0)]
		pub fn narc_defunct_association(
			origin: OriginFor<T>,
			author_id: T::AuthorId,
		) -> DispatchResult {
			todo!()
		}
	}

	#[pallet::storage]
	#[pallet::getter(fn account_id_of)]
	/// We maintain a mapping from the AuthorIds used in the consensus layer
	/// to the AccountIds runtime (including this staking pallet).
	type AuthorIds<T: Config> = StorageMap<_, Twox64Concat, T::AuthorId, T::AccountId, OptionQuery>;

	#[pallet::genesis_config]
	/// Genesis config for author mapping pallet
	pub struct GenesisConfig<T: Config> {
		/// The associations that should exist at chain genesis
		pub author_ids: Vec<(T::AuthorId, T::AccountId)>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self { author_ids: vec![] }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			for (author_id, account_id) in &self.author_ids {
				AuthorIds::<T>::insert(author_id, account_id);
			}
		}
	}

	pub struct MappedEventHandler<T, Inner>(PhantomData<(T, Inner)>);

	impl<T, Inner> EventHandler<T::AuthorId> for MappedEventHandler<T, Inner>
	where
		T: Config,
		Inner: EventHandler<T::AccountId>,
	{
		fn note_author(author_id: T::AuthorId) {
			AuthorIds::<T>::get(&author_id).map(|account_id| Inner::note_author(account_id));
		}
	}

	pub struct MappedCanAuthor<T, Inner>(PhantomData<(T, Inner)>);

	impl<T, Inner> CanAuthor<T::AuthorId> for MappedCanAuthor<T, Inner>
	where
		T: Config,
		Inner: CanAuthor<T::AccountId>,
	{
		fn can_author(author_id: &T::AuthorId, slot: &u32) -> bool {
			AuthorIds::<T>::get(author_id)
				.map(|account_id| Inner::can_author(&account_id, slot))
				.unwrap_or(false)
		}
	}

	pub struct MappedFindAuthor<T, Inner>(PhantomData<(T, Inner)>);

	impl<T, Inner> FindAuthor<T::AccountId> for MappedFindAuthor<T, Inner>
	where
		T: Config,
		Inner: FindAuthor<T::AuthorId>,
	{
		fn find_author<'a, I>(digests: I) -> Option<T::AccountId>
		where
			I: 'a + IntoIterator<Item = (ConsensusEngineId, &'a [u8])>,
		{
			Inner::find_author(digests)
				.map(|a| AuthorIds::<T>::get(a))
				.flatten()
		}
	}
}
