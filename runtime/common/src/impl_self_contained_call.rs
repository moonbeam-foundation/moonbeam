// Copyright 2019-2022 PureStake Inc.
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

#[macro_export]
macro_rules! impl_self_contained_call {
	{} => {
		impl fp_self_contained::SelfContainedCall for Call {
			type SignedInfo = H160;

			fn is_self_contained(&self) -> bool {
				match self {
					Call::Ethereum(call) => call.is_self_contained(),
					_ => false,
				}
			}

			fn check_self_contained(
				&self
			) -> Option<Result<Self::SignedInfo, TransactionValidityError>> {
				match self {
					Call::Ethereum(call) => call.check_self_contained(),
					_ => None,
				}
			}

			fn validate_self_contained(
				&self,
				signed_info: &Self::SignedInfo
			) -> Option<TransactionValidity> {
				match self {
					Call::Ethereum(ref call) => {
						Some(validate_self_contained_inner(&self, &call, signed_info))
					}
					_ => None,
				}
			}

			fn pre_dispatch_self_contained(
				&self,
				info: &Self::SignedInfo,
			) -> Option<Result<(), TransactionValidityError>> {
				match self {
					Call::Ethereum(call) => call.pre_dispatch_self_contained(info),
					_ => None,
				}
			}

			fn apply_self_contained(
				self,
				info: Self::SignedInfo,
			) -> Option<sp_runtime::DispatchResultWithInfo<PostDispatchInfoOf<Self>>> {
				match self {
					call @ Call::Ethereum(pallet_ethereum::Call::transact { .. }) => Some(
						call.dispatch(Origin::from(
							pallet_ethereum::RawOrigin::EthereumTransaction(info)
						))
					),
					_ => None,
				}
			}
		}

		fn validate_self_contained_inner(
			call: &Call,
			eth_call: &pallet_ethereum::Call<Runtime>,
			signed_info: &<Call as fp_self_contained::SelfContainedCall>::SignedInfo
		) -> TransactionValidity {
			if let pallet_ethereum::Call::transact { ref transaction } = eth_call {
				// Previously, ethereum transactions were contained in an unsigned
				// extrinsic, we now use a new form of dedicated extrinsic defined by
				// frontier, but to keep the same behavior as before, we must perform
				// the controls that were performed on the unsigned extrinsic.
				use sp_runtime::traits::SignedExtension as _;
				let input_len = match transaction {
					pallet_ethereum::Transaction::Legacy(t) => t.input.len(),
					pallet_ethereum::Transaction::EIP2930(t) => t.input.len(),
					pallet_ethereum::Transaction::EIP1559(t) => t.input.len(),
				};
				let extra_validation = SignedExtra::validate_unsigned(
					call,
					&call.get_dispatch_info(),
					input_len,
				)?;
				// Then, do the controls defined by the ethereum pallet.
				use fp_self_contained::SelfContainedCall as _;
				let self_contained_validation = eth_call
					.validate_self_contained(signed_info)
					.ok_or(TransactionValidityError::Invalid(InvalidTransaction::BadProof))??;

				Ok(extra_validation.combine_with(self_contained_validation))
			} else {
				Err(TransactionValidityError::Unknown(
					sp_runtime::transaction_validity::UnknownTransaction::CannotLookup
				))
			}
		}
	}
}
