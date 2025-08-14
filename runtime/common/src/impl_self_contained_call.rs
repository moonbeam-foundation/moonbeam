// Copyright 2019-2025 PureStake Inc.
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
		impl fp_self_contained::SelfContainedCall for RuntimeCall {
			type SignedInfo = H160;

			fn is_self_contained(&self) -> bool {
				match self {
					RuntimeCall::Ethereum(call) => call.is_self_contained(),
					_ => false,
				}
			}

			fn check_self_contained(
				&self
			) -> Option<Result<Self::SignedInfo, TransactionValidityError>> {
				match self {
					RuntimeCall::Ethereum(call) => call.check_self_contained(),
					_ => None,
				}
			}

			fn validate_self_contained(
				&self,
				signed_info: &Self::SignedInfo,
				dispatch_info: &DispatchInfoOf<RuntimeCall>,
				len: usize,
			) -> Option<TransactionValidity> {
				match self {
					RuntimeCall::Ethereum(call) => call.validate_self_contained(signed_info, dispatch_info, len),
					_ => None,
				}
			}

			fn pre_dispatch_self_contained(
				&self,
				info: &Self::SignedInfo,
				dispatch_info: &DispatchInfoOf<RuntimeCall>,
				len: usize,
			) -> Option<Result<(), TransactionValidityError>> {
				match self {
					RuntimeCall::Ethereum(call) => call.pre_dispatch_self_contained(info, dispatch_info, len),
					_ => None,
				}
			}

			fn apply_self_contained(
				self,
				info: Self::SignedInfo,
			) -> Option<sp_runtime::DispatchResultWithInfo<PostDispatchInfoOf<Self>>> {
				match self {
					call @ RuntimeCall::Ethereum(pallet_ethereum::Call::transact { .. }) => Some(
						call.dispatch(RuntimeOrigin::from(
							pallet_ethereum::RawOrigin::EthereumTransaction(info)
						))
					),
					_ => None,
				}
			}
		}
	}
}
