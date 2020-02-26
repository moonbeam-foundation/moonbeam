#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;

use frame_support::{decl_module, decl_storage, decl_event, dispatch::DispatchResult};
use frame_support::traits::{Contains,OnUnbalanced,Currency,LockableCurrency};
use system::{ensure_signed,RawOrigin};
use sp_runtime::{traits::EnsureOrigin};

type NegativeImbalanceOf<T> =
	<<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::NegativeImbalance;

pub trait Trait: system::Trait + balances::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
	type Currency: LockableCurrency<Self::AccountId, Moment=Self::BlockNumber>;
}

decl_storage! {
	trait Store for Module<T: Trait> as TemplateModule {
		AccountBalance: map hasher(blake2_256) T::AccountId => T::Balance;
	}
}

decl_event!(
	pub enum Event<T> 
	where 
		AccountId = <T as system::Trait>::AccountId,
		Balance = <T as balances::Trait>::Balance
	{
		AccountBalanceStored(AccountId,Balance),
	}
);

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {

		fn deposit_event() = default;

		pub fn set_account_balance(origin, value: T::Balance) -> DispatchResult {
			let who = ensure_signed(origin)?;
			<AccountBalance<T>>::insert(&who,value);
			Self::deposit_event(RawEvent::AccountBalanceStored(who,value));
			Ok(())
		}
	}
}

/// We want to use pallet_staking without using pallet_collective by now. 
/// All democracy decisions implement the EnsureOrigin trait and return a Result based on a majority of council votes.
/// By default we want to always allow:
///     - pallet_staking::SlashCancelOrigin
///     - pallet_treasury::ApproveOrigin
///     - pallet_treasury::RejectOrigin
///     - pallet_identity::ForceOrigin
///     - pallet_identity::RegistrarOrigin

pub struct Collective<AccountId>(AccountId);
impl<
    O: Into<Result<RawOrigin<AccountId>, O>> + From<RawOrigin<AccountId>>, 
    AccountId
> EnsureOrigin<O> for Collective<AccountId> {
	type Success = ();
	fn try_origin(_o: O) -> Result<Self::Success, O> {
		Ok(())
	}
}

impl<T: Trait> Contains<T::AccountId> for Module<T> {
	fn contains(_who: &T::AccountId) -> bool {
		true
	}
	fn sorted_members() -> Vec<T::AccountId> { vec![] }
}

impl<T: Trait> OnUnbalanced<NegativeImbalanceOf<T>> for Module<T>
{
	fn on_nonzero_unbalanced(_amount: NegativeImbalanceOf<T>) {
		let _a = 1;
	}
}