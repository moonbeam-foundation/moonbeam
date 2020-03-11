#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;
use codec::{HasCompact, Encode, Decode};
use sp_runtime::{RuntimeDebug,Perbill};
// use sp_runtime::traits::{OpaqueKeys,Convert};
use sp_runtime::traits::{Hash,Convert,SaturatedConversion,CheckedSub};
use sp_staking::offence::{OffenceDetails};
use frame_support::{decl_module, decl_storage, decl_event, decl_error, debug};
use frame_support::dispatch::{DispatchResult};
use frame_support::traits::{Currency,Get};
use system::{ensure_signed};

#[path = "../../../runtime/src/constants.rs"]
#[allow(dead_code)]
mod constants;
use constants::time::{MILLISECS_PER_YEAR,EPOCH_DURATION_IN_BLOCKS};
use constants::mb_genesis::{REWARD_PER_YEAR};

type BalanceOf<T> = 
	<<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

pub trait Trait: system::Trait + pallet_balances::Trait + pallet_session::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
	type Currency: Currency<Self::AccountId>;
	type SessionsPerEra: Get<u8>;
}

decl_storage! {
	trait Store for Module<T: Trait> as MoonbeamStakingModule {
		EraIndex: u32;

		Validators: Vec<T::AccountId>;

		// Session
		SessionOfEraIndex: u32;
		BlockOfEraIndex: u32;
		SessionValidators get(session_validators): Vec<T::AccountId>;
		SessionValidatorAuthoring: 
			map hasher(blake2_256) T::AccountId => u32;

		// validator -> endorsers
		ValidatorEndorsers: map hasher(blake2_256) T::AccountId => Vec<T::AccountId>;

		// endorser => validator
		Endorser: map hasher(blake2_256) T::AccountId => T::AccountId; 

		/// A timeline of free_balances for an endorser that allows us to calculate
		/// the average of free_balance of an era.
		/// 
		/// TODO: Used to select era validators at the start (or end TODO) of an era. 
		/// We are by now supposing that an endorsement represents all the free_balance of the token holder.
		/// When the free_balance of an endorser changes, a new snapshot is created together with the current block_index of the current era.
		/// 
		/// Endorser, Validator => (session_block_index,endorser_balance)
		EndorserSnapshots:
			double_map hasher(blake2_256) T::AccountId, hasher(blake2_256) T::AccountId => Vec<(u32,BalanceOf<T>)>;

		// Treasury
		Treasury get(treasury): T::Balance;
	}
    add_extra_genesis {
		config(session_validators): Vec<T::AccountId>;
		config(treasury): T::Balance;
        build(|config: &GenesisConfig<T>| {
			// set all validators
			let _ = <Validators<T>>::append(config.session_validators.clone());
			// set initial selected validators
			let _ = <SessionValidators<T>>::append(config.session_validators.clone());
			// set treasury
			<Treasury<T>>::put(config.treasury);
			// set genesis era
			EraIndex::put(1);
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
		EndSession(u32),
	}
);

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {

		type Error = Error<T>;
		fn deposit_event() = default;

		pub fn endorse(
			origin, to:T::AccountId
		) -> DispatchResult {
			let from = ensure_signed(origin)?;
			if <Endorser<T>>::contains_key(&from) {
				// TODO already endorsing
			}
			<Endorser<T>>::insert(&from,&to);
			<ValidatorEndorsers<T>>::append(&to,vec![&from])?;

			Self::snapshot(&from,&to,T::Currency::free_balance(&from))?;

			Ok(())
		}

		pub fn unendorse(
			origin
		) -> DispatchResult {
			let from = ensure_signed(origin)?;
			if !<Endorser<T>>::contains_key(&from) {
				// TODO is not endorsing
			}
			let validator = <Endorser<T>>::get(&from);
			let mut endorsers = <ValidatorEndorsers<T>>::get(&validator);
			
			// remove from validator's endorser list
			endorsers.retain(|x| x != &from);
			<ValidatorEndorsers<T>>::insert(&validator, endorsers);
			// remove as an endorser
			<Endorser<T>>::remove(&from);
			// remove all snapshots associated to the endorser
			<EndorserSnapshots<T>>::remove_prefix(&from);

			// Self::snapshot(&from,&validator,BalanceOf::<T>::from(0))?;

			Ok(())
		}
	}
}

impl<T: Trait> Module<T> {
	fn snapshot(
		endorser: &T::AccountId, validator: &T::AccountId, amount: BalanceOf<T>
	) -> DispatchResult {
		<EndorserSnapshots<T>>::append(&endorser,&validator,
			vec![(
				BlockOfEraIndex::get(),
				amount
			)])?;
		Ok(())
	}

	fn reset_snapshots() {
		let validators = <Validators<T>>::get();
		for validator in validators.iter() {
			let endorsers = <ValidatorEndorsers<T>>::get(validator);
			for endorser in endorsers.iter() {
				<EndorserSnapshots<T>>::insert(endorser,validator,vec![(
					1 as u32,
					T::Currency::free_balance(endorser)
				)]);
			}
		}
	}
}

pub struct SessionManager<T>(T);
impl<T: Trait> pallet_session::SessionManager<T::AccountId> for SessionManager<T> {
	fn new_session(new_index: u32) -> Option<Vec<T::AccountId>> {

		<Module<T>>::deposit_event(
			RawEvent::NewSession(new_index)
		);

		if new_index > 1 {
			let current_era = EraIndex::get();
			if new_index > (current_era * (T::SessionsPerEra::get() as u32)) {
				// Era change. Reset SessionOfEraIndex to 1, increase the EraIndex by 1
				SessionOfEraIndex::put(1);

				// Reset BlockOfEraIndex to 1
				BlockOfEraIndex::put(1);

				let new_era_idx = EraIndex::get().checked_add(1)
					.ok_or("SessionOfEraIndex Overflow").unwrap();
					
				EraIndex::put(new_era_idx);

				<Module<T>>::reset_snapshots();
				// TODO reset snapshots
					
				// TODO new validator set?
			} else {
				// Same Era, next session. Increase SessionOfEraIndex by 1.
				let new_era_session_idx = SessionOfEraIndex::get().checked_add(1)
					.ok_or("SessionOfEraIndex Overflow").unwrap();
				SessionOfEraIndex::put(new_era_session_idx);
			}
			Some(<SessionValidators<T>>::get())
		} else {
			None
		}
	}

	fn end_session(end_index: u32) {
		<Module<T>>::deposit_event(
			RawEvent::EndSession(end_index)
		);
	}
}

pub struct AuthorshipEventHandler<T>(T);
impl<T: Trait> pallet_authorship::EventHandler<T::AccountId,u32> for AuthorshipEventHandler<T> {
	fn note_author(author: T::AccountId) {
		let authored_blocks = 
			<SessionValidatorAuthoring<T>>::get(&author).checked_add(1).ok_or("Overflow").unwrap();
		<SessionValidatorAuthoring<T>>::insert(&author,authored_blocks);
		BlockOfEraIndex::mutate(|x| *x += 1);
		// <Module<T>>::deposit_event(
		// 	RawEvent::BlockAuthored(author)
		// );
	}
	fn note_uncle(_author: T::AccountId, _age: u32) {
		
	}
}

// All below are trait implemenations that we need to satisfy for the historical feature of the pallet-session
// and by offences. Find a way to remove without having to implement.

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

impl<T: Trait> Convert<T::AccountId, Option<Exposure<T::AccountId, BalanceOf<T>>>>
	for ExposureOf<T>
{
	fn convert(_validator: T::AccountId) -> Option<Exposure<T::AccountId, BalanceOf<T>>> {
		None
	}
}

pub struct Offences<T>(sp_std::marker::PhantomData<T>);
impl <T: Trait> sp_staking::offence::OnOffenceHandler<T::AccountId, pallet_session::historical::IdentificationTuple<T>> for Offences<T> where
	T: pallet_session::Trait<ValidatorId = <T as system::Trait>::AccountId>,
	T: pallet_session::historical::Trait<
		FullIdentification = Exposure<<T as system::Trait>::AccountId, BalanceOf<T>>,
		FullIdentificationOf = ExposureOf<T>,
	>,
	T::SessionHandler: pallet_session::SessionHandler<<T as system::Trait>::AccountId>,
	T::SessionManager: pallet_session::SessionManager<<T as system::Trait>::AccountId>,
	T::ValidatorIdOf: Convert<<T as system::Trait>::AccountId, Option<<T as system::Trait>::AccountId>>
{
	fn on_offence(
		_offenders: &[OffenceDetails<T::AccountId, pallet_session::historical::IdentificationTuple<T>>],
		_slash_fraction: &[Perbill],
		_slash_session: u32,
	) {
		
	}
}

// struct SessionHandler<T>(T);
// impl<T: Trait> pallet_session::SessionHandler<T::AccountId> for SessionHandler<T> {

// 	const KEY_TYPE_IDS: &'static [KeyTypeId] = &[];

// 	fn on_genesis_session<Ks: OpaqueKeys>(_validators: &[(T::AccountId, Ks)]) {}

// 	fn on_new_session<Ks: OpaqueKeys>(
// 		_changed: bool,
// 		validators: &[(T::AccountId, Ks)],
// 		_queued_validators: &[(T::AccountId, Ks)],
// 	) {
// 		SessionIndex::mutate(|x| *x + 1);
// 		let current_session = SessionIndex::get();
// 		<Module<T>>::deposit_event(
// 			RawEvent::NewSessionIndex(current_session)
// 		);
// 	}

// 	fn on_disabled(validator_index: usize) {
		
// 	}
// }