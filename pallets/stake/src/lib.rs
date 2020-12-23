#![recursion_limit = "256"]
//! Minimal Staking Pallet
#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	decl_error,
	decl_event,
	decl_module,
	decl_storage,
	ensure,
	//storage::IterableStorageMap,
	traits::{
		Currency,
		//ExistenceRequirement::KeepAlive, //needed for transfer
		Get,
		ReservableCurrency,
	},
};
use frame_system::{ensure_signed, Config as System};
use parity_scale_codec::{Decode, Encode};
use sp_runtime::{
	traits::{AccountIdConversion, AtLeast32BitUnsigned, Zero},
	DispatchResult, ModuleId, Perbill, RuntimeDebug,
};
use sp_std::prelude::*;

#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
// TODO: make for adding Staked variant, compounding rewards is an additional feature, not mvp
pub enum Destination<AccountId> {
	/// Pay into the stash account, not increasing the amount at stake.
	Stash,
	/// Pay into a specified account.
	Account(AccountId),
}

impl<AccountId> Default for Destination<AccountId> {
	fn default() -> Self {
		Destination::Stash
	}
}

#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
pub enum Reward<Dest> {
	/// Pay into single RewardDestination
	One(Dest),
	/// Pay (Perbill * Total) to 1st Dest, ((1-Perbill) * Total) to 2nd Dest
	Two(Dest, Perbill, Dest),
}

impl<Dest: Default> Default for Reward<Dest> {
	fn default() -> Reward<Dest> {
		Reward::One(Dest::default())
	}
}

#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
/// Validator preferences
pub struct ValPrefs<Balance> {
	/// The fee this validator takes on all profits before returning rewards in proportion to stake weight (which may also include the validator)
	pub fee: Perbill,
	/// Minimum nomination amount accepted by this validator
	pub min: Balance,
}

#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
/// Candidate information
pub struct CandidateInfo<Balance> {
	/// The candidate's expected stake amount
	pub stake: Balance,
	/// Minimum nomination amount accepted by this validator
	pub prefs: ValPrefs<Balance>,
}

#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
pub struct Nomination<AccountId, Balance> {
	pub owner: AccountId,
	pub amount: Balance,
}

impl<A, B> Nomination<A, B> {
	pub fn new(owner: A, amount: B) -> Nomination<A, B> {
		Nomination { owner, amount }
	}
}

#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
pub enum ValStatus {
	Active,
	Chill,
}

#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug)]
pub struct ValState<AccountId, Balance> {
	pub prefs: ValPrefs<Balance>,
	pub nominations: Vec<Nomination<AccountId, Balance>>,
	pub total: Balance,
	pub status: ValStatus,
}

impl<
		A: Ord + Clone,
		B: AtLeast32BitUnsigned + Ord + Copy + sp_std::ops::AddAssign + sp_std::ops::SubAssign,
	> ValState<A, B>
{
	pub fn new(validator: A, prefs: ValPrefs<B>, bond: B) -> ValState<A, B> {
		ValState {
			prefs,
			nominations: vec![Nomination::new(validator, bond)],
			total: bond,
			status: ValStatus::Active,
		}
	}
	pub fn chill(&mut self) {
		self.status = ValStatus::Chill;
	}
	pub fn activate(&mut self) {
		self.status = ValStatus::Active;
	}
	pub fn is_active(&self) -> bool {
		self.status == ValStatus::Active
	}
	/// Adds new nomination (assumes nomination does not already exist for nominator A)
	pub fn add_nomination(&mut self, nominator: A, amount: B) {
		self.nominations.push(Nomination::new(nominator, amount));
		self.total += amount;
	}
	/// Remove the entire nomination and, if removal successful, return unstaked amount
	pub fn rm_nomination(&mut self, nominator: A) -> Option<B> {
		let mut ret: Option<B> = None;
		let nominations = self
			.nominations
			.clone()
			.into_iter()
			.filter_map(|x| {
				if x.owner == nominator {
					ret = Some(x.amount);
					Some(x)
				} else {
					None
				}
			})
			.collect();
		self.nominations = nominations;
		ret
	}
}

type RoundIndex = u32;
type RewardPoint = u32;
type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as System>::AccountId>>::Balance;
type ValidatorState<T> = ValState<<T as System>::AccountId, BalanceOf<T>>;
type RewardPolicy<T> = Reward<Destination<<T as System>::AccountId>>;

pub trait Config: System {
	/// The overarching event type
	type Event: From<Event<Self>> + Into<<Self as System>::Event>;
	/// The currency type
	type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
	/// Maximum number of validators for any given round
	type MaxValidators: Get<usize>;
	/// Maximum individual nominators for all validators
	type MaxNomPerVal: Get<usize>;
	/// Minimum individual nominators for all validators
	type MinNomPerVal: Get<usize>;
	/// Minimum stake for any registered on-chain account to become a validator
	type MinStakeBond: Get<BalanceOf<Self>>;
	/// Minimum stake for any registered on-chain account to become a nominator
	type MinNomBond: Get<BalanceOf<Self>>;
	/// Maximum fee a validator can charge (taken off the top of revenue, before stake-weighted payouts)
	type MaxValFee: Get<Perbill>;
	/// Timer for triggering periodic tasks in `on_finalize`
	type BlocksPerRound: Get<Self::BlockNumber>;
	/// Number of rounds kept in-memory for retroactive rewards/penalties
	type HistoryDepth: Get<usize>;
	/// Maximum reward (per Round)
	type Reward: Get<BalanceOf<Self>>;
	/// The treasury's module id, used for deriving its sovereign account ID.
	type Treasury: Get<ModuleId>;
}

decl_event!(
	pub enum Event<T>
	where
		AccountId = <T as System>::AccountId,
		Balance = BalanceOf<T>,
		BlockNumber = <T as System>::BlockNumber,
	{
		// Account, Amount Locked
		CandidateJoined(AccountId, Balance),
		//ValidatorRegistered(RoundIndex,AccountId,Balance),
		ValidatorChilled(RoundIndex, AccountId),
		ValidatorActivated(RoundIndex, AccountId),
		// Account, Amount Unlocked, New Total Amt Locked
		ValidatorLeft(AccountId, Balance, Balance),
		// Nominator, Amount Locked, Validator, New Total Amt Locked
		ValidatorNominated(AccountId, Balance, AccountId, Balance),
		NominationRevoked(AccountId, Balance, AccountId, Balance),
		Rewarded(AccountId, Balance),
		NewRound(BlockNumber, RoundIndex),
	}
);

decl_error! {
	pub enum Error for Module<T: Config> {
		// Nominator Does Not Exist
		NominatorDNE,
		CandidateDNE,
		ValidatorDNE,
		NominatorExists,
		ValidatorExists,
		StakeBondBelowMin,
		StakeNomReqBelowMin,
		NomBondBelowMin,
		FeeExceedsMaxValFee,
		CannotRmNomNotFound,
		TooManyNomForVal,
		AlreadyActive,
		AlreadyChill,
		NoPointsNoReward,
		CurrentRndRewardsUnClaimable,
	}
}

decl_storage! {
	trait Store for Module<T: Config> as Stake {
		/// Current round, incremented every `BlocksPerRound` in `fn on_finalize`
		pub Round get(fn round): RoundIndex;
		/// Nominators with their validator
		pub Nominators get(fn nominators): map
			hasher(blake2_128_concat) T::AccountId => Option<T::AccountId>;
		/// Validator candidates with nomination state (includes all current validators by default)
		pub Candidates get(fn candidates): map
			hasher(blake2_128_concat) T::AccountId => Option<ValidatorState<T>>;
		/// Total locked capital for a given round
		pub Total get(fn total): map
			hasher(blake2_128_concat) RoundIndex => BalanceOf<T>;
		/// Total points awarded in this round
		pub Points get(fn points): map
			hasher(blake2_128_concat) RoundIndex => RewardPoint;
		/// Validator set for the given round, stores individual points accrued for round per validator
		pub Validators get(fn validators): double_map
			hasher(blake2_128_concat) RoundIndex,
			hasher(blake2_128_concat) T::AccountId => RewardPoint;
		/// Track recipient preferences for receiving rewards
		pub Payee get(fn payee): map
			hasher(blake2_128_concat) T::AccountId => Option<RewardPolicy<T>>;
	}
}

decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {
		type Error = Error<T>;
		fn deposit_event() = default;

		#[weight = 0]
		fn join_candidates(
			origin,
			stake: BalanceOf<T>,
			fee: Perbill,
			min: BalanceOf<T>,
			policy: RewardPolicy<T>,
		) -> DispatchResult {
			let acc = ensure_signed(origin)?;
			ensure!(!Self::is_nominator(&acc),Error::<T>::NominatorExists);
			ensure!(!Self::is_candidate(&acc),Error::<T>::ValidatorExists);
			ensure!(stake >= T::MinStakeBond::get(),Error::<T>::StakeBondBelowMin);
			ensure!(min >= T::MinNomBond::get(),Error::<T>::StakeNomReqBelowMin);
			ensure!(fee <= T::MaxValFee::get(),Error::<T>::FeeExceedsMaxValFee);
			T::Currency::reserve(&acc,stake)?;
			let state: ValidatorState<T> = ValState::new(acc.clone(),ValPrefs{fee,min},stake);
			<Candidates<T>>::insert(&acc,state);
			<Payee<T>>::insert(&acc,policy);
			Self::deposit_event(RawEvent::CandidateJoined(acc,stake));
			Ok(())
		}
		#[weight = 0]
		fn chill(origin) -> DispatchResult {
			let acc = ensure_signed(origin)?;
			let mut validator = <Candidates<T>>::get(&acc).ok_or(Error::<T>::CandidateDNE)?;
			ensure!(validator.is_active(),Error::<T>::AlreadyChill);
			validator.chill();
			<Candidates<T>>::insert(&acc,validator);
			Self::deposit_event(RawEvent::ValidatorChilled(<Round>::get(),acc));
			Ok(())
		}
		#[weight = 0]
		fn activate(origin) -> DispatchResult {
			let acc = ensure_signed(origin)?;
			let mut validator = <Candidates<T>>::get(&acc).ok_or(Error::<T>::CandidateDNE)?;
			ensure!(!validator.is_active(),Error::<T>::AlreadyActive);
			validator.activate();
			<Candidates<T>>::insert(&acc,validator);
			Self::deposit_event(RawEvent::ValidatorActivated(<Round>::get(),acc));
			Ok(())
		}
		#[weight = 0]
		fn exit(origin) -> DispatchResult { Self::return_nominations(ensure_signed(origin)?) }
		#[weight = 0]
		fn nominate(
			origin,
			validator: T::AccountId,
			amount: BalanceOf<T>,
			payee: RewardPolicy<T>,
		) -> DispatchResult {
			let acc = ensure_signed(origin)?;
			ensure!(!Self::is_candidate(&acc),Error::<T>::ValidatorExists);
			ensure!(!Self::is_nominator(&acc),Error::<T>::NominatorExists);
			let mut state = <Candidates<T>>::get(&validator).ok_or(Error::<T>::CandidateDNE)?;
			ensure!(amount >= state.prefs.min,Error::<T>::NomBondBelowMin);
			state.add_nomination(acc.clone(),amount);
			ensure!(state.nominations.len() <= T::MaxNomPerVal::get(), Error::<T>::TooManyNomForVal);
			T::Currency::reserve(&acc,amount)?;
			let new_total = state.total;
			<Candidates<T>>::insert(&validator,state);
			<Payee<T>>::insert(&acc,payee);
			Self::deposit_event(RawEvent::ValidatorNominated(acc,amount,validator,new_total));
			Ok(())
		}
		#[weight = 0]
		fn revoke_nomination(origin) -> DispatchResult {
			let acc = ensure_signed(origin)?;
			let validator = <Nominators<T>>::get(&acc).ok_or(Error::<T>::NominatorDNE)?;
			let mut state = <Candidates<T>>::get(&validator).ok_or(Error::<T>::CandidateDNE)?;
			let amt_unstaked = state.rm_nomination(acc.clone()).ok_or(Error::<T>::CannotRmNomNotFound)?;
			let now = <Round>::get();
			let new_total = <Total<T>>::get(now) - amt_unstaked;
			<Total<T>>::insert(now,new_total);
			<Candidates<T>>::insert(&validator,state);
			<Nominators<T>>::remove(&acc);
			Self::deposit_event(RawEvent::NominationRevoked(acc,amt_unstaked,validator,new_total));
			Ok(())
		}
		#[weight = 0]
		fn pay_validator_and_nominators(
			origin,
			validator: T::AccountId,
			round: RoundIndex,
		) -> DispatchResult {
			ensure_signed(origin)?;
			ensure!(<Round>::get() > round,Error::<T>::CurrentRndRewardsUnClaimable);
			let points = <Validators<T>>::get(round,&validator);
			ensure!(points > Zero::zero(), Error::<T>::NoPointsNoReward);
			let all_pts = <Points>::get(round);
			let pct = Perbill::from_rational_approximation(points,all_pts);
			let all = pct * T::Reward::get();
			let val = <Candidates<T>>::get(&validator).ok_or(Error::<T>::CandidateDNE)?;
			let fee = val.prefs.fee * all;
			Self::make_payout(&validator,fee);
			let remaining = all - fee;
			for Nomination{owner,amount} in val.nominations {
				let percent = Perbill::from_rational_approximation(amount,val.total);
				let for_nom = percent * remaining;
				Self::make_payout(&owner,for_nom);
			}
			let remaining_pts = all_pts - points;
			<Points>::insert(round,remaining_pts);
			<Validators<T>>::remove(round,&validator);
			Ok(())
		}
		fn on_finalize(n: T::BlockNumber) {
			if n % T::BlocksPerRound::get() == T::BlockNumber::zero() {
				let next = <Round>::get() + 1;
				<Round>::put(next);
				// TODO: choose validators from candidates, using IterableStorageAPI
			}
		}
	}
}

impl<T: Config> Module<T> {
	pub fn treasury() -> T::AccountId {
		T::Treasury::get().into_account()
	}
	pub fn is_nominator(acc: &T::AccountId) -> bool {
		<Nominators<T>>::get(acc).is_some()
	}
	pub fn is_candidate(acc: &T::AccountId) -> bool {
		<Candidates<T>>::get(acc).is_some()
	}
	pub fn is_validator(round: RoundIndex, acc: &T::AccountId) -> bool {
		!<Validators<T>>::get(round, acc).is_zero()
	}
	pub fn return_nominations(validator: T::AccountId) -> DispatchResult {
		let state = <Candidates<T>>::get(&validator).ok_or(Error::<T>::ValidatorDNE)?;
		for Nomination { owner, amount } in state.nominations {
			// return stake
			let _ = T::Currency::unreserve(&owner, amount);
		}
		let now = <Round>::get();
		let new_total = <Total<T>>::get(now) - state.total;
		<Total<T>>::insert(now, new_total);
		<Candidates<T>>::remove(&validator);
		Self::deposit_event(RawEvent::ValidatorLeft(validator, state.total, new_total));
		Ok(())
	}
	pub fn make_payout(_acc: &T::AccountId, _amt: BalanceOf<T>) -> Option<()> {
		todo!() // use my PR open in substrate
	}
}
