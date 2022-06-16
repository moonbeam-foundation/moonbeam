// Copyright 2019-2022 PureStake Inc.
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

//! VRF Pallet
//!
//! Stores VRF output per block by the block author as well as the VRF inputs:
//! `most_recent_storage_root + most_recent_slot_number`
//!

#![cfg_attr(not(feature = "std"), no_std)]
use nimbus_primitives::{NimbusId, NIMBUS_ENGINE_ID};
use session_keys_primitives::{InherentError, KeysLookup, VrfId, INHERENT_IDENTIFIER};
use sp_application_crypto::ByteArray;
use sp_consensus_babe::{digests::PreDigest, Slot, Transcript, BABE_ENGINE_ID};
use sp_consensus_vrf::schnorrkel;

// #[cfg(test)]
// mod mock;
// #[cfg(test)]
// mod tests;

use frame_support::pallet;

pub use pallet::*;

type Randomness = schnorrkel::Randomness;

#[pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	/// VRF inputs from the relay chain
	/// Both inputs are expected to change every block
	#[derive(Default, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
	pub struct VrfInput<SlotNumber, RelayHash> {
		/// Relay block slot number
		pub slot_number: SlotNumber,
		/// Relay block storage root
		pub storage_root: RelayHash,
	}

	/// For the runtime to implement to expose cumulus data to this pallet and cost of getting data
	pub trait GetVrfInputs<SlotNumber, StorageRoot> {
		/// Returns most recent relay slot number and weight consumed by get
		fn get_slot_number() -> SlotNumber;
		/// Returns most recent relay storage root and weight consumed by get
		fn get_storage_root() -> StorageRoot;
	}

	/// To set the relay chain data in `pallet-randomness`, to be implemented in the runtime
	pub trait SetRelayRandomness {
		fn set_relay_randomness();
	}

	/// Exposes randomness in this pallet to the runtime
	pub trait MaybeGetRandomness<R> {
		fn maybe_get_randomness() -> Option<R>;
	}

	/// Make VRF transcript
	pub fn make_transcript<Hash: AsRef<[u8]>>(input: VrfInput<Slot, Hash>) -> Transcript {
		let mut transcript = Transcript::new(&BABE_ENGINE_ID);
		transcript.append_u64(b"relay slot number", *input.slot_number);
		transcript.append_message(b"relay storage root", input.storage_root.as_ref());
		transcript
	}

	/// Pallet for storing and exposing VRF outputs by block authors
	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Gets the most recent relay block hash and relay slot number
		type VrfInputs: GetVrfInputs<Slot, Self::Hash>;
		/// Takes NimbusId to return VrfId
		type VrfKeyLookup: KeysLookup<NimbusId, VrfId>;
		/// Handler to set the babe relay randomness in `pallet-randomness`
		type BabeDataSetter: SetRelayRandomness;
	}

	/// Current block randomness
	/// Set in `on_initialize`, before it will contain the randomness for this block
	#[pallet::storage]
	#[pallet::getter(fn current_randomness)]
	pub type CurrentRandomness<T> = StorageValue<_, Randomness>;

	/// Most recent VRF input from relay chain data
	/// Set in `on_initialize` before setting randomness
	#[pallet::storage]
	#[pallet::getter(fn current_vrf_input)]
	pub(crate) type CurrentVrfInput<T: Config> = StorageValue<_, VrfInput<Slot, T::Hash>>;

	/// Last block's VRF input relay chain data
	/// Used in on_initialize as the VRF inputs for this block
	#[pallet::storage]
	#[pallet::getter(fn last_vrf_input)]
	pub(crate) type LastVrfInput<T: Config> = StorageValue<_, VrfInput<Slot, T::Hash>>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// This inherent is a workaround to run code after the "real" inherents have executed,
		/// but before transactions are executed.
		// This should go into on_post_inherents when it is ready
		// https://github.com/paritytech/substrate/pull/10128
		// TODO weight
		#[pallet::weight((1_000 + 7 * T::DbWeight::get().write, DispatchClass::Mandatory))]
		pub fn set_vrf_inputs(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			ensure_none(origin)?;

			let storage_root = T::VrfInputs::get_storage_root();
			let slot_number = T::VrfInputs::get_slot_number();
			if let Some(last_vrf_inputs) = <CurrentVrfInput<T>>::take() {
				// logs if input uniqueness assumptions are violated (no reuse of vrf inputs)
				if last_vrf_inputs.storage_root == storage_root
					|| last_vrf_inputs.slot_number == slot_number
				{
					log::warn!(
						"VRF on_initialize: storage root or slot number did not change between \
					current and last block. Nimbus would've panicked if slot number did not change \
					so probably storage root did not change."
					);
				}
				<LastVrfInput<T>>::put(last_vrf_inputs);
			}
			let inputs = VrfInput {
				slot_number,
				storage_root,
			};
			<CurrentVrfInput<T>>::put(inputs);
			T::BabeDataSetter::set_relay_randomness();

			Ok(Pays::No.into())
		}
	}

	#[pallet::inherent]
	impl<T: Config> ProvideInherent for Pallet<T> {
		type Call = Call<T>;
		type Error = InherentError;
		const INHERENT_IDENTIFIER: InherentIdentifier = INHERENT_IDENTIFIER;

		fn is_inherent_required(_: &InherentData) -> Result<Option<Self::Error>, Self::Error> {
			// Return Ok(Some(_)) unconditionally because this inherent is required in every block
			// If it is not found, throw a VrfInherentRequired error.
			Ok(Some(InherentError::Other(
				sp_runtime::RuntimeString::Borrowed("Inherent required to set VRF inputs"),
			)))
		}

		// The empty-payload inherent extrinsic.
		fn create_inherent(_data: &InherentData) -> Option<Self::Call> {
			Some(Call::set_vrf_inputs {})
		}

		fn is_inherent(call: &Self::Call) -> bool {
			matches!(call, Call::set_vrf_inputs { .. })
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		// Set this block's randomness using the VRF output
		fn on_initialize(_now: BlockNumberFor<T>) -> Weight {
			// TODO: how is this set the first block in the runtime upgrade that includes this code
			// what is the genesis VRF input and how do we populate LastVrfInput with this value
			let vrf_input = <LastVrfInput<T>>::get()
				.expect("Expect to be set in `set_vrf_inputs` inherent prior to on_initialize");
			Self::set_randomness(vrf_input)
		}
		// Ensure next block's VRF input is set in storage
		fn on_finalize(_now: BlockNumberFor<T>) {
			assert!(
				<CurrentVrfInput<T>>::get().is_some(),
				"Current VRF input is not set in this block so it cannot be used as input for
				the VRF output in the next block"
			);
		}
	}

	impl<T: Config> Pallet<T> {
		/// Returns weight consumed in `on_initialize`
		fn set_randomness(input: VrfInput<Slot, T::Hash>) -> Weight {
			let mut block_author_vrf_id: Option<VrfId> = None;
			let maybe_pre_digest: Option<PreDigest> = <frame_system::Pallet<T>>::digest()
				.logs
				.iter()
				.filter_map(|s| s.as_pre_runtime())
				.filter_map(|(id, mut data)| {
					if id == BABE_ENGINE_ID {
						PreDigest::decode(&mut data).ok()
					} else {
						if id == NIMBUS_ENGINE_ID {
							let nimbus_id = NimbusId::decode(&mut data)
								.expect("NimbusId encoded in pre-runtime digest must be valid");

							block_author_vrf_id = Some(
								T::VrfKeyLookup::lookup_keys(&nimbus_id)
									.expect("No VRF Key Mapped to this NimbusId"),
							);
						}
						None
					}
				})
				.next();
			let block_author_vrf_id =
				block_author_vrf_id.expect("VrfId encoded in pre-runtime digest must be valid");
			let pubkey = schnorrkel::PublicKey::from_bytes(block_author_vrf_id.as_slice())
				.expect("Expect VrfId to be valid schnorrkel public key");
			let transcript = make_transcript::<T::Hash>(input);
			let randomness: Randomness = maybe_pre_digest
				.and_then(|digest| {
					digest
						.vrf_output()
						.and_then(|vrf_output| {
							vrf_output.0.attach_input_hash(&pubkey, transcript).ok()
						})
						.map(|inout| inout.make_bytes(&sp_consensus_babe::BABE_VRF_INOUT_CONTEXT))
				})
				.expect("VRF output encoded in pre-runtime digest must be valid");
			CurrentRandomness::<T>::put(randomness);
			T::DbWeight::get().read + 2 * T::DbWeight::get().write
		}
	}

	impl<T: Config> MaybeGetRandomness<T::Hash> for Pallet<T> {
		fn maybe_get_randomness() -> Option<T::Hash> {
			if let Some(r) = CurrentRandomness::<T>::get() {
				T::Hash::decode(&mut &r[..]).ok()
			} else {
				None
			}
		}
	}
}
