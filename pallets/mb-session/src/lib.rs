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

use codec::{Decode, Encode};
use frame_support::dispatch::DispatchResult;
use frame_support::traits::{Currency, Get};
use frame_support::{debug, decl_error, decl_event, decl_module, decl_storage};
use sp_runtime::traits::SaturatedConversion;
use sp_runtime::transaction_validity::{
	InvalidTransaction, TransactionPriority, TransactionSource, TransactionValidity,
	ValidTransaction,
};
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;
use system::{
	self as system, ensure_none, ensure_signed,
	offchain::{
		AppCrypto, CreateSignedTransaction, SendUnsignedTransaction, SignedPayload, Signer,
	},
};

use sp_core::crypto::KeyTypeId;

#[path = "../../../runtime/src/constants.rs"]
#[allow(dead_code)]
mod constants;
use constants::mb_genesis::VALIDATORS_PER_SESSION;
use constants::time::EPOCH_DURATION_IN_BLOCKS;

pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"mbst");

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

pub mod crypto {
	pub use super::KEY_TYPE;
	use sp_core::sr25519::Signature as Sr25519Signature;
	use sp_runtime::{
		app_crypto::{app_crypto, sr25519},
		traits::Verify,
		MultiSignature, MultiSigner,
	};
	app_crypto!(sr25519, KEY_TYPE);

	pub struct AuthId;
	impl system::offchain::AppCrypto<<Sr25519Signature as Verify>::Signer, Sr25519Signature>
		for AuthId
	{
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::sr25519::Signature;
		type GenericPublic = sp_core::sr25519::Public;
	}

	impl system::offchain::AppCrypto<MultiSigner, MultiSignature> for AuthId {
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::sr25519::Signature;
		type GenericPublic = sp_core::sr25519::Public;
	}
}

pub trait Trait:
	system::Trait + pallet_balances::Trait + pallet_session::Trait + CreateSignedTransaction<Call<Self>>
{
	type AuthorityId: AppCrypto<Self::Public, Self::Signature>;
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
	type Call: From<Call<Self>>;
	type Currency: Currency<Self::AccountId>;
	type SessionsPerEra: Get<u8>;
	type UnsignedPriority: Get<TransactionPriority>;
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct SnapshotsPayload<AccountId, Public, BlockNumber, BalanceOf> {
	block_number: BlockNumber,
	snapshots: Vec<(AccountId, AccountId, BalanceOf)>,
	public: Public,
}

impl<T: Trait> SignedPayload<T>
	for SnapshotsPayload<T::AccountId, T::Public, T::BlockNumber, BalanceOf<T>>
{
	fn public(&self) -> T::Public {
		self.public.clone()
	}
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct ValidatorsPayload<AccountId, Public, BlockNumber> {
	block_number: BlockNumber,
	validators: Vec<AccountId>,
	public: Public,
}

impl<T: Trait> SignedPayload<T> for ValidatorsPayload<T::AccountId, T::Public, T::BlockNumber> {
	fn public(&self) -> T::Public {
		self.public.clone()
	}
}

decl_storage! {
	trait Store for Module<T: Trait> as MoonbeamStakingModule {
		/// The number of Era.
		EraIndex: u32;
		/// The total validator pool.
		Validators: Vec<T::AccountId>;
		/// The current session of an Era.
		SessionOfEraIndex: u32;
		/// The current Block Index of an Era.
		BlockOfEraIndex: u32;
		/// The validator set selected for the Era.
		SessionValidators get(fn session_validators): Vec<T::AccountId>;
		/// Number of blocks authored by a given validator in this Era.
		SessionValidatorAuthoring:
			map hasher(blake2_128_concat) T::AccountId => u32;
		/// One to Many Validator -> Endorsers.
		ValidatorEndorsers: map hasher(blake2_128_concat) T::AccountId => Vec<T::AccountId>;
		/// One to One Endorser -> Validator.
		/// A restriction for number of endorsers per validator must be implemented to predict complexity.
		Endorser: map hasher(blake2_128_concat) T::AccountId => T::AccountId;
		/// A timeline of free_balances for an endorser that allows us to calculate
		/// the average of free_balance of an era.
		///
		/// TODO: Used to select era validators at the start (or end?) of an era.
		/// We are by now supposing that an endorsement represents all the free_balance
		/// of the token holder.
		/// When the free_balance of an endorser changes, a new snapshot is created
		/// together with the current block_index of the current era.
		///
		/// Endorser, Validator => (session_block_index,endorser_balance)
		EndorserSnapshots:
			double_map hasher(blake2_128_concat) T::AccountId, hasher(blake2_128_concat) T::AccountId => Vec<(u32,BalanceOf<T>)>;

		/// TODO the Treasury balance. It is still unclear if this will be a pallet account or
		/// will remain as a Storage balance.
		Treasury get(fn treasury): T::Balance;
	}
	add_extra_genesis {
		config(session_validators): Vec<T::AccountId>;
		config(treasury): T::Balance;
		build(|config: &GenesisConfig<T>| {
			// set all validators
			let _ = <Validators<T>>::put(config.session_validators.clone());
			// set initial selected validators
			let _ = <SessionValidators<T>>::put(config.session_validators.clone());
			// set treasury
			<Treasury<T>>::put(config.treasury);
			// set genesis era data
			EraIndex::put(1);
			BlockOfEraIndex::put(1);
		});
	}
}

decl_error! {
	pub enum Error for Module<T: Trait> {
		AlreadyEndorsing,
		NotEndorsing,
	}
}

decl_event!(
	pub enum Event<T>
	where
		AccountId = <T as system::Trait>::AccountId,
	{
		BlockAuthored(AccountId),
		NewEra(u32),
		NewSession(u32),
		StartSession(u32),
		EndSession(u32),
	}
);

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {

		type Error = Error<T>;
		fn deposit_event() = default;

		/// Endorsing dispatchable function
		#[weight = 0]
		pub fn endorse(
			origin, to:T::AccountId
		) -> DispatchResult {
			let from = ensure_signed(origin)?;
			// Check if the Account is already endorsing.
			if <Endorser<T>>::contains_key(&from) {
				return Err(Error::<T>::AlreadyEndorsing).map_err(Into::into);
			}
			// Set One to One endorser->validator association.
			<Endorser<T>>::insert(&from,&to);
			// Set One to Many validator->endorsers association.
			<ValidatorEndorsers<T>>::append(&to,&from);
			// Create a snapshot with the current free balance of the endorser.
			Self::set_snapshot(&from,&to,T::Currency::free_balance(&from))?;
			Ok(())
		}

		/// Unndorsing dispatchable function
		#[weight = 0]
		pub fn unendorse(
			origin
		) -> DispatchResult {
			let from = ensure_signed(origin)?;
			// Check if the Account is actively endorsing.
			if !<Endorser<T>>::contains_key(&from) {
				return Err(Error::<T>::NotEndorsing).map_err(Into::into);
			}

			let validator = <Endorser<T>>::get(&from);
			let mut endorsers = <ValidatorEndorsers<T>>::get(&validator);

			// Remove One to Many validator->endorsers association
			endorsers.retain(|x| x != &from);
			<ValidatorEndorsers<T>>::insert(&validator, endorsers);
			// Remove One to One endorser->validator association
			<Endorser<T>>::remove(&from);
			// Remove all snapshots associated to the endorser, using the double map prefix
			<EndorserSnapshots<T>>::remove_prefix(&from);

			Ok(())
		}

		fn offchain_worker(block_number: T::BlockNumber) {
			// Set snapshots
			let _ = Self::offchain_set_snapshots(block_number);
			// Select validators off-chain
			let _ = Self::offchain_validator_selection(block_number);
		}

		/// Set new selected validators. Called offchain as an unsigned transaction.
		#[weight = 0]
		fn persist_selected_validators(
			origin,
			validators_payload: ValidatorsPayload<T::AccountId, T::Public, T::BlockNumber>,
			_signature: T::Signature
		) -> DispatchResult {
			ensure_none(origin)?;
			debug::native::info!("##### Offchain validators:");
			debug::native::info!("> {:#?}",validators_payload.validators.clone());
			<SessionValidators<T>>::put(validators_payload.validators.clone());
			Ok(())
		}

		/// Set new snapshots. Called offchain as an unsigned transaction.
		#[weight = 0]
		fn persist_snapshots(
			origin,
			snapshots_payload: SnapshotsPayload<T::AccountId, T::Public, T::BlockNumber, BalanceOf<T>>,
			_signature: T::Signature
		) -> DispatchResult {
			ensure_none(origin)?;
			debug::native::info!("##### Offchain snapshots:");
			debug::native::info!("> {:#?}",snapshots_payload.snapshots);
			for s in &snapshots_payload.snapshots {
				Self::set_snapshot(&s.0,&s.1,s.2)?;
			}
			Ok(())
		}
	}
}

impl<T: Trait> Module<T> {
	/// Offchain task to select validators
	fn offchain_validator_selection(block_number: T::BlockNumber) -> Result<(), &'static str> {
		// Find out where we are in Era
		let current_era: u128 = EraIndex::get() as u128;
		let last_block_of_era: u128 =
			(current_era * (T::SessionsPerEra::get() as u128) * (EPOCH_DURATION_IN_BLOCKS as u128))
				.saturated_into();
		let validator_selection_delta: u128 = 5;
		let current_block_number: u128 = block_number.saturated_into();
		// When we are 5 blocks away of a new Era, run the validator selection.
		if (last_block_of_era - current_block_number) == validator_selection_delta {
			// Perform the validator selection
			let validators = <Module<T>>::select_validators();
			// Submit unsigned transaction with signed payload
			let (_, result) = Signer::<T, T::AuthorityId>::any_account()
				.send_unsigned_transaction(
					|account| ValidatorsPayload {
						validators: validators.clone(),
						block_number,
						public: account.public.clone(),
					},
					|payload, signature| Call::persist_selected_validators(payload, signature),
				)
				.ok_or("No local accounts accounts available.")?;
			result.map_err(|()| "Unable to submit transaction")?;
		}

		Ok(())
	}

	/// The below is TODO, just a 1st approach to keep moving forward until we find out how to track BalanceOf
	/// changes in real-time.
	///
	/// This approach, although functional, is invalid as it has multiple issues like
	/// sending signed transactions potentially every block.
	///
	/// Other messy ways could be, again a per-block offchain task, pattern matching the
	/// <system::Module<T>>::events() to find pallet_balances events for the current block?
	fn offchain_set_snapshots(block_number: T::BlockNumber) -> Result<(), &'static str> {
		let mut snapshots: Vec<(T::AccountId, T::AccountId, BalanceOf<T>)> = vec![];
		let validators = <Validators<T>>::get();
		for v in &validators {
			let endorsers = <ValidatorEndorsers<T>>::get(v);
			for ed in &endorsers {
				let snapshots_tmp = <EndorserSnapshots<T>>::get(ed, v.clone());
				let len = snapshots_tmp.len();
				// Make sure we have a previous block reference in this Era
				if len > 0 {
					let snapshot_balance = snapshots_tmp[len - 1].1;
					let current_balance = T::Currency::free_balance(ed);
					if snapshot_balance != current_balance {
						snapshots.push((ed.clone(), v.clone(), current_balance));
					}
				}
			}
		}
		// If there are snapshots, send unsigned transaction with signed payload
		if snapshots.len() > 0 {
			// Submit unsigned transaction with signed payload
			let (_, result) = Signer::<T, T::AuthorityId>::any_account()
				.send_unsigned_transaction(
					|account| SnapshotsPayload {
						snapshots: snapshots.clone(),
						block_number,
						public: account.public.clone(),
					},
					|payload, signature| Call::persist_snapshots(payload, signature),
				)
				.ok_or("No local accounts accounts available.")?;
			result.map_err(|()| "Unable to submit transaction")?;
		}
		Ok(())
	}

	/// Sets a snapshot using the current era's block index and the Account free_balance.
	fn set_snapshot(
		endorser: &T::AccountId,
		validator: &T::AccountId,
		amount: BalanceOf<T>,
	) -> DispatchResult {
		<EndorserSnapshots<T>>::append(&endorser, &validator, (BlockOfEraIndex::get(), amount));
		Ok(())
	}
	/// Calculates a single endorser weighted balance for the era by measuring the
	/// block index distances.
	fn calculate_endorsement(endorser: &T::AccountId, validator: &T::AccountId) -> f64 {
		let duration: u32 = EPOCH_DURATION_IN_BLOCKS;
		let points = <EndorserSnapshots<T>>::get(endorser, validator);
		let points_dim: usize = points.len();

		if points_dim == 0 {
			return 0 as f64;
		}

		let n: usize = points_dim - 1;
		let (points, n) = Self::set_snapshot_boundaries(duration, n, points);
		let mut previous: (u32, BalanceOf<T>) = (0, BalanceOf::<T>::from(0));
		// Find the distances between snapshots, weight the free_balance against them.
		// Finally sum all values.
		let mut endorsement: f64 = points
			.iter()
			.map(|p| {
				let out: f64;
				if previous != (0, BalanceOf::<T>::from(0)) {
					let delta = p.0 - previous.0;
					let w = delta as f64 / duration as f64;
					out = w * previous.1.saturated_into() as f64;
				} else {
					out = 0 as f64;
				}
				previous = *p;
				out
			})
			.sum::<f64>();
		// The above iterative approach excludes the last block, sum it to the result.
		endorsement += (1 as f64 / duration as f64) * points[n].1.saturated_into() as f64;
		endorsement
	}
	/// Selects a new validator set based on their amount of weighted endorsement.
	fn select_validators() -> Vec<T::AccountId> {
		let validators = <Validators<T>>::get();
		// Get the calculated endorsement per validator.
		let mut selected_validators = validators
			.iter()
			.map(|v| {
				let endorsers = <ValidatorEndorsers<T>>::get(v);
				let total_validator_endorsement = endorsers
					.iter()
					.map(|ed| Self::calculate_endorsement(ed, v))
					.sum::<f64>();
				(total_validator_endorsement, v)
			})
			.collect::<Vec<_>>();
		// Sort descendant validators by amount.
		selected_validators.sort_by(|(x0, _y0), (x1, _y1)| x0.partial_cmp(&x1).unwrap());
		selected_validators.reverse();
		// Take the by-configuration amount of validators.
		selected_validators
			.into_iter()
			.take(VALIDATORS_PER_SESSION as usize)
			.map(|(_x, y)| y.clone())
			.collect::<Vec<_>>()
	}
	/// Conditionally set the boundary balances to complete a snapshot series.
	/// (if no snapshot is defined on block 1 or {era_len} indexes).
	fn set_snapshot_boundaries(
		duration: u32,
		mut last_index: usize,
		mut collection: Vec<(u32, BalanceOf<T>)>,
	) -> (Vec<(u32, BalanceOf<T>)>, usize) {
		if collection[0].0 != 1 {
			collection.insert(0, (1, BalanceOf::<T>::from(0)));
			last_index += 1;
		}
		if collection[last_index].0 != duration {
			collection.push((duration, collection[last_index].1));
		}
		(collection, last_index)
	}
	/// All snapshots are reset on era change with a single checkpoint of the
	/// current endorser's Account free_balance.
	fn reset_snapshots() {
		let validators = <Validators<T>>::get();
		for validator in validators.iter() {
			let endorsers = <ValidatorEndorsers<T>>::get(validator);
			for endorser in endorsers.iter() {
				<EndorserSnapshots<T>>::insert(
					endorser,
					validator,
					vec![(1 as u32, T::Currency::free_balance(endorser))],
				);
			}
		}
	}

	/// TODO, needs the right config
	fn validate_validator_transaction(_validators: &Vec<T::AccountId>) -> TransactionValidity {
		ValidTransaction::with_tag_prefix("MbSession")
			.priority(T::UnsignedPriority::get())
			.and_provides(0)
			.longevity(5)
			.propagate(true)
			.build()
	}

	/// TODO, needs the right config
	fn validate_snapshots_transaction(
		_snapshots: &Vec<(T::AccountId, T::AccountId, BalanceOf<T>)>,
	) -> TransactionValidity {
		ValidTransaction::with_tag_prefix("MbSession")
			.priority(T::UnsignedPriority::get())
			.and_provides(0)
			.longevity(5)
			.propagate(true)
			.build()
	}
}

pub struct SessionManager<T>(T);
impl<T: Trait> pallet_session::SessionManager<T::AccountId> for SessionManager<T> {
	fn new_session(new_index: u32) -> Option<Vec<T::AccountId>> {
		<Module<T>>::deposit_event(RawEvent::NewSession(new_index));

		if new_index > 1 {
			let current_era = EraIndex::get();
			if new_index > (current_era * (T::SessionsPerEra::get() as u32)) {
				// Era change
				// Reset SessionOfEraIndex to 1
				SessionOfEraIndex::put(1);
				// Reset BlockOfEraIndex to 1
				BlockOfEraIndex::put(1);
				// Increase the EraIndex by 1
				let new_era_idx = EraIndex::get()
					.checked_add(1)
					.ok_or("SessionOfEraIndex Overflow")
					.unwrap();
				EraIndex::put(new_era_idx);
				// Reset all snapshots
				<Module<T>>::reset_snapshots();
			} else {
				// Same Era, next session. Increase SessionOfEraIndex by 1.
				let new_era_session_idx = SessionOfEraIndex::get()
					.checked_add(1)
					.ok_or("SessionOfEraIndex Overflow")
					.unwrap();
				SessionOfEraIndex::put(new_era_session_idx);
			}
			Some(<SessionValidators<T>>::get())
		} else {
			None
		}
	}

	fn end_session(end_index: u32) {
		<Module<T>>::deposit_event(RawEvent::EndSession(end_index));
	}

	fn start_session(start_index: u32) {
		<Module<T>>::deposit_event(RawEvent::StartSession(start_index));
	}
}

pub struct AuthorshipEventHandler<T>(T);
impl<T: Trait> pallet_authorship::EventHandler<T::AccountId, u32> for AuthorshipEventHandler<T> {
	fn note_author(author: T::AccountId) {
		let authored_blocks = <SessionValidatorAuthoring<T>>::get(&author)
			.checked_add(1)
			.ok_or("Overflow")
			.unwrap();
		<SessionValidatorAuthoring<T>>::insert(&author, authored_blocks);
		BlockOfEraIndex::mutate(|x| *x += 1);
		// <Module<T>>::deposit_event(
		// 	RawEvent::BlockAuthored(author)
		// );
	}
	fn note_uncle(_author: T::AccountId, _age: u32) {}
}

#[allow(deprecated)] // ValidateUnsigned
impl<T: Trait> frame_support::unsigned::ValidateUnsigned for Module<T> {
	type Call = Call<T>;
	fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
		if let Call::persist_selected_validators(ref payload, ref signature) = call {
			let signature_valid =
				SignedPayload::<T>::verify::<T::AuthorityId>(payload, signature.clone());
			if !signature_valid {
				return InvalidTransaction::BadProof.into();
			}
			Self::validate_validator_transaction(&payload.validators)
		} else if let Call::persist_snapshots(ref payload, ref signature) = call {
			let signature_valid =
				SignedPayload::<T>::verify::<T::AuthorityId>(payload, signature.clone());
			if !signature_valid {
				return InvalidTransaction::BadProof.into();
			}
			Self::validate_snapshots_transaction(&payload.snapshots)
		} else {
			InvalidTransaction::Call.into()
		}
	}
}
