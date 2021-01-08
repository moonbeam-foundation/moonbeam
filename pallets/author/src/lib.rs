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
	decl_event, decl_error, decl_module, decl_storage, ensure, weights::Weight,
};
use frame_system::{ensure_none, Config as System};
use parity_scale_codec::{Decode, Encode};
#[cfg(feature = "std")]
use sp_inherents::ProvideInherentData;
use sp_inherents::{InherentData, InherentIdentifier, IsFatalError, ProvideInherent};
use sp_runtime::RuntimeString;
use sp_std::vec::Vec;
use pallet_authorship::EventHandler;

pub trait Config: System {
	/// Event type used by the runtime.
	type Event: From<Event<Self>> + Into<<Self as System>::Event>;

	/// Other pallets that want to be informed about block authorship.
	/// We reuse the `pallet_authorship::EventHandler` for easy compatability with existing pallets.
	type EventHandler: EventHandler<Self::AccountId, Self::BlockNumber>;
}

decl_event! {
	pub enum Event<T> where
		AccountId = <T as System>::AccountId,
		BlockNumber = <T as System>::BlockNumber,
	{
		/// Set Author. Fields are Author, Block height
		AuthorSet(AccountId, BlockNumber),
	}
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

		/// Did the author get set in this block?
		DidUpdate: bool;
	}
}

decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;

		/// Inherent to set the author of a block
		#[weight = 1_000_000]
		fn set_author(origin, author: T::AccountId) {
			ensure_none(origin)?;
			ensure!(<Author<T>>::get().is_none(), Error::<T>::AuthorAlreadySet);

			let current_block = frame_system::Module::<T>::block_number();

			// Update storage
			Author::<T>::put(&author);
			DidUpdate::put(true);

			// TODO we should add a digest item so Apps can detect the block author

			// Notify any other pallets that are listening (eg rewards) about the author
			T::EventHandler::note_author(author.clone());

			Self::deposit_event(Event::<T>::AuthorSet(author, current_block));
		}

		fn on_initialize() -> Weight {
			// Reset the author at the beginning of the block
			Author::<T>::kill();

			// TODO how much weight should we actually be returning here.
			0
		}

		fn on_finalize() {
			// Ensure that the inherent is included in each block.
			// This is how timestamp pallet does it.
			// But there is also this provided method on the ProvideInherent trait.
			// I wonder how it works
			assert!(<Self as Store>::DidUpdate::take(), "Author inherent must be in the block");
		}
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

	fn check_inherent(call: &Self::Call, data: &InherentData) -> Result<(), Self::Error> {
		// TODO make sure that the current author is in the set.
		// maybe call into another pallet to confirm that.
		// Currently all authorship inherents are considered good.
		// Maybe this checking logic should be decoupled somehow so I can hand it off to the stake pallet.
		Ok(())
	}
}
