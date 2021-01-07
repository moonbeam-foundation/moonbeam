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

use frame_support::{
	decl_error, decl_module, decl_storage, ensure, traits::FindAuthor, weights::Weight,
	ConsensusEngineId,
};
use frame_system::{ensure_none, Config as System};
use parity_scale_codec::{Decode, Encode};
#[cfg(feature = "std")]
use sp_inherents::ProvideInherentData;
use sp_inherents::{InherentData, InherentIdentifier, IsFatalError, ProvideInherent};
use sp_runtime::RuntimeString;
use sp_std::vec::Vec;

pub trait Config: System {
	//TODO event for author set.
}

decl_error! {
	pub enum Error for Module<T: Config> {
		/// Author already set in block.
		AuthorAlreadySet,
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

		/// Inherent to set the author of a block
		#[weight = 1_000_000]
		fn set_author(origin, author: T::AccountId) {
			ensure_none(origin)?;
			ensure!(<Author<T>>::get().is_none(), Error::<T>::AuthorAlreadySet);

			<Self as Store>::Author::put(author);

			// TODO we should add a digest item so Apps can detect the block author
			// should that go in on finalize?

			// TODO should we notify an event handler here? Amar had done that in on_finalize earlier.
			// I wonder if either approach is better than the other.

			// TODO we should have an event for author set.
		}

		fn on_initialize() -> Weight {
			// Reset the author to None at the beginning of the block
			<Self as Store>::Author::kill();
			// TODO how much weight should we actually be returning here.
			0
		}


		//TODO what is this?
		// fn on_finalize() {
		// 	if let Some(author) = <Author<T>>::get() {
		// 		T::EventHandler::note_author(author);
		// 	}
		// }

		//TODO we want to ensure that the inherent is set exactly once per block.
		// This is how timestamp pallet does it.
		// But there is also this provided method on the ProvideInherent trait. I wonder how it works
		// https://crates.parity.io/sp_inherents/trait.ProvideInherent.html#method.is_inherent_required
		fn on_finalize() {
			assert!(<Self as Store>::DidUpdate::take(), "Timestamp must be updated once in the block");
		}
	}
}

impl<T: Config> FindAuthor<T::AccountId> for Module<T> {
	fn find_author<'a, I>(_digests: I) -> Option<T::AccountId>
	where
		I: 'a + IntoIterator<Item = (ConsensusEngineId, &'a [u8])>,
	{
		// We don't use the digests at all.
		// That assumes an implementation, and should be removed from the trait IMO

		// This will only return the correct author _after_ the authorship inherent is processed. Is
		// it valid to assume that inherents are the first extrinsics in a block? How does timestamp
		// handle this?
		<Author<T>>::get()
	}
}

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

	fn check_inherent(call: &Self::Call, data: &InherentData) -> result::Result<(), Self::Error> {
		// TODO make sure that the current author is in the set.
		// maybe call into another pallet to confirm that.
		// Currently all authorship inherents are considered good.
		Ok(())
	}
}
