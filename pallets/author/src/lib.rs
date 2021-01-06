#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	decl_error, decl_module, decl_storage, ensure, traits::FindAuthor, weights::Weight,
};
use frame_system::{ensure_none, Config as System};
use parity_scale_codec::{Decode, Encode};
#[cfg(feature = "std")]
use sp_inherents::ProvideInherentData;
use sp_inherents::{InherentData, InherentIdentifier, IsFatalError, ProvideInherent};
use sp_runtime::RuntimeString;
use sp_std::vec::Vec;

#[impl_trait_for_tuples::impl_for_tuples(30)]
pub trait EventHandler<Author> {
	/// Note that the given account ID is the author of the current block.
	fn note_author(author: Author);
}

pub trait Config: System {
	/// Find the author of a block.
	type FindAuthor: FindAuthor<Self::AccountId>;
	/// An event handler for authored blocks.
	type EventHandler: EventHandler<Self::AccountId>;
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
		}

		fn on_initialize() -> Weight {
			// Reset the author to None at the beginning of the block
			<Self as Store>::Author::kill();
			// Return zero weight because we are not using weight-based
			// transaction fees.
			0
		}

		fn on_finalize() {
			if let Some(author) = <Author<T>>::get() {
				T::EventHandler::note_author(author);
			}
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
/// Just a byte array. It will be decoded to an actual pubkey later
pub type InherentType = Vec<u8>;

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
		// Grab the Vec<u8> labelled with "author_" from the map of all inherent data
		let author_raw = data
			.get_data::<InherentType>(&INHERENT_IDENTIFIER)
			.expect("Gets and decodes authorship inherent data")?;

		// Decode the Vec<u8> into an actual author
		let author =
			T::AccountId::decode(&mut &author_raw[..]).expect("Decodes author raw inherent data");
		Some(Call::set_author(author))
	}
}
