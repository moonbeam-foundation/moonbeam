#![recursion_limit = "256"]
//! Minimal staking module with ordered validator selection and reward curve distribution
#![cfg_attr(not(feature = "std"), no_std)]

mod set;

use frame_support::{
	decl_error, decl_event, decl_module, decl_storage, ensure,
	traits::{Currency, EstimateNextNewSession, Get, ReservableCurrency},
};
use frame_system::{ensure_signed, Config as System};
use pallet_staking::{Exposure, IndividualExposure};
use parity_scale_codec::{Decode, Encode, HasCompact};
use set::OrderedSet;
use sp_runtime::{
	traits::{AtLeast32BitUnsigned, Convert, Zero},
	DispatchResult, Perbill, RuntimeDebug,
};
use sp_std::{cmp::Ordering, prelude::*};
#[cfg(test)]
pub(crate) mod mock;
#[cfg(test)]
mod tests;

#[derive(Default, Clone, Encode, Decode, RuntimeDebug)]
pub struct Bond<AccountId, Balance> {
	pub owner: AccountId,
	pub amount: Balance,
}

impl<AccountId: Ord, Balance> Eq for Bond<AccountId, Balance> {}

impl<AccountId: Ord, Balance> Ord for Bond<AccountId, Balance> {
	fn cmp(&self, other: &Self) -> Ordering {
		self.owner.cmp(&other.owner)
	}
}

impl<AccountId: Ord, Balance> PartialOrd for Bond<AccountId, Balance> {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl<AccountId: Ord, Balance> PartialEq for Bond<AccountId, Balance> {
	fn eq(&self, other: &Self) -> bool {
		self.owner == other.owner
	}
}

impl<A, B: HasCompact> Into<IndividualExposure<A, B>> for Bond<A, B> {
	fn into(self) -> IndividualExposure<A, B> {
		IndividualExposure {
			who: self.owner,
			value: self.amount,
		}
	}
}

#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
/// The activity status of the validator
pub enum ValidatorStatus<BlockNumber> {
	/// Committed to be online and producing valid blocks (not equivocating)
	Active,
	/// Temporarily inactive and excused for inactivity
	Idle,
	/// Bonded until the wrapped block
	Leaving(BlockNumber),
}

impl<B> Default for ValidatorStatus<B> {
	fn default() -> ValidatorStatus<B> {
		ValidatorStatus::Active
	}
}

#[derive(Encode, Decode, RuntimeDebug)]
pub struct CandidateState<AccountId, Balance, RoundIndex> {
	pub validator: AccountId,
	pub fee: Perbill,
	pub nominators: OrderedSet<Bond<AccountId, Balance>>,
	pub total: Balance,
	pub state: ValidatorStatus<RoundIndex>,
}

impl<
		A: Ord + Clone,
		B: AtLeast32BitUnsigned + Ord + Copy + sp_std::ops::AddAssign + sp_std::ops::SubAssign,
		C: Ord + Copy,
	> CandidateState<A, B, C>
{
	pub fn new(validator: A, fee: Perbill, bond: B) -> Self {
		let nominators = OrderedSet::from(vec![Bond {
			owner: validator.clone(),
			amount: bond,
		}]);
		let total = bond;
		CandidateState {
			validator,
			fee,
			nominators,
			total,
			state: ValidatorStatus::default(), // default active
		}
	}
	pub fn is_active(&self) -> bool {
		self.state == ValidatorStatus::Active
	}
	pub fn is_leaving(&self) -> bool {
		if let ValidatorStatus::Leaving(_) = self.state {
			true
		} else {
			false
		}
	}
	pub fn go_offline(&mut self) {
		self.state = ValidatorStatus::Idle;
	}
	pub fn go_online(&mut self) {
		self.state = ValidatorStatus::Active;
	}
	pub fn leave_candidates(&mut self, block: C) {
		self.state = ValidatorStatus::Leaving(block);
	}
}

impl<A: PartialEq, B: HasCompact + Zero, C> Into<Exposure<A, B>> for CandidateState<A, B, C> {
	fn into(self) -> Exposure<A, B> {
		let mut others = Vec::<IndividualExposure<A, B>>::new();
		let mut own = Zero::zero();
		for Bond { owner, amount } in self.nominators.0 {
			if owner == self.validator {
				own = amount;
			} else {
				others.push(Bond { owner, amount }.into());
			}
		}
		Exposure {
			total: self.total,
			own,
			others,
		}
	}
}

type RoundIndex = u32;
type RewardPoint = u32;
type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as System>::AccountId>>::Balance;
type Candidate<T> = CandidateState<<T as System>::AccountId, BalanceOf<T>, RoundIndex>;

pub trait Config: System {
	/// The overarching event type
	type Event: From<Event<Self>> + Into<<Self as System>::Event>;
	/// The currency type
	type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
	/// Interface for interacting with sessions module
	type SessionInterface: SessionInterface<Self::AccountId>;
	/// Something that can estimate the next session change, accurately or as a best effort guess.
	type NextNewSession: EstimateNextNewSession<Self::BlockNumber>;
	/// Blocks per round
	type BlocksPerRound: Get<Self::BlockNumber>;
	/// Number of rounds that candidates remain bonded after requesting exit for retroactive accountability
	type BondDuration: Get<RoundIndex>;
	/// Maximum validators per round
	type MaxValidators: Get<u32>;
	/// Maximum nominators per validator
	type MaxNominatorsPerValidator: Get<usize>;
	/// Maximum fee for any validator
	type MaxFee: Get<Perbill>;
	/// Minimum stake for any registered on-chain account to become a validator
	type MinValidatorStk: Get<BalanceOf<Self>>;
	/// Minimum stake for any registered on-chain account to become a nominator
	type MinNominatorStk: Get<BalanceOf<Self>>;
}

decl_event!(
	pub enum Event<T>
	where
		AccountId = <T as System>::AccountId,
		Balance = BalanceOf<T>,
		BlockNumber = <T as System>::BlockNumber,
	{
		// Starting Block, Round, Number of Validators, Total Balance
		NewRound(BlockNumber, RoundIndex, u32, Balance),
		// Account, Amount Locked, New Total Amt Locked
		JoinedValidatorCandidates(AccountId, Balance, Balance),
		// Round, Validator Account, Total Exposed Amount (includes all nominations)
		ValidatorChosen(RoundIndex, AccountId, Balance),
		ValidatorWentOffline(RoundIndex, AccountId),
		ValidatorBackOnline(RoundIndex, AccountId),
		// Round, Validator Account, Scheduled Exit
		ValidatorScheduledExit(RoundIndex, AccountId, RoundIndex),
		// Account, Amount Unlocked, New Total Amt Locked
		ValidatorLeft(AccountId, Balance, Balance),
		// Nominator, Amount Locked, Validator, New Total Amt Locked
		ValidatorNominated(AccountId, Balance, AccountId, Balance),
	}
);

decl_error! {
	pub enum Error for Module<T: Config> {
		// Nominator Does Not Exist
		NominatorDNE,
		CandidateDNE,
		ValidatorDNE,
		NominatorExists,
		CandidateExists,
		ValidatorExists,
		FeeOverMax,
		ValBondBelowMin,
		NomBondBelowMin,
		AlreadyOffline,
		AlreadyActive,
		AlreadyLeaving,
		TooManyNominators,
		CannotActivateIfLeaving,
	}
}

decl_storage! {
	trait Store for Module<T: Config> as Stake {
		/// Current round, incremented every `BlocksPerRound` in `fn on_finalize`
		pub Round get(fn round): RoundIndex;
		/// Current nominators with their validator
		pub Nominators get(fn nominators): map
			hasher(blake2_128_concat) T::AccountId => Option<T::AccountId>;
		/// Current candidates with associated state (includes validator settings for all validators)
		pub Candidates get(fn candidates): map
			hasher(blake2_128_concat) T::AccountId => Option<Candidate<T>>;
		/// Pool of candidates, ordered by account id
		pub CandidateQueue get(fn candidate_queue): OrderedSet<Bond<T::AccountId,BalanceOf<T>>>;
		/// Queue of validator exit requests, ordered by account id
		pub ExitQueue get(fn exit_queue): OrderedSet<Bond<T::AccountId,RoundIndex>>;
		/// Exposure at stake per round, per validator
		pub AtStake get(fn at_stake): double_map
			hasher(blake2_128_concat) RoundIndex,
			hasher(blake2_128_concat) T::AccountId => Exposure<T::AccountId,BalanceOf<T>>;
		/// Total points awarded in this round
		pub Points get(fn points): map
			hasher(blake2_128_concat) RoundIndex => RewardPoint;
		/// Individual points accrued each round per validator
		pub AwardedPts get(fn awarded_pts): double_map
			hasher(blake2_128_concat) RoundIndex,
			hasher(blake2_128_concat) T::AccountId => RewardPoint;
		/// Total Locked
		pub Total get(fn total): BalanceOf<T>;
	}
	add_extra_genesis {
		config(stakers):
			Vec<(T::AccountId,Option<T::AccountId>,BalanceOf<T>)>;
		build(|config: &GenesisConfig<T>| {
			for &(ref actor, ref opt_val, balance) in &config.stakers {
				assert!(
					T::Currency::free_balance(&actor) >= balance,
					"Stash does not have enough balance to bond."
				);
				let _ = if let Some(nominated_val) = opt_val {
					<Module<T>>::nominate(
						T::Origin::from(Some(actor.clone()).into()),
						nominated_val.clone(),
						balance,
					)
				} else {
					<Module<T>>::join_candidates(
						T::Origin::from(Some(actor.clone()).into()),
						Perbill::from_percent(2),
						balance,
					)
				};
			}
			let (v_count, total_staked) = <Module<T>>::best_candidates_become_validators(1u32);
			// start Round 1 at Block 0
			<Round>::put(1u32);
			<Module<T>>::deposit_event(RawEvent::NewRound(T::BlockNumber::zero(), 1u32, v_count, total_staked));
		});
	}
}

decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {
		type Error = Error<T>;
		fn deposit_event() = default;

		#[weight = 0]
		fn join_candidates(
			origin,
			fee: Perbill,
			bond: BalanceOf<T>,
		) -> DispatchResult {
			let acc = ensure_signed(origin)?;
			ensure!(!Self::is_candidate(&acc),Error::<T>::CandidateExists);
			ensure!(!Self::is_nominator(&acc),Error::<T>::NominatorExists);
			ensure!(fee <= T::MaxFee::get(),Error::<T>::FeeOverMax);
			ensure!(bond >= T::MinValidatorStk::get(),Error::<T>::ValBondBelowMin);
			let mut pool = <CandidateQueue<T>>::get();
			ensure!(pool.insert(Bond{owner: acc.clone(), amount: bond}),Error::<T>::CandidateExists);
			T::Currency::reserve(&acc,bond)?;
			let candidate: Candidate<T> = CandidateState::new(acc.clone(),fee,bond);
			let new_total = <Total<T>>::get() + bond;
			<Total<T>>::put(new_total);
			<Candidates<T>>::insert(&acc,candidate);
			<CandidateQueue<T>>::put(pool);
			Self::deposit_event(RawEvent::JoinedValidatorCandidates(acc,bond,new_total));
			Ok(())
		}
		#[weight = 0]
		fn go_offline(origin) -> DispatchResult {
			let validator = ensure_signed(origin)?;
			let mut state = <Candidates<T>>::get(&validator).ok_or(Error::<T>::CandidateDNE)?;
			ensure!(state.is_active(),Error::<T>::AlreadyOffline);
			state.go_offline();
			let mut candidates = <CandidateQueue<T>>::get();
			candidates.0.retain(|c| &c.owner != &validator);
			<CandidateQueue<T>>::put(candidates);
			<Candidates<T>>::insert(&validator,state);
			Self::deposit_event(RawEvent::ValidatorWentOffline(<Round>::get(),validator));
			Ok(())
		}
		#[weight = 0]
		fn go_online(origin) -> DispatchResult {
			let validator = ensure_signed(origin)?;
			let mut state = <Candidates<T>>::get(&validator).ok_or(Error::<T>::CandidateDNE)?;
			ensure!(!state.is_active(),Error::<T>::AlreadyActive);
			ensure!(!state.is_leaving(),Error::<T>::CannotActivateIfLeaving);
			state.go_online();
			let mut candidates = <CandidateQueue<T>>::get();
			ensure!(
				candidates.insert(Bond{owner:validator.clone(),amount:state.total}),
				Error::<T>::AlreadyActive
			);
			<CandidateQueue<T>>::put(candidates);
			<Candidates<T>>::insert(&validator,state);
			Self::deposit_event(RawEvent::ValidatorBackOnline(<Round>::get(),validator));
			Ok(())
		}
		#[weight = 0]
		fn leave_candidates(origin) -> DispatchResult { //TODO: method to cancel leave request
			let validator = ensure_signed(origin)?;
			let mut state = <Candidates<T>>::get(&validator).ok_or(Error::<T>::CandidateDNE)?;
			ensure!(!state.is_leaving(),Error::<T>::AlreadyLeaving);
			let mut exits = <ExitQueue<T>>::get();
			let now = <Round>::get();
			let when = now + T::BondDuration::get();
			ensure!(
				exits.insert(Bond{owner:validator.clone(),amount:when}),
				Error::<T>::AlreadyLeaving
			);
			state.leave_candidates(when);
			let mut pool = <CandidateQueue<T>>::get();
			pool.0.retain(|c| &c.owner != &validator);
			<CandidateQueue<T>>::put(pool);
			<ExitQueue<T>>::put(exits);
			<Candidates<T>>::insert(&validator,state);
			Self::deposit_event(RawEvent::ValidatorScheduledExit(now,validator,when));
			Ok(())
		}
		#[weight = 0]
		fn nominate(
			origin,
			validator: T::AccountId,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let acc = ensure_signed(origin)?;
			ensure!(!Self::is_nominator(&acc),Error::<T>::NominatorExists);
			ensure!(!Self::is_candidate(&acc),Error::<T>::CandidateExists);
			let mut state = <Candidates<T>>::get(&validator).ok_or(Error::<T>::CandidateDNE)?;
			ensure!(amount >= T::MinNominatorStk::get(), Error::<T>::NomBondBelowMin);
			let nomination = Bond {
				owner: acc.clone(),
				amount,
			};
			ensure!(state.nominators.insert(nomination),Error::<T>::NominatorExists);
			ensure!(state.nominators.0.len() <= T::MaxNominatorsPerValidator::get(),Error::<T>::TooManyNominators);
			T::Currency::reserve(&acc,amount)?;
			let new_total = state.total + amount;
			// update total for candidate if in candidate pool
			let mut candidates = <CandidateQueue<T>>::get();
			// only insert if nominated validator is a viable candidate
			let changed = if candidates.remove(&Bond{owner:validator.clone(),amount:state.total}) || state.is_active() {
				candidates.insert(Bond{owner:validator.clone(),amount:new_total})
			} else {
				false
			};
			if changed {
				<CandidateQueue<T>>::put(candidates);
			}
			let new_total_locked = <Total<T>>::get() + amount;
			<Total<T>>::put(new_total_locked);
			<Nominators<T>>::insert(&acc,validator.clone());
			state.total = new_total;
			<Candidates<T>>::insert(&validator,state);
			Self::deposit_event(RawEvent::ValidatorNominated(acc,amount,validator,new_total));
			Ok(())
		}// TODO: enable switching nominations, going offline, leaving
		fn on_finalize(n: T::BlockNumber) {
			if (n % T::BlocksPerRound::get()).is_zero() {
				let next = <Round>::get() + 1;
				Self::execute_delayed_validator_exits(next);
				// insert exposure for next validator set
				let (validator_count, total_staked) = Self::best_candidates_become_validators(next);
				// start next round
				<Round>::put(next);
				Self::deposit_event(RawEvent::NewRound(n, next, validator_count, total_staked));
			}
		}
	}
}

impl<T: Config> Module<T> {
	pub fn is_nominator(acc: &T::AccountId) -> bool {
		<Nominators<T>>::get(acc).is_some()
	}
	pub fn is_candidate(acc: &T::AccountId) -> bool {
		<Candidates<T>>::get(acc).is_some()
	}
	pub fn is_validator(round: RoundIndex, acc: &T::AccountId) -> bool {
		<AtStake<T>>::get(round, acc) != Exposure::default()
	}
	fn execute_delayed_validator_exits(next: RoundIndex) {
		let mut exits = <ExitQueue<T>>::get().0;
		// order exits by round
		exits.sort_unstable_by(|a, b| a.amount.partial_cmp(&b.amount).unwrap());
		let remain_exits = exits
			.into_iter()
			.filter_map(|x| {
				if x.amount > next {
					Some(x)
				} else {
					if let Some(state) = <Candidates<T>>::get(&x.owner) {
						for bond in state.nominators.0 {
							// return funds to nominator
							T::Currency::unreserve(&bond.owner, bond.amount);
						}
						let new_total = <Total<T>>::get() - state.total;
						<Total<T>>::put(new_total);
						<Candidates<T>>::remove(&x.owner);
						Self::deposit_event(RawEvent::ValidatorLeft(
							x.owner.clone(),
							state.total,
							new_total,
						));
					}
					None
				}
			})
			.collect::<Vec<Bond<T::AccountId, RoundIndex>>>();
		<ExitQueue<T>>::put(OrderedSet::from(remain_exits));
	}
	/// Best as in most cumulatively supported in terms of stake
	fn best_candidates_become_validators(next: RoundIndex) -> (u32, BalanceOf<T>) {
		let (mut all_validators, mut total) = (0u32, BalanceOf::<T>::zero());
		let mut candidates = <CandidateQueue<T>>::get().0;
		// order candidates by stake (least to greatest so requires `rev()`)
		candidates.sort_unstable_by(|a, b| a.amount.partial_cmp(&b.amount).unwrap());
		let max_validators = T::MaxValidators::get() as usize;
		// choose the top MaxValidators qualified candidates, ordered by stake
		let validators = candidates
			.into_iter()
			.rev()
			.take(max_validators)
			.map(|x| x.owner)
			.collect::<Vec<T::AccountId>>();
		// insert canonical validator set
		for account in validators.iter() {
			let state = <Candidates<T>>::get(&account)
				.expect("all members of CandidateQ must be viable candidates by construction; qed");
			let amount = state.total;
			let exposure: Exposure<T::AccountId, BalanceOf<T>> = state.into();
			<AtStake<T>>::insert(next, account, exposure);
			all_validators += 1u32;
			total += amount;
			Self::deposit_event(RawEvent::ValidatorChosen(next, account.clone(), amount));
		}
		(all_validators, total)
	}
}

impl<T: Config> Module<T> {
	fn new_session() -> Option<Vec<T::AccountId>> {
		let mut candidates = <CandidateQueue<T>>::get().0;
		// order candidates by stake (least to greatest so requires `rev()`)
		candidates.sort_unstable_by(|a, b| a.amount.partial_cmp(&b.amount).unwrap());
		let max_validators = T::MaxValidators::get() as usize;
		// choose the top MaxValidators qualified candidates, ordered by stake
		let validators = candidates
			.into_iter()
			.rev()
			.take(max_validators)
			.map(|x| x.owner)
			.collect::<Vec<T::AccountId>>();
		if validators.is_empty() {
			return None;
		}
		Some(validators)
	}
	fn start_session(index: RoundIndex) {
		if index > <Round>::get() {
			<Round>::put(index);
		}
	}
}

/// Add reward points to block authors:
/// * 20 points to the block producer for producing a block in the chain
impl<T> author::EventHandler<T::AccountId> for Module<T>
where
	T: Config + author::Config + pallet_session::Config,
{
	fn note_author(author: T::AccountId) {
		let now = <Round>::get();
		let score_plus_20 = <AwardedPts<T>>::get(now, &author) + 20;
		<AwardedPts<T>>::insert(now, author, score_plus_20);
		<Points>::mutate(now, |x| *x += 20);
	}
}

impl<T: Config> pallet_session::SessionManager<T::AccountId> for Module<T> {
	fn new_session(_new_index: RoundIndex) -> Option<Vec<T::AccountId>> {
		Self::new_session()
	}
	fn start_session(start_index: RoundIndex) {
		Self::start_session(start_index)
	}
	fn end_session(_end_index: RoundIndex) {}
}

impl<T: Config>
	pallet_session::historical::SessionManager<T::AccountId, Exposure<T::AccountId, BalanceOf<T>>>
	for Module<T>
{
	fn new_session(
		new_index: RoundIndex,
	) -> Option<Vec<(T::AccountId, Exposure<T::AccountId, BalanceOf<T>>)>> {
		<Self as pallet_session::SessionManager<_>>::new_session(new_index).map(|validators| {
			let current_era = Self::round();
			validators
				.into_iter()
				.map(|v| {
					let exposure = Self::at_stake(current_era, &v);
					(v, exposure)
				})
				.collect()
		})
	}
	fn start_session(start_index: RoundIndex) {
		<Self as pallet_session::SessionManager<_>>::start_session(start_index)
	}
	fn end_session(end_index: RoundIndex) {
		<Self as pallet_session::SessionManager<_>>::end_session(end_index)
	}
}

/// A typed conversion from stash account ID to the active exposure of nominators
/// on that account.
pub struct ExposureOf<T>(sp_std::marker::PhantomData<T>);

impl<T: Config> Convert<T::AccountId, Option<Exposure<T::AccountId, BalanceOf<T>>>>
	for ExposureOf<T>
{
	fn convert(validator: T::AccountId) -> Option<Exposure<T::AccountId, BalanceOf<T>>> {
		Some(<Module<T>>::at_stake(<Round>::get(), &validator))
	}
}

/// A `Convert` implementation that finds the stash of the given controller account,
/// if any.
pub struct StashOf<T>(sp_std::marker::PhantomData<T>);

impl<T: Config> Convert<T::AccountId, Option<T::AccountId>> for StashOf<T> {
	fn convert(controller: T::AccountId) -> Option<T::AccountId> {
		// our module only has one account per user so we just return the input if it is a validator or nominator
		if <Module<T>>::is_nominator(&controller) || <Module<T>>::is_candidate(&controller) {
			Some(controller)
		} else {
			None
		}
	}
}

/// Means for interacting with a specialized version of the `session` trait.
///
/// This is needed because `Staking` sets the `ValidatorIdOf` of the `pallet_session::Config`
pub trait SessionInterface<AccountId>: frame_system::Config {
	/// Disable a given validator by stash ID.
	///
	/// Returns `true` if new era should be forced at the end of this session.
	/// This allows preventing a situation where there is too many validators
	/// disabled and block production stalls.
	fn disable_validator(validator: &AccountId) -> Result<bool, ()>;
	/// Get the validators from session.
	fn validators() -> Vec<AccountId>;
	/// Prune historical session tries up to but not including the given index.
	fn prune_historical_up_to(up_to: RoundIndex);
}

impl<T: Config> SessionInterface<<T as frame_system::Config>::AccountId> for T
where
	T: pallet_session::Config<ValidatorId = <T as frame_system::Config>::AccountId>,
	T: pallet_session::historical::Config<
		FullIdentification = Exposure<<T as frame_system::Config>::AccountId, BalanceOf<T>>,
		FullIdentificationOf = ExposureOf<T>,
	>,
	T::SessionHandler: pallet_session::SessionHandler<<T as frame_system::Config>::AccountId>,
	T::SessionManager: pallet_session::SessionManager<<T as frame_system::Config>::AccountId>,
	T::ValidatorIdOf: Convert<
		<T as frame_system::Config>::AccountId,
		Option<<T as frame_system::Config>::AccountId>,
	>,
{
	fn disable_validator(validator: &<T as frame_system::Config>::AccountId) -> Result<bool, ()> {
		<pallet_session::Module<T>>::disable(validator)
	}

	fn validators() -> Vec<<T as frame_system::Config>::AccountId> {
		<pallet_session::Module<T>>::validators()
	}

	fn prune_historical_up_to(up_to: RoundIndex) {
		<pallet_session::historical::Module<T>>::prune_up_to(up_to);
	}
}
