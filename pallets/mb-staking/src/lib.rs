#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;
use codec::{HasCompact, Encode, Decode};
use sp_runtime::{RuntimeDebug,Perbill};
// use sp_runtime::traits::{OpaqueKeys,Convert};
use sp_runtime::traits::{Convert};
use sp_staking::offence::{OffenceDetails};
use frame_support::{decl_module, decl_storage, decl_event};
// use frame_support::dispatch::{DispatchResult};
use frame_support::traits::{Currency,LockableCurrency};
// use system::{ensure_root,ensure_signed};

type BalanceOf<T> = 
	<<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

pub trait Trait: system::Trait + pallet_balances::Trait + pallet_session::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
	type Currency: LockableCurrency<Self::AccountId, Moment=Self::BlockNumber>;
}

decl_storage! {
	trait Store for Module<T: Trait> as MoonbeamStakingModule {
		SessionIndexStore: u32;
		SessionValidators get(session_validators): Vec<T::AccountId>;
		SessionValidatorAuthoring: map hasher(blake2_256) T::AccountId => u32;
	}
    add_extra_genesis {
		config(session_validators): Vec<T::AccountId>;
        build(|config: &GenesisConfig<T>| {
			let _ = <SessionValidators<T>>::append(config.session_validators.clone());
        });
    }
}

decl_event!(
	pub enum Event<T> 
	where 
		AccountId = <T as system::Trait>::AccountId,
	{
		BlockAuthored(AccountId),
		NewSessionIndex(u32),
		EndSessionIndex(u32),
	}
);

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event() = default;
	}
}

impl<T: Trait> Module<T> {

}

pub struct SessionManager<T>(T);
impl<T: Trait> pallet_session::SessionManager<T::AccountId> for SessionManager<T> {
	fn new_session(new_index: u32) -> Option<Vec<T::AccountId>> {
		SessionIndexStore::put(new_index);
		<Module<T>>::deposit_event(
			RawEvent::NewSessionIndex(new_index)
		);
		if new_index > 1 {
			// TODO rotate, validator queue, etc use SessionHandler trait?
			// TODO call custom economic logic
			let validators = <SessionValidators<T>>::get();
			for v in validators.iter() {
				<SessionValidatorAuthoring<T>>::insert(v.clone(),0);
			}
			Some(validators)
		} else {
			None
		}
	}

	fn end_session(end_index: u32) {
		<Module<T>>::deposit_event(
			RawEvent::EndSessionIndex(end_index)
		);
	}
}

pub struct AuthorshipEventHandler<T>(T);
impl<T: Trait> pallet_authorship::EventHandler<T::AccountId,u32> for AuthorshipEventHandler<T> {
	fn note_author(author: T::AccountId) {
		let authored_blocks = 
			<SessionValidatorAuthoring<T>>::get(&author).checked_add(1).ok_or("Overflow").unwrap();
		<SessionValidatorAuthoring<T>>::insert(&author,authored_blocks);
		// <Module<T>>::deposit_event(
		// 	RawEvent::BlockAuthored(author)
		// );
	}
	fn note_uncle(_author: T::AccountId, _age: u32) {
		
	}
}

// All below are trait implemenations that we need to satisfy for the historical feature of the pallet-session
// Is required by offences and session::historical. Find a way to remove without having to implement.

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