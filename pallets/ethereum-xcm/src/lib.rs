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

//! # Ethereum Xcm pallet
//!
//! The Xcm Ethereum pallet is a bridge for Xcm Transact to Ethereum pallet

// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::comparison_chain, clippy::large_enum_variant)]

#[cfg(all(feature = "std", test))]
mod mock;
#[cfg(all(feature = "std", test))]
mod tests;

use ethereum_types::{H160, U256};
use fp_ethereum::{TransactionData, ValidatedTransaction};
use fp_evm::{CheckEvmTransaction, CheckEvmTransactionConfig, InvalidEvmTransactionError};
#[cfg(feature = "try-runtime")]
use frame_support::traits::OnRuntimeUpgradeHelpersExt;
use frame_support::{
	codec::{Decode, Encode},
	dispatch::DispatchResultWithPostInfo,
	scale_info::TypeInfo,
	traits::{EnsureOrigin, Get},
	weights::{Pays, PostDispatchInfo, Weight},
};
use frame_system::pallet_prelude::OriginFor;
use pallet_evm::GasWeightMapping;
use sp_runtime::{traits::UniqueSaturatedInto, RuntimeDebug};
use sp_std::{marker::PhantomData, prelude::*};

pub use ethereum::{
	AccessListItem, BlockV2 as Block, LegacyTransactionMessage, Log, ReceiptV3 as Receipt,
	TransactionAction, TransactionV2 as Transaction,
};
pub use fp_rpc::TransactionStatus;
pub use xcm_primitives::{EthereumXcmTransaction, XcmToEthereum};

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum RawOrigin {
	XcmEthereumTransaction(H160),
}

pub fn ensure_xcm_ethereum_transaction<OuterOrigin>(o: OuterOrigin) -> Result<H160, &'static str>
where
	OuterOrigin: Into<Result<RawOrigin, OuterOrigin>>,
{
	match o.into() {
		Ok(RawOrigin::XcmEthereumTransaction(n)) => Ok(n),
		_ => Err("bad origin: expected to be a xcm Ethereum transaction"),
	}
}

pub struct EnsureXcmEthereumTransaction;
impl<O: Into<Result<RawOrigin, O>> + From<RawOrigin>> EnsureOrigin<O>
	for EnsureXcmEthereumTransaction
{
	type Success = H160;
	fn try_origin(o: O) -> Result<Self::Success, O> {
		o.into().map(|o| match o {
			RawOrigin::XcmEthereumTransaction(id) => id,
		})
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn successful_origin() -> O {
		O::from(RawOrigin::XcmEthereumTransaction(Default::default()))
	}
}

pub use self::pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_timestamp::Config + pallet_evm::Config {
		/// Invalid transaction error
		type InvalidEvmTransactionError: From<InvalidEvmTransactionError>;
		/// Handler for applying an already validated transaction
		type ValidatedTransaction: ValidatedTransaction;
		/// Origin for xcm transact
		type XcmEthereumOrigin: EnsureOrigin<Self::Origin, Success = H160>;
		/// Maximum Weight reserved for xcm in a block
		type ReservedXcmpWeight: Get<Weight>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::origin]
	pub type Origin = RawOrigin;

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		OriginFor<T>: Into<Result<RawOrigin, OriginFor<T>>>,
	{
		/// Xcm Transact an Ethereum transaction.
		#[pallet::weight(<T as pallet_evm::Config>::GasWeightMapping::gas_to_weight({
			match xcm_transaction {
				EthereumXcmTransaction::V1(v1_tx) =>  v1_tx.gas_limit.unique_saturated_into(),
				EthereumXcmTransaction::V2(v2_tx) =>  v2_tx.gas_limit.unique_saturated_into()
			}
		}))]
		pub fn transact(
			origin: OriginFor<T>,
			xcm_transaction: EthereumXcmTransaction,
		) -> DispatchResultWithPostInfo {
			let source = T::XcmEthereumOrigin::ensure_origin(origin)?;

			let (who, account_weight) = pallet_evm::Pallet::<T>::account_basic(&source);

			let transaction: Option<Transaction> =
				xcm_transaction.into_transaction_v2(U256::zero(), who.nonce);
			if let Some(transaction) = transaction {
				let transaction_data: TransactionData = (&transaction).into();

				let _ = CheckEvmTransaction::<T::InvalidEvmTransactionError>::new(
					CheckEvmTransactionConfig {
						evm_config: T::config(),
						block_gas_limit: U256::from(
							<T as pallet_evm::Config>::GasWeightMapping::weight_to_gas(
								T::ReservedXcmpWeight::get(),
							),
						),
						base_fee: U256::zero(),
						chain_id: 0u64,
						is_transactional: true,
					},
					transaction_data.into(),
				)
				// We only validate the gas limit against the evm transaction cost.
				// No need to validate fee payment, as it is handled by the xcm executor.
				.validate_in_block_for(&who)
				.map_err(|_| sp_runtime::DispatchErrorWithPostInfo {
					post_info: PostDispatchInfo {
						actual_weight: Some(account_weight),
						pays_fee: Pays::Yes,
					},
					error: sp_runtime::DispatchError::Other(
						"Failed to validate ethereum transaction",
					),
				})?;

				T::ValidatedTransaction::apply(source, transaction)
			} else {
				Err(sp_runtime::DispatchErrorWithPostInfo {
					post_info: PostDispatchInfo {
						actual_weight: Some(account_weight),
						pays_fee: Pays::Yes,
					},
					error: sp_runtime::DispatchError::Other(
						"Cannot convert xcm payload to known type",
					),
				})
			}
		}
	}
}
