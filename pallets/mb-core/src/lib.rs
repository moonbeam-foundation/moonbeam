#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{decl_module, decl_storage, decl_event, dispatch::DispatchResult};
use system::ensure_signed;

pub trait Trait: system::Trait + balances::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
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