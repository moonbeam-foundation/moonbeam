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

//! TODO: add stake docs

#![cfg_attr(not(feature = "std"), no_std)]

mod inflation;
mod set;
use frame_support::pallet;
pub use inflation::{InflationInfo, Range};

pub use pallet::*;

#[pallet]
pub mod pallet {
	use super::{InflationInfo, Range};
	use crate::set::OrderedSet;
	use frame_support::pallet_prelude::*;
	use frame_support::traits::{Currency, Get, ReservableCurrency};
	use frame_system::pallet_prelude::*;
	use parity_scale_codec::{Decode, Encode};
	use sp_runtime::{
		traits::{AtLeast32BitUnsigned, Zero},
		DispatchResult, Perbill, RuntimeDebug,
	};
	use sp_std::{cmp::Ordering, prelude::*};

	/// Pallet for parachain staking
	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	#[derive(Default, Clone, Encode, Decode, RuntimeDebug)]
	pub struct Bond<AccountId, Balance> {
		pub owner: AccountId,
		pub amount: Balance,
	}

	impl<A, B: Default> Bond<A, B> {
		fn from_owner(owner: A) -> Self {
			Bond {
				owner,
				amount: B::default(),
			}
		}
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

	#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
	/// The activity status of the validator
	pub enum ValidatorStatus {
		/// Committed to be online and producing valid blocks (not equivocating)
		Active,
		/// Temporarily inactive and excused for inactivity
		Idle,
		/// Bonded until the inner round
		Leaving(RoundIndex),
	}

	impl Default for ValidatorStatus {
		fn default() -> ValidatorStatus {
			ValidatorStatus::Active
		}
	}

	#[derive(Default, Encode, Decode, RuntimeDebug)]
	/// Snapshot of validator state at the start of the round for which they are selected
	pub struct ValidatorSnapshot<AccountId, Balance> {
		pub fee: Perbill,
		pub bond: Balance,
		pub nominators: Vec<Bond<AccountId, Balance>>,
		pub total: Balance,
	}

	#[derive(Encode, Decode, RuntimeDebug)]
	/// Global validator state with commission fee, bonded stake, and nominations
	pub struct Validator<AccountId, Balance> {
		pub id: AccountId,
		pub fee: Perbill,
		pub bond: Balance,
		pub nominators: OrderedSet<Bond<AccountId, Balance>>,
		pub total: Balance,
		pub state: ValidatorStatus,
	}

	impl<
			A: Ord + Clone,
			B: AtLeast32BitUnsigned + Ord + Copy + sp_std::ops::AddAssign + sp_std::ops::SubAssign,
		> Validator<A, B>
	{
		pub fn new(id: A, fee: Perbill, bond: B) -> Self {
			let total = bond;
			Validator {
				id,
				fee,
				bond,
				nominators: OrderedSet::new(),
				total,
				state: ValidatorStatus::default(), // default active
			}
		}
		pub fn is_active(&self) -> bool {
			self.state == ValidatorStatus::Active
		}
		pub fn is_leaving(&self) -> bool {
			matches!(self.state, ValidatorStatus::Leaving(_))
		}
		pub fn bond_more(&mut self, more: B) {
			self.bond += more;
			self.total += more;
		}
		// Returns None if underflow or less == self.bond (in which case validator should leave instead)
		pub fn bond_less(&mut self, less: B) -> Option<B> {
			if self.bond > less {
				self.bond -= less;
				self.total -= less;
				Some(self.bond)
			} else {
				None
			}
		}
		// infallible so nominator must exist before calling
		pub fn rm_nominator(&mut self, nominator: A) -> B {
			let mut total = self.total;
			let nominators = self
				.nominators
				.0
				.iter()
				.filter_map(|x| {
					if x.owner == nominator {
						total -= x.amount;
						None
					} else {
						Some(x.clone())
					}
				})
				.collect();
			self.nominators = OrderedSet::from(nominators);
			self.total = total;
			total
		}
		// infallible so nominator dne before calling
		pub fn add_nominator(&mut self, owner: A, amount: B) -> B {
			self.nominators.insert(Bond { owner, amount });
			self.total += amount;
			self.total
		}
		// only call with an amount larger than existing amount
		pub fn update_nominator(&mut self, nominator: A, amount: B) -> B {
			let mut difference: B = 0u32.into();
			let nominators = self
				.nominators
				.0
				.iter()
				.map(|x| {
					if x.owner == nominator {
						// new amount must be greater or will underflow
						difference = amount - x.amount;
						Bond {
							owner: x.owner.clone(),
							amount,
						}
					} else {
						x.clone()
					}
				})
				.collect();
			self.nominators = OrderedSet::from(nominators);
			self.total += difference;
			self.total
		}
		pub fn inc_nominator(&mut self, nominator: A, more: B) {
			for x in &mut self.nominators.0 {
				if x.owner == nominator {
					x.amount += more;
					self.total += more;
					return;
				}
			}
		}
		pub fn dec_nominator(&mut self, nominator: A, less: B) {
			for x in &mut self.nominators.0 {
				if x.owner == nominator {
					x.amount -= less;
					self.total -= less;
					return;
				}
			}
		}
		pub fn go_offline(&mut self) {
			self.state = ValidatorStatus::Idle;
		}
		pub fn go_online(&mut self) {
			self.state = ValidatorStatus::Active;
		}
		pub fn leave_candidates(&mut self, round: RoundIndex) {
			self.state = ValidatorStatus::Leaving(round);
		}
	}

	impl<A: Clone, B: Copy> From<Validator<A, B>> for ValidatorSnapshot<A, B> {
		fn from(other: Validator<A, B>) -> ValidatorSnapshot<A, B> {
			ValidatorSnapshot {
				fee: other.fee,
				bond: other.bond,
				nominators: other.nominators.0,
				total: other.total,
			}
		}
	}

	#[derive(Encode, Decode, RuntimeDebug)]
	pub struct Nominator<AccountId, Balance> {
		pub nominations: OrderedSet<Bond<AccountId, Balance>>,
		pub total: Balance,
	}

	impl<
			AccountId: Ord + Clone,
			Balance: Copy
				+ sp_std::ops::AddAssign
				+ sp_std::ops::Add<Output = Balance>
				+ sp_std::ops::SubAssign
				+ PartialOrd,
		> Nominator<AccountId, Balance>
	{
		pub fn new(validator: AccountId, nomination: Balance) -> Self {
			Nominator {
				nominations: OrderedSet::from(vec![Bond {
					owner: validator,
					amount: nomination,
				}]),
				total: nomination,
			}
		}
		pub fn add_nomination(&mut self, bond: Bond<AccountId, Balance>) -> bool {
			let amt = bond.amount;
			if self.nominations.insert(bond) {
				self.total += amt;
				true
			} else {
				false
			}
		}
		// Returns Some(remaining balance), must be more than MinNominatorStk
		// Returns None if nomination not found
		pub fn rm_nomination(&mut self, validator: AccountId) -> Option<Balance> {
			let mut amt: Option<Balance> = None;
			let nominations = self
				.nominations
				.0
				.iter()
				.filter_map(|x| {
					if x.owner == validator {
						amt = Some(x.amount);
						None
					} else {
						Some(x.clone())
					}
				})
				.collect();
			if let Some(balance) = amt {
				self.nominations = OrderedSet::from(nominations);
				self.total -= balance;
				Some(self.total)
			} else {
				None
			}
		}
		// Returns Some(new balances) if old was nominated and None if it wasn't nominated
		pub fn swap_nomination(
			&mut self,
			old: AccountId,
			new: AccountId,
		) -> Option<(Balance, Balance)> {
			let mut amt: Option<Balance> = None;
			let nominations = self
				.nominations
				.0
				.iter()
				.filter_map(|x| {
					if x.owner == old {
						amt = Some(x.amount);
						None
					} else {
						Some(x.clone())
					}
				})
				.collect();
			if let Some(swapped_amt) = amt {
				let mut old_new_amt: Option<Balance> = None;
				let nominations2 = self
					.nominations
					.0
					.iter()
					.filter_map(|x| {
						if x.owner == new {
							old_new_amt = Some(x.amount);
							None
						} else {
							Some(x.clone())
						}
					})
					.collect();
				let new_amount = if let Some(old_amt) = old_new_amt {
					// update existing nomination
					self.nominations = OrderedSet::from(nominations2);
					let new_amt = old_amt + swapped_amt;
					self.nominations.insert(Bond {
						owner: new,
						amount: new_amt,
					});
					new_amt
				} else {
					// insert completely new nomination
					self.nominations = OrderedSet::from(nominations);
					self.nominations.insert(Bond {
						owner: new,
						amount: swapped_amt,
					});
					swapped_amt
				};
				Some((swapped_amt, new_amount))
			} else {
				None
			}
		}
		// Returns None if nomination not found
		pub fn inc_nomination(&mut self, validator: AccountId, more: Balance) -> Option<Balance> {
			for x in &mut self.nominations.0 {
				if x.owner == validator {
					x.amount += more;
					self.total += more;
					return Some(x.amount);
				}
			}
			None
		}
		// Returns Some(Some(balance)) if successful
		// None if nomination not found
		// Some(None) if underflow
		pub fn dec_nomination(
			&mut self,
			validator: AccountId,
			less: Balance,
		) -> Option<Option<Balance>> {
			for x in &mut self.nominations.0 {
				if x.owner == validator {
					if x.amount > less {
						x.amount -= less;
						self.total -= less;
						return Some(Some(x.amount));
					} else {
						// underflow error; should rm entire nomination if x.amount == validator
						return Some(None);
					}
				}
			}
			None
		}
	}

	type RoundIndex = u32;
	type RewardPoint = u32;
	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
	//type Candidate<T> = Validator<<T as frame_system::Config>::T::AccountId, BalanceOf<T>>;

	/// Configuration trait of this pallet.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Overarching event type
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// The currency type
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
		/// Number of blocks per round
		type BlocksPerRound: Get<u32>;
		/// Number of rounds that validators remain bonded before exit request is executed
		type BondDuration: Get<RoundIndex>;
		/// Maximum validators per round
		type MaxValidators: Get<u32>;
		/// Maximum nominators per validator
		type MaxNominatorsPerValidator: Get<u32>;
		/// Maximum validators per nominator
		type MaxValidatorsPerNominator: Get<u32>;
		/// Maximum fee for any validator
		type MaxFee: Get<Perbill>;
		/// Minimum stake for any registered on-chain account to become a validator
		type MinValidatorStk: Get<BalanceOf<Self>>;
		/// Minimum stake for any registered on-chain account to nominate
		type MinNomination: Get<BalanceOf<Self>>;
		/// Minimum stake for any registered on-chain account to become a nominator
		type MinNominatorStk: Get<BalanceOf<Self>>;
	}

	#[pallet::error]
	pub enum Error<T> {
		// Nominator Does Not Exist
		NominatorDNE,
		CandidateDNE,
		NominatorExists,
		CandidateExists,
		FeeOverMax,
		ValBondBelowMin,
		NomBondBelowMin,
		NominationBelowMin,
		AlreadyOffline,
		AlreadyActive,
		AlreadyLeaving,
		TooManyNominators,
		CannotActivateIfLeaving,
		ExceedMaxValidatorsPerNom,
		AlreadyNominatedValidator,
		NominationDNE,
		Underflow,
		CannotSwitchToSameNomination,
		InvalidSchedule,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Starting Block, Round, Number of Validators, Total Balance
		NewRound(T::BlockNumber, RoundIndex, u32, BalanceOf<T>),
		/// Account, Amount Locked, New Total Amt Locked
		JoinedValidatorCandidates(T::AccountId, BalanceOf<T>, BalanceOf<T>),
		/// Round, Validator Account, Total Exposed Amount (includes all nominations)
		ValidatorChosen(RoundIndex, T::AccountId, BalanceOf<T>),
		/// Validator Account, Old Bond, New Bond
		ValidatorBondedMore(T::AccountId, BalanceOf<T>, BalanceOf<T>),
		/// Validator Account, Old Bond, New Bond
		ValidatorBondedLess(T::AccountId, BalanceOf<T>, BalanceOf<T>),
		ValidatorWentOffline(RoundIndex, T::AccountId),
		ValidatorBackOnline(RoundIndex, T::AccountId),
		/// Round, Validator Account, Scheduled Exit
		ValidatorScheduledExit(RoundIndex, T::AccountId, RoundIndex),
		/// Account, Amount Unlocked, New Total Amt Locked
		ValidatorLeft(T::AccountId, BalanceOf<T>, BalanceOf<T>),
		// Nominator, Validator, Old Nomination, New Nomination
		NominationIncreased(T::AccountId, T::AccountId, BalanceOf<T>, BalanceOf<T>),
		// Nominator, Validator, Old Nomination, New Nomination
		NominationDecreased(T::AccountId, T::AccountId, BalanceOf<T>, BalanceOf<T>),
		// Nominator, Swapped Amount, Old Nominator, New Nominator
		NominationSwapped(T::AccountId, BalanceOf<T>, T::AccountId, T::AccountId),
		/// Nominator, Amount Staked
		NominatorJoined(T::AccountId, BalanceOf<T>),
		/// Nominator, Amount Unstaked
		NominatorLeft(T::AccountId, BalanceOf<T>),
		/// Nominator, Amount Locked, Validator, New Total Amt Locked
		ValidatorNominated(T::AccountId, BalanceOf<T>, T::AccountId, BalanceOf<T>),
		/// Nominator, Validator, Amount Unstaked, New Total Amt Staked for Validator
		NominatorLeftValidator(T::AccountId, T::AccountId, BalanceOf<T>, BalanceOf<T>),
		/// Paid the account (nominator or validator) the balance as liquid rewards
		Rewarded(T::AccountId, BalanceOf<T>),
		/// Round inflation range set with the provided annual inflation range
		RoundInflationSet(Perbill, Perbill, Perbill),
		/// Staking expectations set
		StakeExpectationsSet(BalanceOf<T>, BalanceOf<T>, BalanceOf<T>),
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::storage]
	#[pallet::getter(fn round)]
	pub type Round<T: Config> = StorageValue<_, RoundIndex, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn nominators)]
	pub type Nominators<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, u8, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn contract_address)]
	pub type ContractAddress<T: Config> = StorageMap<_, Twox64Concat, u8, u8, OptionQuery>;

	// #[pallet::genesis_config]
	// pub struct GenesisConfig {
	// 	pub nonce: U256,
	// }

	// #[cfg(feature = "std")]
	// impl Default for GenesisConfig {
	// 	fn default() -> Self {
	// 		Self {
	// 			nonce: U256::zero(),
	// 		}
	// 	}
	// }

	// #[pallet::genesis_build]
	// impl<T: Config> GenesisBuild<T> for GenesisConfig {
	// 	fn build(&self) {
	// 		<Nonce<T>>::put(self.nonce);
	// 	}
	// }

	#[pallet::call]
	impl<T: Config> Pallet<T> {}
}
