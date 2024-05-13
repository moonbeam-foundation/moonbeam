// Copyright 2024 Moonbeam Foundation.
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

use frame_support::{dispatch::DispatchInfo, DefaultNoBound};
use frame_system::Config;
use parity_scale_codec::{Decode, Encode};
use sp_runtime::{
	scale_info::TypeInfo,
	traits::{DispatchInfoOf, Dispatchable, SignedExtension},
	transaction_validity::{InvalidTransaction, TransactionValidity, TransactionValidityError},
};
use sp_std::{marker::PhantomData, prelude::*};

/// Check to ensure that the extrinsic is signed
#[derive(Encode, Decode, DefaultNoBound, Clone, Eq, PartialEq, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct ForbidUnsigned<T>(PhantomData<T>);

impl<T: Config + Send + Sync> sp_std::fmt::Debug for ForbidUnsigned<T> {
	#[cfg(feature = "std")]
	fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		write!(f, "ForbidUnsigned")
	}

	#[cfg(not(feature = "std"))]
	fn fmt(&self, _: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		Ok(())
	}
}

impl<T: Config + Send + Sync> SignedExtension for ForbidUnsigned<T>
where
	T::RuntimeCall: Dispatchable<Info = DispatchInfo>,
{
	type AccountId = T::AccountId;
	type Call = T::RuntimeCall;
	type AdditionalSigned = ();
	type Pre = ();
	const IDENTIFIER: &'static str = "ForbidUnsigned";

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

	fn validate_unsigned(
		_call: &Self::Call,
		_info: &DispatchInfoOf<Self::Call>,
		_len: usize,
	) -> TransactionValidity {
		Err(TransactionValidityError::Invalid(
			InvalidTransaction::BadSigner,
		))
	}
}
