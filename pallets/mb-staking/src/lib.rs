

#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;
use codec::{HasCompact, Encode, Decode};
use sp_runtime::{RuntimeDebug,Perbill};
// use sp_runtime::traits::{OpaqueKeys,Convert};
use sp_runtime::traits::{Convert,SaturatedConversion,CheckedSub};
use sp_staking::offence::{OffenceDetails};
use frame_support::{decl_module, decl_storage, decl_event};
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

#[derive(Encode, Decode)]
enum EndorserStatus { Raised, Active, Chill }
impl Default for EndorserStatus {
    fn default() -> Self {
        EndorserStatus::Raised
    }
}

decl_storage! {
	trait Store for Module<T: Trait> as MoonbeamStakingModule {
		EraIndex: u32;
		// Session
		SessionOfEraIndex: u32;
		SessionValidators get(session_validators): Vec<T::AccountId>;
		SessionValidatorAuthoring: map hasher(blake2_256) T::AccountId => u32;
		// Endorsement
		EndorsementStatus: 
			double_map hasher(blake2_256) T::AccountId, hasher(blake2_256) T::AccountId => EndorserStatus;
		EndorsementBalance: 
			double_map hasher(blake2_256) T::AccountId, hasher(blake2_256) T::AccountId => T::Balance; // Endorser, Validator => amount
		Endorsers: map hasher(blake2_256) T::AccountId => Vec<T::AccountId>; // Validator => Endorsers
		// Validator Balance
		ValidatorBalance: map hasher(blake2_256) T::AccountId => T::Balance;
		// Treasury
		Treasury get(treasury): T::Balance;
	}
    add_extra_genesis {
		config(session_validators): Vec<T::AccountId>;
		config(treasury): T::Balance;
        build(|config: &GenesisConfig<T>| {
			// set validators
			let _ = <SessionValidators<T>>::append(config.session_validators.clone());
			// set treasury
			<Treasury<T>>::put(config.treasury);
			// set genesis era
			EraIndex::put(1);
        });
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
		fn deposit_event() = default;

		pub fn endorse(
			origin, to:T::AccountId, value: T::Balance
		) -> DispatchResult {
			let from = ensure_signed(origin)?;
			<EndorsementBalance<T>>::insert(&from,&to,value);
			<Endorsers<T>>::append(&to,vec![from])?;
			Ok(())
		}
	}
}

impl<T: Trait> Module<T> {

	/// Calculates the total reward for the current era using the expected yearly rewards.
	/// (TODO This could probably be cached).
	fn total_payout() -> u64 {
		let era_duration_in_ms: u64 = 
			(EPOCH_DURATION_IN_BLOCKS as u64) * 1000 * (T::SessionsPerEra::get() as u64);
		let year_amount: u128 = REWARD_PER_YEAR;
		let era_coef: f64 = 
			era_duration_in_ms as f64 / MILLISECS_PER_YEAR as f64;
		let total_payout: u64 = (era_coef as f64 * year_amount as f64) as u64;
		total_payout
	}

	/// The payout for each validator is calculated as a weighted average of its produced blocks and endorsement,
	/// and the expected total era reward.
	/// 
	/// For every validator we use:
	/// 	- The number of blocks `b` produced in the era.
	/// 	- The endorsement `d` behind it.
	/// For a total number of blocks B and a total endorsement D we calculate the validator weight W:
	/// 	W = ((b/B) + (d/D)) / 2
	/// For an era payout P, each validator payout `p`:
	/// 	p = W * P
	/// 
	/// TODO:
	/// How about not using iterators? The number of validators and its endorsers are not expected to be big. Also
	/// this code is executed on end of eras, so it should not impact to the chain performance.
	/// 
	/// However, it is desirable to find a storage layout using index helpers to help us remove the iterative approach.
	fn validator_payout() {
		let total_payout = Self::total_payout();
		
		let validators = <SessionValidators<T>>::get();

		// get the total blocks and endorsement
		let mut total_block_count: u32 = 0;
		let mut total_endoresement: u128 = 0;
		for v in &validators {
			total_block_count = total_block_count.checked_add(
				<SessionValidatorAuthoring<T>>::get(&v)
			).ok_or("total_block_count Overflow").unwrap();
			let endorsers = <Endorsers<T>>::get(&v);
			for ed in endorsers {
				total_endoresement = total_endoresement.checked_add(
					<EndorsementBalance<T>>::get(&ed,&v).saturated_into()
				).ok_or("total_endoresement Overflow").unwrap();
			}
		}

		if total_block_count > 0 && total_endoresement > 0 {
			for v in &validators {
				// block fraction
				let block_count: u64 = <SessionValidatorAuthoring<T>>::get(&v) as u64;
				let block_count_coef: f64 = block_count as f64 / total_block_count as f64;
				// endorsement fraction
				let mut endorsement: u128 = 0;
				let endorsers = <Endorsers<T>>::get(&v);
				for ed in &endorsers {
					endorsement = endorsement
						.checked_add(
							<EndorsementBalance<T>>::get(&ed,&v).saturated_into()
						)
						.ok_or("endorsement Overflow").unwrap();
				}
				let endorsement_coef: f64 = endorsement as f64 / total_endoresement as f64;
				// weighted avg
				let coef: f64 = (block_count_coef + endorsement_coef) / (2 as f64);
				let payout: u32 = (total_payout as f64 * coef) as u32;
				// desposit from treasury
				let new_balance = <ValidatorBalance<T>>::get(&v) + T::Balance::from(payout);
				let new_treasury = <Treasury<T>>::get().checked_sub(&T::Balance::from(payout))
					.ok_or("new_treasury Overflow").unwrap();

				<ValidatorBalance<T>>::insert(&v,new_balance);
				<Treasury<T>>::put(new_treasury);
			}
		}
	}

	fn reset_validator_stats() {
		let validators = <SessionValidators<T>>::get();
		for v in &validators {
			<SessionValidatorAuthoring<T>>::insert(v.clone(),0);
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

				let new_era_idx = EraIndex::get().checked_add(1)
					.ok_or("SessionOfEraIndex Overflow").unwrap();
					
				EraIndex::put(new_era_idx);

				<Module<T>>::validator_payout();
				<Module<T>>::reset_validator_stats();
					
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