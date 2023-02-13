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
use frame_support::{
	codec::{Decode, Encode, MaxEncodedLen},
	dispatch::{DispatchResultWithPostInfo, Pays, PostDispatchInfo},
	scale_info::TypeInfo,
	traits::{EnsureOrigin, Get},
	weights::Weight,
};
use frame_system::pallet_prelude::OriginFor;
use pallet_evm::{AddressMapping, GasWeightMapping};
use sp_runtime::{traits::UniqueSaturatedInto, DispatchErrorWithPostInfo, RuntimeDebug};
use sp_std::{marker::PhantomData, prelude::*};

pub use ethereum::{
	AccessListItem, BlockV2 as Block, LegacyTransactionMessage, Log, ReceiptV3 as Receipt,
	TransactionAction, TransactionV2 as Transaction,
};
pub use fp_rpc::TransactionStatus;
pub use xcm_primitives::{EnsureProxy, EthereumXcmTransaction, XcmToEthereum};

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
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

#[frame_support::pallet(dev_mode)]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_timestamp::Config + pallet_evm::Config {
		/// Invalid transaction error
		type InvalidEvmTransactionError: From<InvalidEvmTransactionError>;
		/// Handler for applying an already validated transaction
		type ValidatedTransaction: ValidatedTransaction;
		/// Origin for xcm transact
		type XcmEthereumOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = H160>;
		/// Maximum Weight reserved for xcm in a block
		type ReservedXcmpWeight: Get<Weight>;
		/// Ensure proxy
		type EnsureProxy: EnsureProxy<Self::AccountId>;
		/// The origin that is allowed to resume or suspend the XCM to Ethereum executions.
		type ControllerOrigin: EnsureOrigin<Self::RuntimeOrigin>;
	}

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(PhantomData<T>);

	/// Global nonce used for building Ethereum transaction payload.
	#[pallet::storage]
	#[pallet::getter(fn nonce)]
	pub(crate) type Nonce<T: Config> = StorageValue<_, U256, ValueQuery>;

	/// Whether or not Ethereum-XCM is suspended from executing
	#[pallet::storage]
	#[pallet::getter(fn ethereum_xcm_suspended)]
	pub(super) type EthereumXcmSuspended<T: Config> = StorageValue<_, bool, ValueQuery>;

	#[pallet::origin]
	pub type Origin = RawOrigin;

	#[pallet::error]
	pub enum Error<T> {
		/// Xcm to Ethereum execution is suspended
		EthereumXcmExecutionSuspended,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		OriginFor<T>: Into<Result<RawOrigin, OriginFor<T>>>,
	{
		/// Xcm Transact an Ethereum transaction.
		/// Weight: Gas limit plus the db read involving the suspension check
		#[pallet::weight({
			let without_base_extrinsic_weight = false;
			<T as pallet_evm::Config>::GasWeightMapping::gas_to_weight({
				match xcm_transaction {
					EthereumXcmTransaction::V1(v1_tx) =>  v1_tx.gas_limit.unique_saturated_into(),
					EthereumXcmTransaction::V2(v2_tx) =>  v2_tx.gas_limit.unique_saturated_into()
				}
			}, without_base_extrinsic_weight).saturating_add(T::DbWeight::get().reads(1))
		})]
		pub fn transact(
			origin: OriginFor<T>,
			xcm_transaction: EthereumXcmTransaction,
		) -> DispatchResultWithPostInfo {
			let source = T::XcmEthereumOrigin::ensure_origin(origin)?;
			ensure!(
				!EthereumXcmSuspended::<T>::get(),
				DispatchErrorWithPostInfo {
					error: Error::<T>::EthereumXcmExecutionSuspended.into(),
					post_info: PostDispatchInfo {
						actual_weight: Some(T::DbWeight::get().reads(1)),
						pays_fee: Pays::Yes
					}
				}
			);
			Self::validate_and_apply(source, xcm_transaction)
		}

		/// Xcm Transact an Ethereum transaction through proxy.
		/// Weight: Gas limit plus the db reads involving the suspension and proxy checks
		#[pallet::weight({
			let without_base_extrinsic_weight = false;
			<T as pallet_evm::Config>::GasWeightMapping::gas_to_weight({
				match xcm_transaction {
					EthereumXcmTransaction::V1(v1_tx) =>  v1_tx.gas_limit.unique_saturated_into(),
					EthereumXcmTransaction::V2(v2_tx) =>  v2_tx.gas_limit.unique_saturated_into()
				}
			}, without_base_extrinsic_weight).saturating_add(T::DbWeight::get().reads(2))
		})]
		pub fn transact_through_proxy(
			origin: OriginFor<T>,
			transact_as: H160,
			xcm_transaction: EthereumXcmTransaction,
		) -> DispatchResultWithPostInfo {
			let source = T::XcmEthereumOrigin::ensure_origin(origin)?;
			ensure!(
				!EthereumXcmSuspended::<T>::get(),
				DispatchErrorWithPostInfo {
					error: Error::<T>::EthereumXcmExecutionSuspended.into(),
					post_info: PostDispatchInfo {
						actual_weight: Some(T::DbWeight::get().reads(1)),
						pays_fee: Pays::Yes
					}
				}
			);
			let _ = T::EnsureProxy::ensure_ok(
				T::AddressMapping::into_account_id(transact_as),
				T::AddressMapping::into_account_id(source),
			)
			.map_err(|e| sp_runtime::DispatchErrorWithPostInfo {
				post_info: PostDispatchInfo {
					actual_weight: Some(T::DbWeight::get().reads(2)),
					pays_fee: Pays::Yes,
				},
				error: sp_runtime::DispatchError::Other(e),
			})?;

			Self::validate_and_apply(transact_as, xcm_transaction)
		}

		/// Suspends all Ethereum executions from XCM.
		///
		/// - `origin`: Must pass `ControllerOrigin`.
		#[pallet::weight((T::DbWeight::get().writes(1), DispatchClass::Operational,))]
		pub fn suspend_ethereum_xcm_execution(origin: OriginFor<T>) -> DispatchResult {
			T::ControllerOrigin::ensure_origin(origin)?;

			EthereumXcmSuspended::<T>::put(true);

			Ok(())
		}

		/// Resumes all Ethereum executions from XCM.
		///
		/// - `origin`: Must pass `ControllerOrigin`.
		#[pallet::weight((T::DbWeight::get().writes(1), DispatchClass::Operational,))]
		pub fn resume_ethereum_xcm_execution(origin: OriginFor<T>) -> DispatchResult {
			T::ControllerOrigin::ensure_origin(origin)?;

			EthereumXcmSuspended::<T>::put(false);

			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	fn validate_and_apply(
		source: H160,
		xcm_transaction: EthereumXcmTransaction,
	) -> DispatchResultWithPostInfo {
		// The lack of a real signature where different callers with the
		// same nonce are providing identical transaction payloads results in a collision and
		// the same ethereum tx hash.
		// We use a global nonce instead the user nonce for all Xcm->Ethereum transactions to avoid
		// this.
		let current_nonce = Self::nonce();
		let error_weight = T::DbWeight::get().reads(1);

		let transaction: Option<Transaction> =
			xcm_transaction.into_transaction_v2(current_nonce, T::ChainId::get());
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
			.validate_common()
			.map_err(|_| sp_runtime::DispatchErrorWithPostInfo {
				post_info: PostDispatchInfo {
					actual_weight: Some(error_weight),
					pays_fee: Pays::Yes,
				},
				error: sp_runtime::DispatchError::Other("Failed to validate ethereum transaction"),
			})?;

			// Once we know a new transaction hash exists - the user can afford storing the
			// transaction on chain - we increase the global nonce.
			<Nonce<T>>::put(current_nonce.saturating_add(U256::one()));

			T::ValidatedTransaction::apply(source, transaction)
		} else {
			Err(sp_runtime::DispatchErrorWithPostInfo {
				post_info: PostDispatchInfo {
					actual_weight: Some(error_weight),
					pays_fee: Pays::Yes,
				},
				error: sp_runtime::DispatchError::Other("Cannot convert xcm payload to known type"),
			})
		}
	}
}
