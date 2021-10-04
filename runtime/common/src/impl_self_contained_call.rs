// Copyright 2019-2021 PureStake Inc.
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

			fn check_self_contained(&self) -> Option<Result<Self::SignedInfo, TransactionValidityError>> {
				match self {
					Call::Ethereum(call) => call.check_self_contained(),
					_ => None,
				}
			}

			fn validate_self_contained(&self, info: &Self::SignedInfo) -> Option<TransactionValidity> {
				match self {
					Call::Ethereum(call) => {
						// Previously, ethereum transactions were contained in an unsigned
						// extrinsic, we now use a new form of dedicated extrinsic defined by
						// frontier, but to keep the same behavior as before, we must perform
						// the controls that were performed on the unsigned extrinsic.
						if let pallet_ethereum::Call::transact(ref eth_tx) = call {
							use sp_runtime::traits::SignedExtension as _;
							if let Err(error) = SignedExtra::validate_unsigned(
								&self,
								&self.get_dispatch_info(),
								eth_tx.input.len(),
							) {
								return Some(Err(error));
							}
						}
						// Then, do the controls defined by the ethereum pallet.
						call.validate_self_contained(info)
					}
					_ => None,
				}
			}

			fn apply_self_contained(
				self,
				info: Self::SignedInfo,
			) -> Option<sp_runtime::DispatchResultWithInfo<PostDispatchInfoOf<Self>>> {
				match self {
					call @ Call::Ethereum(pallet_ethereum::Call::transact(_)) => Some(call.dispatch(
						Origin::from(pallet_ethereum::RawOrigin::EthereumTransaction(info)),
					)),
					_ => None,
				}
			}
		}
	}
}
