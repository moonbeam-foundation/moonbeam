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

//! Pallet that allows block authors to include their identity in a block via an inherent.
//! Currently the author does not _prove_ their identity, just states it. So it should not be used,
//! for things like equivocation slashing that require authenticated authorship information.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::debug;
use frame_support::{
	decl_error, decl_module, decl_storage, ensure,
	traits::FindAuthor,
	weights::{DispatchClass, Weight},
};
use frame_system::{ensure_none, Config as System};
use parity_scale_codec::{Decode, Encode};
#[cfg(feature = "std")]
use sp_inherents::ProvideInherentData;
use sp_inherents::{InherentData, InherentIdentifier, IsFatalError, ProvideInherent};
use sp_runtime::{ConsensusEngineId, DigestItem, RuntimeString};
use sp_std::vec::Vec;

/// The given account ID is the author of the current block.
pub trait EventHandler<Author> {
	fn note_author(author: Author);
}
/// Permissions for what block author can be set in this pallet
pub trait CanAuthor<AccountId> {
	fn can_author(account: &AccountId) -> bool;
}
/// Default implementation where anyone can author, see `stake` and `author-filter` pallets for
/// additional implementations.
impl<T> CanAuthor<T> for () {
	fn can_author(_: &T) -> bool {
		true
	}
}

pub trait Config: System {
	/// Other pallets that want to be informed about block authorship
	type EventHandler: EventHandler<Self::AccountId>;

	/// Checks if account can be set as block author.
	/// If the pallet that implements this trait depends on an inherent, that inherent **must**
	/// be included before this one.
	type CanAuthor: CanAuthor<Self::AccountId>;
}

decl_error! {
	pub enum Error for Module<T: Config> {
		/// Author already set in block.
		AuthorAlreadySet,
		/// The author in the inherent is not an eligible author.
		CannotBeAuthor,
	}
}

decl_storage! {
	trait Store for Module<T: Config> as Author {
		/// Author of current block.
		Author: Option<T::AccountId>;
	}
}

decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn on_initialize() -> Weight {
			<Author<T>>::kill();
			0
		}

		/// Inherent to set the author of a block
		#[weight = (
			0,
			DispatchClass::Mandatory
		)]
		fn set_author(origin, author: T::AccountId) {
			debug::RuntimeLogger::init();
			debug::info!("In the author inherent dispatchable");

			ensure_none(origin)?;
			ensure!(<Author<T>>::get().is_none(), Error::<T>::AuthorAlreadySet);
			ensure!(T::CanAuthor::can_author(&author), Error::<T>::CannotBeAuthor);

			// Update storage
			Author::<T>::put(&author);

			// Add a digest item so Apps can detect the block author
			// For now we use the Consensus digest item.
			// Maybe this will change later.
			frame_system::Module::<T>::deposit_log(DigestItem::<T::Hash>::Consensus(
				ENGINE_ID,
				author.encode(),
			));

			// Notify any other pallets that are listening (eg rewards) about the author
			T::EventHandler::note_author(author);
		}
	}
}

impl<T: Config> FindAuthor<T::AccountId> for Module<T> {
	fn find_author<'a, I>(_digests: I) -> Option<T::AccountId>
	where
		I: 'a + IntoIterator<Item = (ConsensusEngineId, &'a [u8])>,
	{
		// We don't use the digests at all.
		// This will only return the correct author _after_ the authorship inherent is processed.
		<Author<T>>::get()
	}
}

// Can I express this as `*b"auth"` like we do for the inherent id?
pub const ENGINE_ID: ConsensusEngineId = [b'a', b'u', b't', b'h'];

pub const INHERENT_IDENTIFIER: InherentIdentifier = *b"author__";

#[derive(Encode)]
#[cfg_attr(feature = "std", derive(Debug, Decode))]
pub enum InherentError {
	Other(RuntimeString),
}

impl IsFatalError for InherentError {
	fn is_fatal_error(&self) -> bool {
		match *self {
			InherentError::Other(_) => true,
		}
	}
}

impl InherentError {
	/// Try to create an instance ouf of the given identifier and data.
	#[cfg(feature = "std")]
	pub fn try_from(id: &InherentIdentifier, data: &[u8]) -> Option<Self> {
		if id == &INHERENT_IDENTIFIER {
			<InherentError as parity_scale_codec::Decode>::decode(&mut &data[..]).ok()
		} else {
			None
		}
	}
}

/// The type of data that the inherent will contain.
/// Just a byte array. It will be decoded to an actual account id later.
pub type InherentType = Vec<u8>;

/// The thing that the outer node will use to actually inject the inherent data
#[cfg(feature = "std")]
pub struct InherentDataProvider(pub InherentType);

#[cfg(feature = "std")]
impl ProvideInherentData for InherentDataProvider {
	fn inherent_identifier(&self) -> &'static InherentIdentifier {
		&INHERENT_IDENTIFIER
	}

	fn provide_inherent_data(
		&self,
		inherent_data: &mut InherentData,
	) -> Result<(), sp_inherents::Error> {
		inherent_data.put_data(INHERENT_IDENTIFIER, &self.0)
	}

	fn error_to_string(&self, error: &[u8]) -> Option<String> {
		InherentError::try_from(&INHERENT_IDENTIFIER, error).map(|e| format!("{:?}", e))
	}
}

impl<T: Config> ProvideInherent for Module<T> {
	type Call = Call<T>;
	type Error = InherentError;
	const INHERENT_IDENTIFIER: InherentIdentifier = INHERENT_IDENTIFIER;

	fn is_inherent_required(_: &InherentData) -> Result<Option<Self::Error>, Self::Error> {
		// Return Ok(Some(_)) unconditionally because this inherent is required in every block
		// If it is not found, throw an AuthorInherentRequired error.
		Ok(Some(InherentError::Other(
			sp_runtime::RuntimeString::Borrowed("AuthorInherentRequired"),
		)))
	}

	fn create_inherent(data: &InherentData) -> Option<Self::Call> {
		// Grab the Vec<u8> labelled with "author__" from the map of all inherent data
		let author_raw = data
			.get_data::<InherentType>(&INHERENT_IDENTIFIER)
			.expect("Gets and decodes authorship inherent data")?;

		//TODO we need to make the author _prove_ their identity, not just claim it.
		// we should have them sign something here. Best idea so far: parent block hash.

		// Decode the Vec<u8> into an account Id
		let author =
			T::AccountId::decode(&mut &author_raw[..]).expect("Decodes author raw inherent data");

		Some(Call::set_author(author))
	}

	fn check_inherent(call: &Self::Call, _data: &InherentData) -> Result<(), Self::Error> {
		// This if let should always be true. This is the only call that the inherent could make.
		if let Self::Call::set_author(claimed_author) = call {
			debug::RuntimeLogger::init();
			debug::info!("In the author inherent's `check_inherent` impl");
			ensure!(
				T::CanAuthor::can_author(&claimed_author),
				InherentError::Other(sp_runtime::RuntimeString::Borrowed("Cannot Be Author"))
			);
		}

		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	use frame_support::{
		assert_noop, assert_ok, impl_outer_origin, parameter_types,
		traits::{OnFinalize, OnInitialize},
	};
	use sp_core::H256;
	use sp_io::TestExternalities;
	use sp_runtime::{
		testing::Header,
		traits::{BlakeTwo256, IdentityLookup},
	};

	pub fn new_test_ext() -> TestExternalities {
		let t = frame_system::GenesisConfig::default()
			.build_storage::<Test>()
			.unwrap();
		TestExternalities::new(t)
	}

	impl_outer_origin! {
		pub enum Origin for Test where system = frame_system {}
	}

	mod author_inherent {
		pub use super::super::*;
	}

	impl<T> EventHandler<T> for () {
		fn note_author(_author: T) {}
	}

	#[derive(Clone, Eq, PartialEq)]
	pub struct Test;
	parameter_types! {
		pub const BlockHashCount: u64 = 250;
	}
	impl System for Test {
		type BaseCallFilter = ();
		type BlockWeights = ();
		type BlockLength = ();
		type DbWeight = ();
		type Origin = Origin;
		type Index = u64;
		type BlockNumber = u64;
		type Call = ();
		type Hash = H256;
		type Hashing = BlakeTwo256;
		type AccountId = u64;
		type Lookup = IdentityLookup<Self::AccountId>;
		type Header = Header;
		type Event = ();
		type BlockHashCount = BlockHashCount;
		type Version = ();
		type PalletInfo = ();
		type AccountData = ();
		type OnNewAccount = ();
		type OnKilledAccount = ();
		type SystemWeightInfo = ();
		type SS58Prefix = ();
	}
	impl Config for Test {
		type EventHandler = ();
		type CanAuthor = ();
	}
	type AuthorInherent = Module<Test>;
	type Sys = frame_system::Module<Test>;

	pub fn roll_to(n: u64) {
		while Sys::block_number() < n {
			Sys::on_finalize(Sys::block_number());
			Sys::set_block_number(Sys::block_number() + 1);
			Sys::on_initialize(Sys::block_number());
			AuthorInherent::on_initialize(Sys::block_number());
		}
	}

	#[test]
	fn set_author_works() {
		new_test_ext().execute_with(|| {
			assert_ok!(AuthorInherent::set_author(Origin::none(), 1));
			roll_to(1);
			assert_ok!(AuthorInherent::set_author(Origin::none(), 1));
			roll_to(2);
		});
	}

	#[test]
	fn double_author_fails() {
		new_test_ext().execute_with(|| {
			assert_ok!(AuthorInherent::set_author(Origin::none(), 1));
			assert_noop!(
				AuthorInherent::set_author(Origin::none(), 1),
				Error::<Test>::AuthorAlreadySet
			);
		});
	}
}
