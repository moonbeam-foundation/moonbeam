use frame_support::dispatch::DispatchInfo;
pub use frame_support::traits::Get;
use frame_system::Config;
pub use moonbeam_core_primitives::AccountId;
use parity_scale_codec::{Decode, Encode};
use polkadot_parachain::primitives::Sibling;
use scale_info::TypeInfo;
use sp_core::TypeId;
use sp_runtime::{
	traits::{DispatchInfoOf, Dispatchable, SignedExtension},
	transaction_validity::{
		InvalidTransaction, TransactionValidity, TransactionValidityError, ValidTransaction,
	},
};
use sp_std::{marker::PhantomData, prelude::*};
use xcm_primitives::ParentSovereign;

/// Check to ensure that the sender is not a sovereign account.
#[derive(Encode, Decode, Clone, Eq, PartialEq, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct CheckNotSovereignSender<T: Config + Send + Sync>(PhantomData<T>);

impl<T: Config + Send + Sync> sp_std::fmt::Debug for CheckNotSovereignSender<T> {
	#[cfg(feature = "std")]
	fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		write!(f, "CheckNotSovereignSender")
	}

	#[cfg(not(feature = "std"))]
	fn fmt(&self, _: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		Ok(())
	}
}

impl<T: Config + Send + Sync> CheckNotSovereignSender<T> {
	/// Create new `SignedExtension` to check runtime version.
	pub fn new() -> Self {
		Self(sp_std::marker::PhantomData)
	}
}
// Parent:Get<> (get ), Sibling or Parachain
// for parachains check the whole bytes (last twelve bytes are 0x00)
// create parentsovereign within xcm_primitives
impl<T: Config + Send + Sync> SignedExtension for CheckNotSovereignSender<T>
where
	T::RuntimeCall: Dispatchable<Info = DispatchInfo>,
	Sibling: TypeId,
	ParentSovereign: Get<AccountId>,
{
	type AccountId = T::AccountId;
	type Call = T::RuntimeCall;
	type AdditionalSigned = ();
	type Pre = ();
	const IDENTIFIER: &'static str = "CheckNotSovereignSender";

	fn additional_signed(&self) -> sp_std::result::Result<(), TransactionValidityError> {
		Ok(())
	}

	fn pre_dispatch(
		self,
		who: &Self::AccountId,
		call: &Self::Call,
		info: &DispatchInfoOf<Self::Call>,
		len: usize,
	) -> Result<Self::Pre, TransactionValidityError> {
		self.validate(who, call, info, len).map(|_| ())
	}

	fn validate(
		&self,
		who: &Self::AccountId,
		_call: &Self::Call,
		_info: &DispatchInfoOf<Self::Call>,
		_len: usize,
	) -> TransactionValidity {
		//ParentSovereign::get().cmp(who);

		//let sovereign: Self::AccountId = ParentSovereign::get().into();

		/* if who == ParentSovereign::get() {
			return Err(TransactionValidityError::Invalid(
				InvalidTransaction::BadSigner,
			));
		} */

		// TOASK: how to encode the account properly?
		let encoded_account = who.encode();
		let [_a, _b, _c, _d] = Sibling::TYPE_ID;

		match encoded_account.as_slice() {
			[_a, _b, _c, _d, _middle @ .., 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00] => {
				return Err(TransactionValidityError::Invalid(
					InvalidTransaction::BadSigner,
				))
			}
			_ => {}
		}
		Ok(ValidTransaction::default())
	}
}

/* #[cfg(test)]
mod tests {
	use super::*;
	use crate::mock::{new_test_ext, Test, CALL};
	use frame_support::{assert_noop, assert_ok};

	#[test]
	fn zero_account_ban_works() {
		new_test_ext().execute_with(|| {
			let info = DispatchInfo::default();
			let len = 0_usize;
			assert_noop!(
				CheckNonZeroSender::<Test>::new().validate(&0, CALL, &info, len),
				InvalidTransaction::BadSigner
			);
			assert_ok!(CheckNonZeroSender::<Test>::new().validate(&1, CALL, &info, len));
		})
	}
} */
